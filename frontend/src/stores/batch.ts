import { defineStore } from "pinia";
import { getBatch, queueBatchAction, removeBatchAction, removeBatchGroup } from "../api/client";
import type { BatchAction, BatchRun, QueueRequest } from "../types/batch";

interface BatchState {
  actions: BatchAction[];
  status: BatchRun["status"];
  loading: boolean;
  error: string | null;
  lastSkippedConflicts: string[];
}

export const useBatchStore = defineStore("batch", {
  state: (): BatchState => ({
    actions: [],
    status: "idle",
    loading: false,
    error: null,
    lastSkippedConflicts: [],
  }),
  getters: {
    count: (state) => state.actions.length,
  },
  actions: {
    applyRun(run: BatchRun) {
      this.actions = run.actions;
      this.status = run.status;
    },
    async fetch() {
      this.loading = true;
      this.error = null;
      try {
        this.applyRun(await getBatch());
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
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
        this.error = e instanceof Error ? e.message : String(e);
        throw e;
      }
    },
    async remove(id: string) {
      this.error = null;
      try {
        this.applyRun(await removeBatchAction(id));
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        throw e;
      }
    },
    async removeGroup(groupId: string) {
      this.error = null;
      try {
        this.applyRun(await removeBatchGroup(groupId));
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        throw e;
      }
    },
  },
});
