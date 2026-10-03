import type { BootstrapPayload } from "../shared/contracts";
import type { Translation } from "../shared/i18n";

/** Describes recordings that started or ended between two engine snapshots. */
export function recordingNotifications(
  previous: BootstrapPayload,
  next: BootstrapPayload,
  t: Translation,
): string[] {
  const previousTargets = new Map(previous.targets.map((target) => [target.id, target]));
  const messages: string[] = [];
  for (const target of next.targets) {
    const before = previousTargets.get(target.id);
    if (!before) continue;
    const wasRecording = before.state === "Recording";
    const isRecording = target.state === "Recording";
    if (!wasRecording && isRecording) {
      messages.push(t.toast.recordingStartedFor(target.name));
    } else if (wasRecording && !isRecording && before.activeJobId) {
      // Stops the user asked for (Stop, Pause all, disabling a source) are
      // recorded as "Cancelled" and do not need a notification.
      const job = next.jobs.find((item) => item.id === before.activeJobId);
      if (job?.state === "Completed") messages.push(t.toast.recordingFinishedFor(target.name));
      else if (job?.state === "Failed") messages.push(t.toast.recordingFailedFor(target.name));
    }
  }
  return messages;
}
