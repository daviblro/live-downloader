import type { CSSProperties } from "react";

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
};

export function StatusDot({ state, compact = false }: { state: string; compact?: boolean }) {
  return <span className={`status ${stateClass[state] ?? "muted"} ${compact ? "compact" : ""}`}><i />{!compact && state}</span>;
}

export function Avatar({ label, index = 0 }: { label: string; index?: number }) {
  const colors = ["#1f7aff", "#7e5bef", "#1d9e8a", "#e46b3f", "#c17b1c", "#5165d6"];
  const style = { "--avatar": colors[index % colors.length] } as CSSProperties;
  return <span className="avatar" style={style}>{label.slice(0, 1).toUpperCase()}</span>;
}
