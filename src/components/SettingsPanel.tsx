import { Check, FolderCog, Save } from "lucide-react";
import { useEffect, useState } from "react";
import type { AppSettings } from "../types";

interface SettingsPanelProps {
  settings: AppSettings;
  onSave: (settings: AppSettings) => Promise<void>;
}

export function SettingsPanel({ settings, onSave }: SettingsPanelProps) {
  const [draft, setDraft] = useState(settings);
  const [state, setState] = useState<"idle" | "saving" | "saved" | "error">("idle");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => setDraft(settings), [settings]);
  const update = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => setDraft((current) => ({ ...current, [key]: value }));

  async function save() {
    setState("saving");
    setError(null);
    try {
      await onSave(draft);
      setState("saved");
      window.setTimeout(() => setState("idle"), 2000);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
      setState("error");
    }
  }

  return <section className="settings-panel">
    <div className="view-heading"><div><h1>Settings</h1><p>Control the local engine, recording library, appearance, and background behaviour.</p></div><button type="button" className="primary-action" onClick={save} disabled={state === "saving"}><Save size={16} />{state === "saving" ? "Saving…" : state === "saved" ? "Saved" : "Save changes"}</button></div>
    <div className="settings-grid">
      <section className="settings-group"><h2>Appearance</h2><label>Theme<select value={draft.theme} onChange={(event) => update("theme", event.target.value as AppSettings["theme"])}><option value="system">Use Windows setting</option><option value="dark">Dark</option><option value="light">Light</option></select></label><label className="switch-row"><span><strong>Notifications</strong><small>Tell me when recording starts, ends, or needs attention.</small></span><input type="checkbox" checked={draft.notificationsEnabled} onChange={(event) => update("notificationsEnabled", event.target.checked)} /></label></section>
      <section className="settings-group"><h2>Recording library</h2><label>Download directory<div className="input-icon"><FolderCog size={17} /><input value={draft.downloadDirectory} onChange={(event) => update("downloadDirectory", event.target.value)} /></div></label><label>Keep diagnostics<select value={draft.retainLogsDays} onChange={(event) => update("retainLogsDays", Number(event.target.value))}><option value={7}>7 days</option><option value={30}>30 days</option><option value={90}>90 days</option></select></label></section>
      <section className="settings-group"><h2>Monitoring</h2><div className="number-row"><label>Check each source every<input type="number" min="30" max="86400" value={draft.probeIntervalSeconds} onChange={(event) => update("probeIntervalSeconds", Number(event.target.value))} /></label><span>seconds</span></div><div className="number-row"><label>Concurrent recordings<input type="number" min="1" max="16" value={draft.maxConcurrentRecordings} onChange={(event) => update("maxConcurrentRecordings", Number(event.target.value))} /></label><span>slots</span></div></section>
      <section className="settings-group"><h2>Windows startup</h2><label className="switch-row"><span><strong>Start with Windows</strong><small>Launch Live Downloader when you sign in.</small></span><input type="checkbox" checked={draft.startWithWindows} onChange={(event) => update("startWithWindows", event.target.checked)} /></label><label className="switch-row"><span><strong>Launch to tray</strong><small>Keep the dashboard hidden at sign-in while monitoring is ready.</small></span><input type="checkbox" checked={draft.launchToTray} onChange={(event) => update("launchToTray", event.target.checked)} /></label></section>
    </div>
    {state === "saved" && <p className="save-feedback"><Check size={16} />Settings were saved locally.</p>}
    {error && <p className="form-error" role="alert">{error}</p>}
  </section>;
}
