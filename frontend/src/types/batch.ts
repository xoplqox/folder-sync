export type ActionKind =
  | {
      kind: "sync_file";
      rel_path: string;
      source_clone: string;
      target_clones: string[];
    }
  | {
      kind: "delete_file";
      rel_path: string;
      clones: string[];
    };

export type ActionStatus = "queued" | "running" | "done" | "failed";

export interface BatchAction {
  id: string;
  group_name: string;
  group_number: string;
  kind: ActionKind;
  status: ActionStatus;
  error: string | null;
  bytes_total: number | null;
  bytes_done: number;
  group_id: string | null;
}

export type RunStatus = "idle" | "running" | "cancelled" | "completed";

export interface BatchRun {
  actions: BatchAction[];
  status: RunStatus;
}

export type QueueRequest =
  | { kind: "sync_file"; group_name: string; group_number: string; rel_path: string }
  | { kind: "delete_file"; group_name: string; group_number: string; rel_path: string }
  | { kind: "sync_folder"; group_name: string; group_number: string; rel_path: string }
  | { kind: "delete_folder"; group_name: string; group_number: string; rel_path: string }
  | { kind: "resolve_conflict"; group_name: string; group_number: string; rel_path: string; chosen_clone: string };

export interface QueueResponseBody {
  queued: number;
  skipped_conflicts: string[];
  run: BatchRun;
}
