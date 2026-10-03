import { describe, expect, it } from "vitest";
import { demoPayload } from "../mocks/demoPayload";
import type { BootstrapPayload, RecordingJob, WatchTarget } from "../shared/contracts";
import { translations } from "../shared/i18n";
import { recordingNotifications } from "./recordingNotifications";

const t = translations.en;

const target = (state: WatchTarget["state"], activeJobId: string | null = null): WatchTarget => ({
  id: "target-1",
  name: "Example",
  url: "https://www.twitch.tv/example",
  enabled: true,
  state,
  statusDetail: "",
  nextCheckAt: null,
  lastCheckedAt: null,
  lastRecordingAt: null,
  activeJobId,
  createdAt: "2026-01-01T00:00:00Z",
  providerUserId: null,
  avatarUrl: null,
});

const job = (state: RecordingJob["state"]): RecordingJob => ({
  id: "job-1",
  targetId: "target-1",
  targetName: "Example",
  targetAvatarUrl: null,
  state,
  startedAt: "2026-01-01T00:00:00Z",
  finishedAt: null,
  outputPath: null,
  fileExists: false,
  message: "",
  processId: null,
});

const snapshot = (targets: WatchTarget[], jobs: RecordingJob[] = []): BootstrapPayload => ({
  ...demoPayload,
  targets,
  jobs,
});

describe("recordingNotifications", () => {
  it("names the channel that started recording", () => {
    expect(
      recordingNotifications(
        snapshot([target("Checking")]),
        snapshot([target("Recording", "job-1")]),
        t,
      ),
    ).toEqual([t.toast.recordingStartedFor("Example")]);
  });

  it("reports completed and failed recordings", () => {
    const recording = snapshot([target("Recording", "job-1")]);
    expect(
      recordingNotifications(recording, snapshot([target("Watching")], [job("Completed")]), t),
    ).toEqual([t.toast.recordingFinishedFor("Example")]);
    expect(
      recordingNotifications(recording, snapshot([target("Watching")], [job("Failed")]), t),
    ).toEqual([t.toast.recordingFailedFor("Example")]);
  });

  it("stays quiet for stops the user requested", () => {
    expect(
      recordingNotifications(
        snapshot([target("Recording", "job-1")]),
        snapshot([target("Watching")], [job("Cancelled")]),
        t,
      ),
    ).toEqual([]);
  });
});
