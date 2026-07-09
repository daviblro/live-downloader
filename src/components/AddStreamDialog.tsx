import { X } from "lucide-react";
import { FormEvent, useState } from "react";

interface AddStreamDialogProps {
  onClose: () => void;
  onSubmit: (input: { name: string; url: string }) => Promise<void>;
}

export function AddStreamDialog({ onClose, onSubmit }: AddStreamDialogProps) {
  const [name, setName] = useState("");
  const [url, setUrl] = useState("");
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

  return <div className="modal-backdrop" role="presentation" onMouseDown={onClose}>
    <section className="modal" role="dialog" aria-modal="true" aria-labelledby="add-stream-title" onMouseDown={(event) => event.stopPropagation()}>
      <header><div><h2 id="add-stream-title">Add stream</h2><p>Live Downloader will validate the URL before it is watched.</p></div><button type="button" className="icon-button" onClick={onClose} aria-label="Close"><X size={18} /></button></header>
      <form onSubmit={submit}>
        <label>Source name<input autoFocus required value={name} onChange={(event) => setName(event.target.value)} placeholder="e.g. Northernlight" /></label>
        <label>Stream URL<input required type="url" value={url} onChange={(event) => setUrl(event.target.value)} placeholder="https://www.twitch.tv/example" /></label>
        {error && <p className="form-error" role="alert">{error}</p>}
        <footer><button type="button" className="secondary-action" onClick={onClose}>Cancel</button><button className="primary-action" disabled={submitting}>{submitting ? "Adding…" : "Add stream"}</button></footer>
      </form>
    </section>
  </div>;
}
