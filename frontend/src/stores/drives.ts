import { defineStore } from "pinia";
import { getDrives, rescanDrives } from "../api/client";
import type { DriveGroupSummary } from "../types/drive";

interface DrivesState {
  groups: DriveGroupSummary[];
  scanRoot: string;
  scanRootExists: boolean;
  loading: boolean;
  loaded: boolean;
  error: string | null;
}

export const useDrivesStore = defineStore("drives", {
  state: (): DrivesState => ({
    groups: [],
    scanRoot: "",
    scanRootExists: true,
    loading: false,
    loaded: false,
    error: null,
  }),
  actions: {
    async fetch() {
      this.loading = true;
      this.error = null;
      try {
        const res = await getDrives();
        this.groups = res.groups;
        this.scanRoot = res.scan_root;
        this.scanRootExists = res.scan_root_exists;
        this.loaded = true;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
      } finally {
        this.loading = false;
      }
    },
    async rescan() {
      this.loading = true;
      this.error = null;
      try {
        const res = await rescanDrives();
        this.groups = res.groups;
        this.scanRoot = res.scan_root;
        this.scanRootExists = res.scan_root_exists;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
      } finally {
        this.loading = false;
      }
    },
  },
});
