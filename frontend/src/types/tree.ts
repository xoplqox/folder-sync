import type { ComparisonMode } from "./config";

export type EntryKind = "file" | "folder";

export type MatchState = "present" | "differs" | "missing";

export type RollupState =
  | "identical"
  | "partially_differs"
  | "partially_missing"
  | "empty";

export interface CloneStatus {
  clone: string;
  state: MatchState;
  size: number | null;
  mtime_unix: number | null;
  hash: number | null;
}

export interface RollupCloneStatus {
  clone: string;
  state: RollupState;
}

export interface MergedNode {
  name: string;
  kind: EntryKind;
  rel_path: string;
  clones: CloneStatus[];
  children: MergedNode[];
  rollup: RollupCloneStatus[] | null;
}

export interface MergedTree {
  group_name: string;
  group_number: string;
  comparison_mode: ComparisonMode;
  root: MergedNode;
}
