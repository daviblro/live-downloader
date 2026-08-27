import { useI18n } from "../i18n";

const stateClass: Record<string, string> = {
  Recording: "recording",
  Watching: "watching",
  Checking: "checking",
  Queued: "queued",
  Retrying: "retrying",
  "Needs attention": "attention",
  Completed: "completed",
  Failed: "attention",
  Cancelled: "muted",
  Disabled: "muted",
};

export function StatusDot({ state, compact = false }: { state: string; compact?: boolean }) {
  const { translation: t } = useI18n();
  return (
    <span className={`status ${stateClass[state] ?? "muted"} ${compact ? "compact" : ""}`}>
      <i />
      {!compact && (t.states[state] ?? state)}
    </span>
  );
}
