import { X } from "lucide-react";
import { type FormEvent, useState } from "react";
import type { WatchTarget } from "../../../shared/contracts";
import { useI18n } from "../../../shared/i18n";

interface AddStreamDialogProps {
  target?: WatchTarget;
  onClose: () => void;
  onSubmit: (input: { name: string; url: string }) => Promise<void>;
}

export function AddStreamDialog({ target, onClose, onSubmit }: AddStreamDialogProps) {
  const { translation: t } = useI18n();
  const [name, setName] = useState(target?.name ?? "");
  const [url, setUrl] = useState(target?.url ?? "");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  async function submit(event: FormEvent) {
    event.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      await onSubmit({ name, url });
      onClose();
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="modal-backdrop" role="presentation" onMouseDown={onClose}>
      <section
        className="modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="add-stream-title"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header>
          <div>
            <h2 id="add-stream-title">{target ? t.dialog.editTitle : t.dialog.title}</h2>
            <p>{target ? t.dialog.editDescription : t.dialog.description}</p>
          </div>
          <button
            type="button"
            className="icon-button"
            onClick={onClose}
            aria-label={t.dialog.close}
          >
            <X size={18} />
          </button>
        </header>
        <form onSubmit={submit}>
          <label>
            {t.dialog.sourceName}
            <input
              autoFocus
              required
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder={t.dialog.sourceNamePlaceholder}
            />
          </label>
          <label>
            {t.dialog.streamUrl}
            <input
              required
              type="url"
              value={url}
              onChange={(event) => setUrl(event.target.value)}
              placeholder="https://www.twitch.tv/example"
            />
          </label>
          {error && (
            <p className="form-error" role="alert">
              {error}
            </p>
          )}
          <footer>
            <button type="button" className="secondary-action" onClick={onClose}>
              {t.common.cancel}
            </button>
            <button className="primary-action" disabled={submitting}>
              {submitting
                ? target
                  ? t.common.saving
                  : t.dialog.adding
                : target
                  ? t.common.saveChanges
                  : t.common.addStream}
            </button>
          </footer>
        </form>
      </section>
    </div>
  );
}
