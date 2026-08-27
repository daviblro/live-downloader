import { ChevronLeft, ChevronRight, FileX2 } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { formatDateTime } from "../../../shared/lib/dates";
import { Avatar } from "../../../shared/components/Avatar";
import { StatusDot } from "../../../shared/components/StatusDot";
import type { RecordingJob } from "../../../shared/contracts";
import { localizeRuntimeText, useI18n } from "../../../shared/i18n";

const pageSize = 10;

export function HistoryTable({
  jobs,
  onReveal,
}: {
  jobs: RecordingJob[];
  onReveal: (jobId: string) => void;
}) {
  const { locale, translation: t } = useI18n();
  const [page, setPage] = useState(1);
  const pageCount = Math.max(1, Math.ceil(jobs.length / pageSize));

  useEffect(() => {
    setPage((current) => Math.min(current, pageCount));
  }, [pageCount]);

  const visibleJobs = useMemo(
    () => jobs.slice((page - 1) * pageSize, page * pageSize),
    [jobs, page],
  );

  if (!jobs.length) {
    return (
      <div className="empty-table history-empty">
        <strong>{t.history.emptyTitle}</strong>
        <span>{t.history.emptyDescription}</span>
      </div>
    );
  }

  return (
    <>
      <div className="table-wrap history-table-wrap">
        <table className="history-table">
          <thead>
            <tr>
              <th>{t.common.source}</th>
              <th>{t.history.started}</th>
              <th>{t.common.state}</th>
              <th>{t.history.details}</th>
              <th className="actions-head">{t.history.file}</th>
            </tr>
          </thead>
          <tbody>
            {visibleJobs.map((job) => {
              const deleted = Boolean(job.outputPath) && !job.fileExists;
              return (
                <tr key={job.id}>
                  <td>
                    <div className="source-cell">
                      <Avatar
                        label={job.targetName}
                        colorKey={job.targetId}
                        imageUrl={job.targetAvatarUrl}
                      />
                      <div>
                        <strong>{job.targetName}</strong>
                      </div>
                    </div>
                  </td>
                  <td className="muted-data">{formatDateTime(job.startedAt, locale)}</td>
                  <td>
                    <StatusDot state={job.state} />
                  </td>
                  <td className="history-message">{localizeRuntimeText(job.message, t)}</td>
                  <td className="history-file-cell">
                    {deleted && (
                      <span className="deleted-file-notice">
                        <FileX2 size={14} aria-hidden="true" />
                        {t.history.fileDeleted}
                      </span>
                    )}
                    {!job.outputPath && <span className="muted-data">{t.history.noFileYet}</span>}
                    <button
                      type="button"
                      className="quiet-action"
                      disabled={!job.fileExists}
                      onClick={() => onReveal(job.id)}
                    >
                      {t.history.revealFile}
                    </button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      <nav className="pagination" aria-label={t.history.pagination}>
        <span>{t.history.page(page, pageCount)}</span>
        <div>
          <button
            type="button"
            className="icon-button"
            disabled={page === 1}
            onClick={() => setPage((current) => current - 1)}
            aria-label={t.history.previousPage}
          >
            <ChevronLeft size={17} />
          </button>
          <button
            type="button"
            className="icon-button"
            disabled={page === pageCount}
            onClick={() => setPage((current) => current + 1)}
            aria-label={t.history.nextPage}
          >
            <ChevronRight size={17} />
          </button>
        </div>
      </nav>
    </>
  );
}
