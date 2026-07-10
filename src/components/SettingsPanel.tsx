import { Check, FolderCog, Save } from "lucide-react";
import { useEffect, useState } from "react";
import { localeOptions, useI18n } from "../i18n";
import type { AppSettings, Locale } from "../types";

interface SettingsPanelProps {
  settings: AppSettings;
  onSave: (settings: AppSettings) => Promise<void>;
  onLocalePreview: (locale: Locale) => void;
}

export function SettingsPanel({ settings, onSave, onLocalePreview }: SettingsPanelProps) {
  const { translation: t } = useI18n();
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
    <div className="view-heading"><div><h1>{t.settings.title}</h1><p>{t.settings.description}</p></div><button type="button" className="primary-action" onClick={save} disabled={state === "saving"}><Save size={16} />{state === "saving" ? t.common.saving : state === "saved" ? t.common.saved : t.common.saveChanges}</button></div>
    <div className="settings-grid">
      <section className="settings-group"><h2>{t.settings.appearance}</h2><label>{t.settings.language}<select value={draft.locale} onChange={(event) => { const locale = event.target.value as Locale; update("locale", locale); onLocalePreview(locale); }}>{localeOptions.map(({ value, nativeName }) => <option key={value} value={value}>{nativeName}</option>)}</select></label><label>{t.settings.theme}<select value={draft.theme} onChange={(event) => update("theme", event.target.value as AppSettings["theme"])}><option value="system">{t.settings.useWindowsSetting}</option><option value="dark">{t.settings.dark}</option><option value="light">{t.settings.light}</option></select></label><label className="switch-row"><span><strong>{t.settings.notifications}</strong><small>{t.settings.notificationsDescription}</small></span><input type="checkbox" checked={draft.notificationsEnabled} onChange={(event) => update("notificationsEnabled", event.target.checked)} /></label></section>
      <section className="settings-group"><h2>{t.settings.recordingLibrary}</h2><label>{t.settings.downloadDirectory}<div className="input-icon"><FolderCog size={17} /><input value={draft.downloadDirectory} onChange={(event) => update("downloadDirectory", event.target.value)} /></div></label><label>{t.settings.keepDiagnostics}<select value={draft.retainLogsDays} onChange={(event) => update("retainLogsDays", Number(event.target.value))}>{[7, 30, 90].map((days) => <option key={days} value={days}>{t.settings.days(days)}</option>)}</select></label></section>
      <section className="settings-group"><h2>{t.settings.monitoring}</h2><div className="number-row"><label>{t.settings.checkEvery}<input type="number" min="30" max="86400" value={draft.probeIntervalSeconds} onChange={(event) => update("probeIntervalSeconds", Number(event.target.value))} /></label><span>{t.settings.seconds}</span></div><div className="number-row"><label>{t.settings.concurrentRecordings}<input type="number" min="1" max="16" value={draft.maxConcurrentRecordings} onChange={(event) => update("maxConcurrentRecordings", Number(event.target.value))} /></label><span>{t.settings.slots}</span></div></section>
      <section className="settings-group"><h2>{t.settings.windowsStartup}</h2><label className="switch-row"><span><strong>{t.settings.startWithWindows}</strong><small>{t.settings.startWithWindowsDescription}</small></span><input type="checkbox" checked={draft.startWithWindows} onChange={(event) => update("startWithWindows", event.target.checked)} /></label><label className="switch-row"><span><strong>{t.settings.launchToTray}</strong><small>{t.settings.launchToTrayDescription}</small></span><input type="checkbox" checked={draft.launchToTray} onChange={(event) => update("launchToTray", event.target.checked)} /></label></section>
    </div>
    {state === "saved" && <p className="save-feedback"><Check size={16} />{t.settings.savedLocally}</p>}
    {error && <p className="form-error" role="alert">{error}</p>}
  </section>;
}
