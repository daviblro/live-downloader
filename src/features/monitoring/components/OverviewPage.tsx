import { ArrowRight, FolderOpen, Import, Pause, Play, Plus } from "lucide-react";
import { formatTime } from "../../../shared/lib/dates";
import { Avatar } from "../../../shared/components/Avatar";
import { StatusDot } from "../../../shared/components/StatusDot";
import type { BootstrapPayload, RecordingJob, WatchTarget } from "../../../shared/contracts";
import { localizeRuntimeText, useI18n } from "../../../shared/i18n";
import { WatchTable } from "../../watch-targets/components/WatchTable";
import { Inspector } from "./Inspector";

interface OverviewPageProps {
  payload: BootstrapPayload;
  selected: WatchTarget | null;
  jobs: RecordingJob[];
  onAdd: () => void;
  onCheck: (target: WatchTarget) => void;
  onImportLegacy: () => void;
  onOpenDownloads: () => void;
  onPauseAll: () => void;
  onRemove: (target: WatchTarget) => void;
  onResumeAll: () => void;
  onSelect: (target: WatchTarget) => void;
  onStop: (jobId: string) => void;
  onToggle: (target: WatchTarget) => void;
}

export function OverviewPage({
  payload,
  selected,
  jobs,
  onAdd,
  onCheck,
  onImportLegacy,
  onOpenDownloads,
  onPauseAll,
  onRemove,
  onResumeAll,
  onSelect,
  onStop,
  onToggle,
}: OverviewPageProps) {
  const { locale, translation: t } = useI18n();
  const recordings = payload.targets.filter((target) => target.state === "Recording");
  const selectedJob = selected?.activeJobId
    ? (jobs.find((job) => job.id === selected.activeJobId) ?? null)
    : null;
  const nextCheck = payload.engine.nextGlobalCheckAt
    ? formatTime(payload.engine.nextGlobalCheckAt, locale)
    : t.overview.notScheduled;

  return (
    <section className="overview-view">
      <header className="topbar">
        <div>
          <h1>{payload.engine.running ? t.overview.serviceReady : t.overview.monitoringPaused}</h1>
          <p>
            <span className="recording-count">
              {t.overview.recordings(payload.engine.activeRecordings)}
            </span>{" "}
            · {t.overview.watching(payload.engine.enabledTargets)}
          </p>
        </div>
        <div className="top-actions">
          <button type="button" className="secondary-action desktop-only" onClick={onOpenDownloads}>
            <FolderOpen size={16} />
            {t.common.downloads}
          </button>
          <button type="button" className="primary-action" onClick={onAdd}>
            <Plus size={18} />
            {t.common.addStream}
          </button>
        </div>
      </header>
      {payload.legacyConfigAvailable && (
        <button type="button" className="migration-callout" onClick={onImportLegacy}>
          <Import size={17} />
          <span>
            <strong>{t.overview.importLegacyTitle}</strong>
            <small>{t.overview.importLegacyBody}</small>
          </span>
          <ArrowRight size={17} />
        </button>
      )}
      <div className="activity-header">
        <h2>{t.overview.activeRecordings}</h2>
        <span>{localizeRuntimeText(payload.engine.sidecarStatus, t)}</span>
      </div>
      <div className="active-rail">
        {recordings.length ? (
          recordings.map((target, index) => (
            <button
              type="button"
              className={`recording-card ${selected?.id === target.id ? "active" : ""}`}
              onClick={() => onSelect(target)}
              key={target.id}
            >
              <Avatar label={target.name} index={index} />
              <span>
                <strong>{target.name}</strong>
                <small>{new URL(target.url).hostname.replace("www.", "")}</small>
              </span>
              <StatusDot state="Recording" />
            </button>
          ))
        ) : (
          <div className="inactive-rail">
            <Play size={18} />
            {t.overview.noActiveRecordings}
          </div>
        )}
      </div>
      <div className="dashboard-grid">
        <section className="watch-section">
          <div className="section-heading">
            <div>
              <h2>
                {t.nav.watchList} <span>({payload.targets.length})</span>
              </h2>
              <p>{t.overview.nextGlobalCheck(nextCheck)}</p>
            </div>
            <button
              type="button"
              className="quiet-action"
              onClick={payload.engine.running ? onPauseAll : onResumeAll}
            >
              {payload.engine.running ? (
                <>
                  <Pause size={15} />
                  {t.overview.pauseAll}
                </>
              ) : (
                <>
                  <Play size={15} />
                  {t.overview.resume}
                </>
              )}
            </button>
          </div>
          <WatchTable
            targets={payload.targets}
            selectedId={selected?.id ?? null}
            onSelect={onSelect}
            onCheck={onCheck}
            onToggle={onToggle}
            onRemove={onRemove}
          />
        </section>
        <Inspector
          target={selected}
          job={selectedJob}
          onPause={onPauseAll}
          onStop={() => selectedJob && onStop(selectedJob.id)}
          onOpen={onOpenDownloads}
        />
      </div>
      <footer className="status-footer">
        <span>
          <i className={payload.engine.running ? "online" : "offline"} />
          {payload.engine.running ? t.overview.serviceRunning : t.overview.servicePaused}
        </span>
        <span>{t.overview.authorisedOnly}</span>
      </footer>
    </section>
  );
}
