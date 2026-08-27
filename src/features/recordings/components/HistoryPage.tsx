import { FolderOpen, Trash2 } from "lucide-react";
import type { RecordingJob } from "../../../shared/contracts";
import { useI18n } from "../../../shared/i18n";
import { HistoryTable } from "./HistoryTable";

export function HistoryPage({
  jobs,
  onClear,
  onOpenDownloads,
  onReveal,
}: {
  jobs: RecordingJob[];
  onClear: () => void;
  onOpenDownloads: () => void;
  onReveal: (jobId: string) => void;
}) {
  const { translation: t } = useI18n();
  return (
    <section className="history-view">
      <header className="topbar">
        <div>
          <h1>{t.history.title}</h1>
          <p>{t.history.description}</p>
        </div>
        <div className="top-actions">
          <button
            type="button"
            className="secondary-action danger-action"
            disabled={!jobs.some((job) => job.state !== "Recording")}
            onClick={onClear}
          >
            <Trash2 size={16} />
            {t.history.clearHistory}
          </button>
          <button type="button" className="secondary-action" onClick={onOpenDownloads}>
            <FolderOpen size={16} />
            {t.common.openDownloads}
          </button>
        </div>
      </header>
      <div className="history-content">
        <HistoryTable jobs={jobs} onReveal={onReveal} />
      </div>
    </section>
  );
}
