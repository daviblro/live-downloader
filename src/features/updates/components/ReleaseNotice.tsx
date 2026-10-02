import { Download, ExternalLink, RefreshCw, X } from "lucide-react";
import { displayReleaseVersion } from "../release";
import { useI18n } from "../../../shared/i18n";

export interface AvailableRelease {
  version: string;
  url: string;
  /** True when the release can be downloaded and installed from inside the app. */
  installable: boolean;
}

export type UpdateInstallState =
  | { phase: "idle" }
  | { phase: "downloading"; percent: number | null }
  | { phase: "installing" }
  | { phase: "failed"; message: string };

export function ReleaseNotice({
  release,
  installState,
  onInstall,
  onOpen,
  onDismiss,
}: {
  release: AvailableRelease;
  installState: UpdateInstallState;
  onInstall: () => void;
  onOpen: () => void;
  onDismiss: () => void;
}) {
  const { translation: t } = useI18n();
  const busy = installState.phase === "downloading" || installState.phase === "installing";
  const description =
    installState.phase === "downloading"
      ? t.release.downloading(installState.percent)
      : installState.phase === "installing"
        ? t.release.installing
        : installState.phase === "failed"
          ? t.release.failed(installState.message)
          : release.installable
            ? t.release.installableDescription
            : t.release.description;
  return (
    <aside className="release-notice" role="status">
      <Download size={19} aria-hidden="true" />
      <div>
        <strong>{t.release.available(displayReleaseVersion(release.version))}</strong>
        <small>{description}</small>
      </div>
      <div className="release-notice-actions">
        {release.installable && (
          <button type="button" className="primary-action" onClick={onInstall} disabled={busy}>
            <RefreshCw size={15} aria-hidden="true" />
            {busy ? t.release.updating : t.release.updateNow}
          </button>
        )}
        <button type="button" className="secondary-action" onClick={onOpen}>
          {t.release.viewRelease} <ExternalLink size={15} />
        </button>
        <button
          type="button"
          className="icon-button"
          onClick={onDismiss}
          disabled={busy}
          aria-label={t.release.dismiss}
        >
          <X size={16} />
        </button>
      </div>
    </aside>
  );
}
