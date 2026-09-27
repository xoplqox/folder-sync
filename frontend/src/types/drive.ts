export interface DriveGroupSummary {
  name: string;
  number: string;
  clone_count: number;
  clones: string[];
}

export interface DrivesResponse {
  scan_root: string;
  scan_root_exists: boolean;
  groups: DriveGroupSummary[];
}
