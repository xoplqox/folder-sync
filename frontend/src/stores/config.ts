import { defineStore } from "pinia";
import { getConfig, updateConfig } from "../api/client";
import type { ComparisonMode, ConfigUpdate } from "../types/config";

interface ConfigState {
  scanRoot: string;
  comparisonMode: ComparisonMode;
  readOnly: boolean;
  loading: boolean;
  loaded: boolean;
  error: string | null;
}

export const useConfigStore = defineStore("config", {
  state: (): ConfigState => ({
    scanRoot: "",
    comparisonMode: "name_size",
    readOnly: false,
    loading: false,
    loaded: false,
    error: null,
  }),
  actions: {
    async fetch() {
      this.loading = true;
      this.error = null;
      try {
        const res = await getConfig();
        this.scanRoot = res.scan_root;
        this.comparisonMode = res.comparison_mode;
        this.readOnly = res.read_only;
        this.loaded = true;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
      } finally {
        this.loading = false;
      }
    },
    async update(patch: ConfigUpdate) {
      this.loading = true;
      this.error = null;
      try {
        const res = await updateConfig(patch);
        this.scanRoot = res.scan_root;
        this.comparisonMode = res.comparison_mode;
        this.readOnly = res.read_only;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        throw e;
      } finally {
        this.loading = false;
      }
    },
  },
});
