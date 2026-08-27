export type Theme = "system" | "light" | "dark";
export type Locale = "en" | "pt-BR";
export type TargetState =
  "Watching" | "Checking" | "Recording" | "Queued" | "Retrying" | "Needs attention" | "Disabled";
export type RecordingState = "Recording" | "Completed" | "Failed" | "Cancelled" | "Interrupted";

export interface AppSettings {
  locale: Locale;
  theme: Theme;
  downloadDirectory: string;
  probeIntervalSeconds: number;
  maxConcurrentRecordings: number;
  startWithWindows: boolean;
  notificationsEnabled: boolean;
  externalYtdlpPath: string | null;
}

export interface WatchTarget {
  id: string;
  name: string;
  url: string;
  enabled: boolean;
  state: TargetState;
  statusDetail: string;
  nextCheckAt: string | null;
  lastCheckedAt: string | null;
  lastRecordingAt: string | null;
  activeJobId: string | null;
  createdAt: string;
}

export interface RecordingJob {
  id: string;
  targetId: string;
  targetName: string;
  state: RecordingState;
  startedAt: string;
  finishedAt: string | null;
  outputPath: string | null;
  fileExists: boolean;
  message: string;
  processId: number | null;
}

export interface EngineSummary {
  running: boolean;
  activeRecordings: number;
  enabledTargets: number;
  nextGlobalCheckAt: string | null;
  sidecarStatus: string;
}

export interface DiskUsage {
  totalBytes: number;
  availableBytes: number;
}

export interface BootstrapPayload {
  settings: AppSettings;
  diskUsage: DiskUsage | null;
  targets: WatchTarget[];
  jobs: RecordingJob[];
  engine: EngineSummary;
  legacyConfigAvailable: boolean;
}

export interface CreateTargetInput {
  name: string;
  url: string;
}

export interface UpdateTargetInput extends CreateTargetInput {
  id: string;
  enabled: boolean;
}
