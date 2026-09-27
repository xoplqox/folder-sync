export type ComparisonMode = "name_size" | "name_size_hash";

export interface ConfigResponse {
  scan_root: string;
  comparison_mode: ComparisonMode;
  read_only: boolean;
}

export interface ConfigUpdate {
  scan_root?: string;
  comparison_mode?: ComparisonMode;
}
