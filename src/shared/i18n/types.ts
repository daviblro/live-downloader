import type { Locale } from "../contracts";

export const localeOptions: ReadonlyArray<{ value: Locale; nativeName: string }> = [
  { value: "en", nativeName: "English" },
  { value: "pt-BR", nativeName: "Português (Brasil)" },
];

export interface Translation {
  nav: Record<"overview" | "watchList" | "history" | "settings" | "help", string>;
  sidebar: Record<
    "primaryNavigation" | "diskHealth" | "good" | "libraryReady" | "recordingPath",
    string
  > & { usedOfTotal: (used: string, total: string, percent: number) => string };
  overview: {
    serviceReady: string;
    monitoringPaused: string;
    recordings: (count: number) => string;
    watching: (count: number) => string;
    importLegacyTitle: string;
    importLegacyBody: string;
    activeRecordings: string;
    noActiveRecordings: string;
    nextGlobalCheck: (value: string) => string;
    pauseAll: string;
    resume: string;
    serviceRunning: string;
    servicePaused: string;
    authorisedOnly: string;
    notScheduled: string;
    starting: string;
  };
  common: {
    addStream: string;
    downloads: string;
    openDownloads: string;
    actions: string;
    source: string;
    state: string;
    cancel: string;
    saveChanges: string;
    saving: string;
    saved: string;
    pause: string;
    resume: string;
    remove: string;
    switchView: string;
    checkNow: (name: string) => string;
    editSource: (name: string) => string;
    pauseSource: (name: string) => string;
    resumeSource: (name: string) => string;
    removeSource: (name: string) => string;
  };
  list: {
    title: string;
    description: string;
    filterPlaceholder: string;
    sources: (count: number) => string;
  };
  history: {
    title: string;
    description: string;
    started: string;
    details: string;
    file: string;
    revealFile: string;
    noFileYet: string;
    fileDeleted: string;
    clearHistory: string;
    clearConfirmation: string;
    emptyTitle: string;
    emptyDescription: string;
    pagination: string;
    page: (current: number, total: number) => string;
    previousPage: string;
    nextPage: string;
  };
  help: {
    title: string;
    description: string;
    openIssue: string;
    youtubeTitle: string;
    youtubeBody: string;
    startTitle: string;
    startBody: string;
    accessTitle: string;
    accessBody: string;
    moreTitle: string;
    moreBody: string;
  };
  settings: {
    title: string;
    description: string;
    appearance: string;
    language: string;
    theme: string;
    useWindowsSetting: string;
    dark: string;
    light: string;
    notifications: string;
    notificationsDescription: string;
    recordingLibrary: string;
    downloadDirectory: string;
    monitoring: string;
    checkEvery: string;
    seconds: string;
    concurrentRecordings: string;
    slots: string;
    windowsStartup: string;
    startWithWindows: string;
    startWithWindowsDescription: string;
    savedLocally: string;
    twitch: string;
    twitchDescription: string;
    twitchConnectedAs: (login: string) => string;
    twitchDisconnected: string;
    twitchUnavailable: string;
    connectTwitch: string;
    disconnectTwitch: string;
    waitingForTwitch: string;
    activationCode: string;
    twitchExpired: string;
  };
  dialog: {
    title: string;
    description: string;
    editTitle: string;
    editDescription: string;
    close: string;
    sourceName: string;
    sourceNamePlaceholder: string;
    streamUrl: string;
    adding: string;
  };
  inspector: {
    selectSource: string;
    recordingActivity: string;
    active: string;
    healthyRecordingActivity: string;
    healthySignal: string;
    monitorStandingBy: string;
    currentFile: string;
    preparingFile: string;
    process: string;
    noActiveProcess: string;
    status: string;
    pauseMonitoring: string;
    stopRecording: string;
    openRecordingFolder: string;
  };
  table: {
    emptyTitle: string;
    emptyDescription: string;
    nextCheck: string;
    lastRecording: string;
  };
  release: {
    available: (version: string) => string;
    description: string;
    viewRelease: string;
    dismiss: string;
  };
  toast: {
    notifications: string;
    close: string;
    settingsSaved: string;
    downloadsOpened: string;
    legacyImported: string;
    monitoringPaused: string;
    monitoringResumed: string;
    checking: (name: string) => string;
    recordingStopping: string;
    recordingStopped: string;
    recordingStartedFor: (name: string) => string;
    recordingFinishedFor: (name: string) => string;
    recordingFailedFor: (name: string) => string;
    historyCleared: string;
    removed: (name: string) => string;
    removeConfirmation: (name: string) => string;
    removeRecordingConfirmation: (name: string) => string;
    updated: (name: string) => string;
  };
  states: Record<string, string>;
  runtime: Record<string, string>;
}
