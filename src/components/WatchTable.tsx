import { Ellipsis, Pause, Play, RefreshCw, Trash2 } from "lucide-react";
import { useState } from "react";
import { useI18n } from "../i18n";
import { formatDateTime } from "../lib/dates";
import type { WatchTarget } from "../types";
import { Avatar, StatusDot } from "./StatusDot";

interface WatchTableProps {
  targets: WatchTarget[];
  selectedId: string | null;
  onSelect: (target: WatchTarget) => void;
  onCheck: (target: WatchTarget) => void;
  onToggle: (target: WatchTarget) => void;
  onRemove: (target: WatchTarget) => void;
}

export function WatchTable({ targets, selectedId, onSelect, onCheck, onToggle, onRemove }: WatchTableProps) {
  const { locale, translation: t } = useI18n();
  const [openActionsId, setOpenActionsId] = useState<string | null>(null);
  const displayDateTime = (value: string | null) => value ? formatDateTime(value, locale) : "—";
  if (!targets.length) return <div className="empty-table"><strong>{t.table.emptyTitle}</strong><span>{t.table.emptyDescription}</span></div>;
  return <div className="table-wrap"><table>
    <thead><tr><th>{t.common.source}</th><th>{t.common.state}</th><th>{t.table.nextCheck}</th><th>{t.table.lastRecording}</th><th className="actions-head">{t.common.actions}</th></tr></thead>
    <tbody>{targets.map((target, index) => {
      const recording = target.state === "Recording";
      return <tr key={target.id} className={selectedId === target.id ? "selected" : ""} onClick={() => onSelect(target)}>
        <td><div className="source-cell"><Avatar label={target.name} index={index} /><div><strong>{target.name}</strong><span>{new URL(target.url).hostname.replace("www.", "")}</span></div></div></td>
        <td><StatusDot state={target.enabled ? target.state : "Cancelled"} /></td>
        <td className="muted-data">{recording ? "—" : displayDateTime(target.nextCheckAt)}</td>
        <td className="muted-data">{displayDateTime(target.lastRecordingAt)}</td>
        <td className="actions-cell"><div className="row-actions row-actions-inline" onClick={(event) => event.stopPropagation()}>
          <button type="button" className="icon-button" aria-label={t.common.checkNow(target.name)} onClick={() => onCheck(target)}><RefreshCw size={16} /></button>
          <button type="button" className="icon-button" aria-label={target.enabled ? t.common.pauseSource(target.name) : t.common.resumeSource(target.name)} onClick={() => onToggle(target)}>{target.enabled ? <Pause size={16} /> : <Play size={16} />}</button>
          <button type="button" className="icon-button danger" aria-label={t.common.removeSource(target.name)} onClick={() => onRemove(target)}><Trash2 size={16} /></button>
        </div><div
          className="compact-actions"
          onBlur={(event) => {
            if (!event.currentTarget.contains(event.relatedTarget as Node | null)) setOpenActionsId(null);
          }}
          onClick={(event) => event.stopPropagation()}
          onKeyDown={(event) => {
            if (event.key === "Escape") setOpenActionsId(null);
          }}
        >
          <button
            type="button"
            className="icon-button compact-actions-trigger"
            aria-label={`${t.common.actions}: ${target.name}`}
            aria-expanded={openActionsId === target.id}
            aria-controls={`watch-actions-${target.id}`}
            onClick={() => setOpenActionsId((current) => current === target.id ? null : target.id)}
          ><Ellipsis size={18} /></button>
          {openActionsId === target.id && <div className="compact-actions-menu" id={`watch-actions-${target.id}`} role="group" aria-label={`${t.common.actions}: ${target.name}`}>
            <button type="button" className="icon-button" aria-label={t.common.checkNow(target.name)} onClick={() => { setOpenActionsId(null); onCheck(target); }}><RefreshCw size={16} /></button>
            <button type="button" className="icon-button" aria-label={target.enabled ? t.common.pauseSource(target.name) : t.common.resumeSource(target.name)} onClick={() => { setOpenActionsId(null); onToggle(target); }}>{target.enabled ? <Pause size={16} /> : <Play size={16} />}</button>
            <button type="button" className="icon-button danger" aria-label={t.common.removeSource(target.name)} onClick={() => { setOpenActionsId(null); onRemove(target); }}><Trash2 size={16} /></button>
          </div>}
        </div></td>
      </tr>;
    })}</tbody>
  </table></div>;
}
