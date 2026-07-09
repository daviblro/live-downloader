import { Ellipsis, Pause, Play, RefreshCw, Trash2 } from "lucide-react";
import type { WatchTarget } from "../types";
import { Avatar, StatusDot } from "./StatusDot";

const formatTime = (value: string | null) => value ? new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit", month: "short", day: "numeric" }).format(new Date(value)) : "—";

interface WatchTableProps {
  targets: WatchTarget[];
  selectedId: string | null;
  onSelect: (target: WatchTarget) => void;
  onCheck: (target: WatchTarget) => void;
  onToggle: (target: WatchTarget) => void;
  onRemove: (target: WatchTarget) => void;
}

export function WatchTable({ targets, selectedId, onSelect, onCheck, onToggle, onRemove }: WatchTableProps) {
  if (!targets.length) {
    return <div className="empty-table"><strong>Your watch list is empty.</strong><span>Add a public stream URL to begin monitoring.</span></div>;
  }
  return <div className="table-wrap"><table>
    <thead><tr><th>Source</th><th>State</th><th>Next check</th><th>Last recording</th><th className="actions-head">Actions</th></tr></thead>
    <tbody>{targets.map((target, index) => {
      const recording = target.state === "Recording";
      return <tr key={target.id} className={selectedId === target.id ? "selected" : ""} onClick={() => onSelect(target)}>
        <td><div className="source-cell"><Avatar label={target.name} index={index} /><div><strong>{target.name}</strong><span>{new URL(target.url).hostname.replace("www.", "")}</span></div></div></td>
        <td><StatusDot state={target.enabled ? target.state : "Cancelled"} /></td>
        <td className="muted-data">{recording ? "—" : formatTime(target.nextCheckAt)}</td>
        <td className="muted-data">{formatTime(target.lastRecordingAt)}</td>
        <td><div className="row-actions" onClick={(event) => event.stopPropagation()}>
          <button type="button" className="icon-button" aria-label={`Check ${target.name} now`} onClick={() => onCheck(target)}><RefreshCw size={16} /></button>
          <button type="button" className="icon-button" aria-label={`${target.enabled ? "Pause" : "Resume"} ${target.name}`} onClick={() => onToggle(target)}>{target.enabled ? <Pause size={16} /> : <Play size={16} />}</button>
          <button type="button" className="icon-button danger" aria-label={`Remove ${target.name}`} onClick={() => onRemove(target)}><Trash2 size={16} /></button>
          <Ellipsis size={18} className="more-icon" aria-hidden="true" />
        </div></td>
      </tr>;
    })}</tbody>
  </table></div>;
}
