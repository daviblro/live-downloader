import { FolderOpen, Pause, Square } from "lucide-react";
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
  if (!target) return <aside className="inspector empty-inspector"><span>Select a source to inspect its recording activity.</span></aside>;
  const recording = target.state === "Recording";
  return <aside className="inspector">
    <h2>Recording activity</h2>
    <div className="inspector-source"><Avatar label={target.name} index={0} /><div><strong>{target.name}</strong><span>{new URL(target.url).hostname.replace("www.", "")}</span></div></div>
    <div className="inspector-state"><StatusDot state={target.state} /><span>{recording ? "Active" : target.statusDetail}</span></div>
    <div className="wave" aria-label="Healthy recording activity"><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /></div>
    <p className="signal">{recording ? "Healthy signal" : "Monitor is standing by"}</p>
    <dl className="job-details"><div><dt>Current file</dt><dd>{job?.outputPath ?? "Preparing the recording file"}</dd></div><div><dt>Process</dt><dd>{job?.processId ? `yt-dlp · PID ${job.processId}` : "No active process"}</dd></div><div><dt>Status</dt><dd>{job?.message ?? target.statusDetail}</dd></div></dl>
    <div className="inspector-actions">
      <button type="button" className="primary-action" onClick={onPause}><Pause size={16} />Pause monitoring</button>
      <button type="button" className="secondary-action" onClick={onStop} disabled={!job}><Square size={14} />Stop recording</button>
      <button type="button" className="secondary-action" onClick={onOpen}><FolderOpen size={15} />Open recording folder</button>
    </div>
  </aside>;
}
