export type UpdatePhase =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "installed"
  | "upToDate"
  | "error";

export interface AvailableUpdate {
  version: string;
  currentVersion: string;
  body: string | null;
  date: string | null;
}

export interface DownloadProgress {
  downloaded: number;
  total: number | null;
}
