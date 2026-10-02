import { useEffect, useState, type CSSProperties } from "react";

const colors = ["#1f7aff", "#7e5bef", "#1d9e8a", "#e46b3f", "#c17b1c", "#5165d6"];

export function avatarColor(value: string) {
  let hash = 2166136261;
  for (const character of value.toLowerCase()) {
    hash ^= character.codePointAt(0) ?? 0;
    hash = Math.imul(hash, 16777619);
  }
  return colors[(hash >>> 0) % colors.length];
}

export function Avatar({
  label,
  colorKey = label,
  imageUrl,
}: {
  label: string;
  colorKey?: string;
  imageUrl?: string | null;
}) {
  const [imageFailed, setImageFailed] = useState(false);
  useEffect(() => setImageFailed(false), [imageUrl]);
  const style = { "--avatar": avatarColor(colorKey) } as CSSProperties;
  return (
    <span className="avatar" style={style} aria-label={label}>
      {imageUrl && !imageFailed ? (
        <img src={imageUrl} alt="" onError={() => setImageFailed(true)} />
      ) : (
        label.slice(0, 1).toUpperCase()
      )}
    </span>
  );
}
