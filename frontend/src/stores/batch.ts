import { defineStore } from "pinia";
import { cancelBatch, getBatch, queueBatchAction, removeBatchAction, removeBatchGroup, startBatch } from "../api/client";
import type { BatchAction, BatchRun, QueueRequest } from "../types/batch";
import type { ProgressEvent } from "../api/ws";
import { useToastStore } from "./toast";

interface FinishedActionRef {
  group_name: string;
  group_number: string;
}

interface BatchState {
  actions: BatchAction[];
  status: BatchRun["status"];
  loading: boolean;
  error: string | null;
  lastSkippedConflicts: string[];
  lastCompleted: { succeeded: number; failed: number } | null;
  /** Set (to a fresh object) whenever an action finishes successfully, so views can react to it. */
  lastFinishedAction: FinishedActionRef | null;
  /** True from the moment "Batch starten" is clicked until the start request resolves — for immediate button feedback. */
  starting: boolean;
  /** True from the moment "Abbrechen" is clicked until the cancel request resolves. */
  cancelling: boolean;
}

function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

// Module-scoped (not store state) since it's a plain timer handle, not
// something views should react to — same pattern as the WS reconnect timer.
let pollTimer: ReturnType<typeof setInterval> | undefined;

export const useBatchStore = defineStore("batch", {
  state: (): BatchState => ({
    actions: [],
    status: "idle",
    loading: false,
    error: null,
    lastSkippedConflicts: [],
    lastCompleted: null,
    lastFinishedAction: null,
    starting: false,
    cancelling: false,
  }),
  getters: {
    count: (state) => state.actions.length,
    /** Actions still needing attention (queued/running/failed) — excludes successfully done ones. */
    pendingCount: (state) => state.actions.filter((a) => a.status !== "done").length,
    queuedCount: (state) => state.actions.filter((a) => a.status === "queued").length,
    isRunning: (state) => state.status === "running",
  },
  actions: {
    /**
     * Applies a fresh queue snapshot, from either the WebSocket stream or a
     * plain REST fetch/poll. Detects actions that newly transitioned to
     * "done" (compared to the previous state) and records one as
     * `lastFinishedAction`, so views can react the same way regardless of
     * which channel delivered the update — this is what keeps the UI live
     * even if the WebSocket never connects (e.g. blocked by a proxy).
     */
    applyRun(run: BatchRun) {
      const previousStatus = new Map(this.actions.map((a) => [a.id, a.status]));
      for (const action of run.actions) {
        if (action.status === "done" && previousStatus.get(action.id) !== "done") {
          this.lastFinishedAction = { group_name: action.group_name, group_number: action.group_number };
        }
      }
      this.actions = run.actions;
      this.status = run.status;
    },
    handleEvent(event: ProgressEvent) {
      if (event.type === "run_completed") {
        this.lastCompleted = { succeeded: event.succeeded, failed: event.failed };
        const toast = useToastStore();
        if (event.failed > 0) {
          toast.push(`Batch abgeschlossen: ${event.succeeded} erfolgreich, ${event.failed} fehlgeschlagen.`, "error", 8000);
        } else {
          toast.push(`Batch abgeschlossen: ${event.succeeded} erfolgreich.`, "success");
        }
      } else if (event.type === "run_cancelled") {
        useToastStore().push("Batch abgebrochen. Verbleibende Aktionen bleiben geplant.", "info");
      }
    },
    /**
     * Fallback for when the WebSocket stream doesn't (or can't) deliver live
     * updates — e.g. a devcontainer/proxy setup that doesn't forward
     * WebSocket upgrades. Polls GET /api/batch while a run is active so the
     * UI still catches up without requiring a manual page reload.
     */
    startPolling() {
      if (pollTimer) return;
      pollTimer = setInterval(() => {
        this.pollOnce();
      }, 1500);
    },
    stopPolling() {
      if (pollTimer) {
        clearInterval(pollTimer);
        pollTimer = undefined;
      }
    },
    async pollOnce() {
      try {
        this.applyRun(await getBatch());
      } catch {
        // Transient error — the next tick retries; don't spam toasts for this.
      }
      if (this.status !== "running") {
        this.stopPolling();
      }
    },
    async fetch() {
      this.loading = true;
      this.error = null;
      try {
        this.applyRun(await getBatch());
        if (this.status === "running") this.startPolling();
      } catch (e) {
        this.error = errorMessage(e);
      } finally {
        this.loading = false;
      }
    },
    /** Returns the count of files that need the conflict wizard (not auto-queueable). */
    async queue(req: QueueRequest): Promise<number> {
      this.error = null;
      try {
        const res = await queueBatchAction(req);
        this.applyRun(res.run);
        this.lastSkippedConflicts = res.skipped_conflicts;
        return res.skipped_conflicts.length;
      } catch (e) {
        this.error = errorMessage(e);
        throw e;
      }
    },
    async remove(id: string) {
      this.error = null;
      try {
        this.applyRun(await removeBatchAction(id));
      } catch (e) {
        this.error = errorMessage(e);
        useToastStore().push(this.error, "error");
        throw e;
      }
    },
    async removeGroup(groupId: string) {
      this.error = null;
      try {
        this.applyRun(await removeBatchGroup(groupId));
      } catch (e) {
        this.error = errorMessage(e);
        useToastStore().push(this.error, "error");
        throw e;
      }
    },
    async start() {
      this.error = null;
      this.starting = true;
      // Flip immediately so the button/UI reacts to the click without
      // waiting on the network round-trip or the next poll/WS tick — rolled
      // back below if the request actually fails.
      const previousStatus = this.status;
      this.status = "running";
      try {
        await startBatch();
        this.startPolling();
      } catch (e) {
        this.status = previousStatus;
        this.error = errorMessage(e);
        useToastStore().push(this.error, "error");
        throw e;
      } finally {
        this.starting = false;
      }
    },
    async cancel() {
      this.error = null;
      this.cancelling = true;
      try {
        await cancelBatch();
      } catch (e) {
        this.error = errorMessage(e);
        useToastStore().push(this.error, "error");
        throw e;
      } finally {
        this.cancelling = false;
      }
    },
  },
});
