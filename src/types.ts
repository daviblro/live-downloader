export type Theme = "system" | "light" | "dark";
export type Locale = "en" | "pt-BR";

export interface AppSettings {
  locale: Locale;
  theme: Theme;
  downloadDirectory: string;
  probeIntervalSeconds: number;
  maxConcurrentProbes: number;
  maxConcurrentRecordings: number;
  launchToTray: boolean;
  startWithWindows: boolean;
  notificationsEnabled: boolean;
  retainLogsDays: number;
  externalYtdlpPath: string | null;
}

export interface WatchTarget {
  id: string;
  name: string;
  url: string;
  enabled: boolean;
  state: string;
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
  state: string;
  startedAt: string;
  finishedAt: string | null;
  outputPath: string | null;
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

export interface BootstrapPayload {
  settings: AppSettings;
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
