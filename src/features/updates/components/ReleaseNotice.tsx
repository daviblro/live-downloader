import { Download, ExternalLink, X } from "lucide-react";
import { displayReleaseVersion } from "../release";
import { useI18n } from "../../../shared/i18n";

export interface AvailableRelease {
  version: string;
  url: string;
}

export function ReleaseNotice({
  release,
  onOpen,
  onDismiss,
}: {
  release: AvailableRelease;
  onOpen: () => void;
  onDismiss: () => void;
}) {
  const { translation: t } = useI18n();
  return (
    <aside className="release-notice" role="status">
      <Download size={19} aria-hidden="true" />
      <div>
        <strong>{t.release.available(displayReleaseVersion(release.version))}</strong>
        <small>{t.release.description}</small>
      </div>
      <div className="release-notice-actions">
        <button type="button" className="secondary-action" onClick={onOpen}>
          {t.release.viewRelease} <ExternalLink size={15} />
        </button>
        <button
          type="button"
          className="icon-button"
          onClick={onDismiss}
          aria-label={t.release.dismiss}
        >
          <X size={16} />
        </button>
      </div>
    </aside>
  );
}
