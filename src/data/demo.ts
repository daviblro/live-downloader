import type { BootstrapPayload } from "../types";

const now = new Date();
const at = (minutes: number) => new Date(now.getTime() + minutes * 60_000).toISOString();

export const demoPayload: BootstrapPayload = {
  settings: {
    locale: "en",
    theme: "dark",
    downloadDirectory: "D:\\LiveDownloader\\Recordings",
    probeIntervalSeconds: 300,
    maxConcurrentProbes: 6,
    maxConcurrentRecordings: 3,
    launchToTray: true,
    startWithWindows: false,
    notificationsEnabled: true,
    retainLogsDays: 30,
    externalYtdlpPath: null,
  },
  engine: {
    running: true,
    activeRecordings: 3,
    enabledTargets: 6,
    nextGlobalCheckAt: at(2),
    sidecarStatus: "Managed yt-dlp sidecar",
  },
  diskUsage: { totalBytes: 4_000_000_000_000, availableBytes: 2_100_000_000_000 },
  legacyConfigAvailable: true,
  targets: [
    { id: "1", name: "soucarlosdaniel", url: "https://www.twitch.tv/soucarlosdaniel", enabled: true, state: "Recording", statusDetail: "Recording in the background", nextCheckAt: null, lastCheckedAt: at(-1), lastRecordingAt: at(-80), activeJobId: "job-1", createdAt: at(-10000) },
    { id: "2", name: "Northernlight", url: "https://www.youtube.com/@northernlight/live", enabled: true, state: "Recording", statusDetail: "Recording in the background", nextCheckAt: null, lastCheckedAt: at(-1), lastRecordingAt: at(-65), activeJobId: "job-2", createdAt: at(-10000) },
    { id: "3", name: "PixelSonic", url: "https://www.twitch.tv/pixelsonic", enabled: true, state: "Recording", statusDetail: "Recording in the background", nextCheckAt: null, lastCheckedAt: at(-1), lastRecordingAt: at(-42), activeJobId: "job-3", createdAt: at(-10000) },
    { id: "4", name: "RetroRumble", url: "https://www.youtube.com/@retrorumble/live", enabled: true, state: "Watching", statusDetail: "Waiting for the stream to go live", nextCheckAt: at(1), lastCheckedAt: at(-4), lastRecordingAt: at(-1440), activeJobId: null, createdAt: at(-10000) },
    { id: "5", name: "CodeCove", url: "https://www.twitch.tv/codecove", enabled: true, state: "Watching", statusDetail: "Waiting for the stream to go live", nextCheckAt: at(4), lastCheckedAt: at(-1), lastRecordingAt: at(-2000), activeJobId: null, createdAt: at(-10000) },
    { id: "6", name: "WanderLens", url: "https://www.youtube.com/@wanderlens/live", enabled: true, state: "Watching", statusDetail: "Waiting for the stream to go live", nextCheckAt: at(6), lastCheckedAt: at(-3), lastRecordingAt: at(-2000), activeJobId: null, createdAt: at(-10000) },
  ],
  jobs: [
    { id: "job-1", targetId: "1", targetName: "soucarlosdaniel", state: "Recording", startedAt: at(-84), finishedAt: null, outputPath: null, message: "Recording in the background", processId: 4312 },
    { id: "job-2", targetId: "2", targetName: "Northernlight", state: "Recording", startedAt: at(-61), finishedAt: null, outputPath: null, message: "Recording in the background", processId: 8452 },
    { id: "job-3", targetId: "3", targetName: "PixelSonic", state: "Recording", startedAt: at(-29), finishedAt: null, outputPath: null, message: "Recording in the background", processId: 6784 },
    { id: "job-4", targetId: "4", targetName: "RetroRumble", state: "Completed", startedAt: at(-1500), finishedAt: at(-1440), outputPath: "D:\\LiveDownloader\\Recordings\\RetroRumble_20260708.mp4", message: "Recording completed", processId: null },
  ],
};
