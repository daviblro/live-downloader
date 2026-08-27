import type { CSSProperties } from "react";

export function Avatar({ label, index = 0 }: { label: string; index?: number }) {
  const colors = ["#1f7aff", "#7e5bef", "#1d9e8a", "#e46b3f", "#c17b1c", "#5165d6"];
  const style = { "--avatar": colors[index % colors.length] } as CSSProperties;
  return (
    <span className="avatar" style={style}>
      {label.slice(0, 1).toUpperCase()}
    </span>
  );
}
