import { defineStore } from "pinia";
import { cancelBatch, getBatch, queueBatchAction, removeBatchAction, removeBatchGroup, startBatch } from "../api/client";
import type { BatchAction, BatchRun, QueueRequest } from "../types/batch";
import type { ProgressEvent } from "../api/ws";
import { useToastStore } from "./toast";

interface BatchState {
  actions: BatchAction[];
  status: BatchRun["status"];
  loading: boolean;
  error: string | null;
  lastSkippedConflicts: string[];
  lastCompleted: { succeeded: number; failed: number } | null;
}

function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export const useBatchStore = defineStore("batch", {
  state: (): BatchState => ({
    actions: [],
    status: "idle",
    loading: false,
    error: null,
    lastSkippedConflicts: [],
    lastCompleted: null,
  }),
  getters: {
    count: (state) => state.actions.length,
    queuedCount: (state) => state.actions.filter((a) => a.status === "queued").length,
    isRunning: (state) => state.status === "running",
  },
  actions: {
    applyRun(run: BatchRun) {
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
    async fetch() {
      this.loading = true;
      this.error = null;
      try {
        this.applyRun(await getBatch());
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
      try {
        await startBatch();
      } catch (e) {
        this.error = errorMessage(e);
        useToastStore().push(this.error, "error");
        throw e;
      }
    },
    async cancel() {
      this.error = null;
      try {
        await cancelBatch();
      } catch (e) {
        this.error = errorMessage(e);
        useToastStore().push(this.error, "error");
        throw e;
      }
    },
  },
});
