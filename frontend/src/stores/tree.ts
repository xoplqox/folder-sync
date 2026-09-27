import { defineStore } from "pinia";
import { getTree } from "../api/client";
import type { MergedTree } from "../types/tree";
import type { ComparisonMode } from "../types/config";

interface TreeState {
  tree: MergedTree | null;
  loading: boolean;
  error: string | null;
}

export const useTreeStore = defineStore("tree", {
  state: (): TreeState => ({
    tree: null,
    loading: false,
    error: null,
  }),
  actions: {
    async fetch(name: string, number: string, mode?: ComparisonMode) {
      this.loading = true;
      this.error = null;
      try {
        this.tree = await getTree(name, number, mode);
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        this.tree = null;
      } finally {
        this.loading = false;
      }
    },
  },
});
