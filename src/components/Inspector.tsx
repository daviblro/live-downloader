import { FolderOpen, Pause, Square } from "lucide-react";
import { localizeRuntimeText, useI18n } from "../i18n";
import type { RecordingJob, WatchTarget } from "../types";
import { Avatar, StatusDot } from "./StatusDot";

interface InspectorProps {
  target: WatchTarget | null;
  job: RecordingJob | null;
  onPause: () => void;
  onStop: () => void;
  onOpen: () => void;
}

export function Inspector({ target, job, onPause, onStop, onOpen }: InspectorProps) {
  const { translation: t } = useI18n();
  if (!target) return <aside className="inspector empty-inspector"><span>{t.inspector.selectSource}</span></aside>;
  const recording = target.state === "Recording";
  return <aside className="inspector">
    <h2>{t.inspector.recordingActivity}</h2>
    <div className="inspector-source"><Avatar label={target.name} index={0} /><div><strong>{target.name}</strong><span>{new URL(target.url).hostname.replace("www.", "")}</span></div></div>
    <div className="inspector-state"><StatusDot state={target.state} /><span>{recording ? t.inspector.active : localizeRuntimeText(target.statusDetail, t)}</span></div>
    <div className="wave" aria-label={t.inspector.healthyRecordingActivity}><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /></div>
    <p className="signal">{recording ? t.inspector.healthySignal : t.inspector.monitorStandingBy}</p>
    <dl className="job-details"><div><dt>{t.inspector.currentFile}</dt><dd>{job?.outputPath ?? t.inspector.preparingFile}</dd></div><div><dt>{t.inspector.process}</dt><dd>{job?.processId ? `yt-dlp · PID ${job.processId}` : t.inspector.noActiveProcess}</dd></div><div><dt>{t.inspector.status}</dt><dd>{localizeRuntimeText(job?.message ?? target.statusDetail, t)}</dd></div></dl>
    <div className="inspector-actions">
      <button type="button" className="primary-action" onClick={onPause}><Pause size={16} />{t.inspector.pauseMonitoring}</button>
      <button type="button" className="secondary-action" onClick={onStop} disabled={!job}><Square size={14} />{t.inspector.stopRecording}</button>
      <button type="button" className="secondary-action" onClick={onOpen}><FolderOpen size={15} />{t.inspector.openRecordingFolder}</button>
    </div>
  </aside>;
}
