import { getVersion } from "@tauri-apps/api/app";
import { ArrowRight, CircleAlert, Download, ExternalLink, FolderOpen, Import, Menu, Pause, Play, Plus, Search, X } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { AddStreamDialog } from "./components/AddStreamDialog";
import { Inspector } from "./components/Inspector";
import { Sidebar, type View } from "./components/Sidebar";
import { SettingsPanel } from "./components/SettingsPanel";
import { Avatar, StatusDot } from "./components/StatusDot";
import { WatchTable } from "./components/WatchTable";
import { demoPayload } from "./data/demo";
import { I18nProvider, localeTag, localizeRuntimeText, translations, useI18n } from "./i18n";
import { api, isDesktop } from "./lib/desktop";
import { displayReleaseVersion, isNewerRelease, type GitHubRelease } from "./lib/releases";
import type { AppSettings, BootstrapPayload, Locale, RecordingJob, WatchTarget } from "./types";

const releasesApiUrl = "https://api.github.com/repos/daviblro/live-downloader/releases/latest";
const releasesPageUrl = "https://github.com/daviblro/live-downloader/releases";

type AvailableRelease = { version: string; url: string };

function ReleaseNotice({ release, onOpen, onDismiss }: { release: AvailableRelease; onOpen: () => void; onDismiss: () => void }) {
  const { translation: t } = useI18n();
  return <aside className="release-notice" role="status">
    <Download size={19} aria-hidden="true" />
    <div><strong>{t.release.available(displayReleaseVersion(release.version))}</strong><small>{t.release.description}</small></div>
    <div className="release-notice-actions"><button type="button" className="secondary-action" onClick={onOpen}>{t.release.viewRelease} <ExternalLink size={15} /></button><button type="button" className="icon-button" onClick={onDismiss} aria-label={t.release.dismiss}><X size={16} /></button></div>
  </aside>;
}

function Overview({ payload, selected, jobs, onAdd, onCheck, onOpenDownloads, onPauseAll, onResumeAll, onSelect, onStop }: {
  payload: BootstrapPayload; selected: WatchTarget | null; jobs: RecordingJob[]; onAdd: () => void; onCheck: (target: WatchTarget) => void; onOpenDownloads: () => void; onPauseAll: () => void; onResumeAll: () => void; onSelect: (target: WatchTarget) => void; onStop: (jobId: string) => void;
}) {
  const { locale, translation: t } = useI18n();
  const recordings = payload.targets.filter((target) => target.state === "Recording");
  const selectedJob = selected?.activeJobId ? jobs.find((job) => job.id === selected.activeJobId) ?? null : null;
  const formatTime = (value: string | null) => value ? new Intl.DateTimeFormat(localeTag[locale], { hour: "2-digit", minute: "2-digit" }).format(new Date(value)) : t.overview.notScheduled;
  return <section className="overview-view">
    <header className="topbar"><div><h1>{payload.engine.running ? t.overview.serviceReady : t.overview.monitoringPaused}</h1><p><span className="recording-count">{t.overview.recordings(payload.engine.activeRecordings)}</span> · {t.overview.watching(payload.engine.enabledTargets)}</p></div><div className="top-actions"><button type="button" className="secondary-action desktop-only" onClick={onOpenDownloads}><FolderOpen size={16} />{t.common.downloads}</button><button type="button" className="primary-action" onClick={onAdd}><Plus size={18} />{t.common.addStream}</button></div></header>
    {payload.legacyConfigAvailable && <button type="button" className="migration-callout" onClick={() => window.dispatchEvent(new Event("legacy-import"))}><Import size={17} /><span><strong>{t.overview.importLegacyTitle}</strong><small>{t.overview.importLegacyBody}</small></span><ArrowRight size={17} /></button>}
    <div className="activity-header"><h2>{t.overview.activeRecordings}</h2><span>{localizeRuntimeText(payload.engine.sidecarStatus, t)}</span></div>
    <div className="active-rail">{recordings.length ? recordings.map((target, index) => <button type="button" className={`recording-card ${selected?.id === target.id ? "active" : ""}`} onClick={() => onSelect(target)} key={target.id}><Avatar label={target.name} index={index} /><span><strong>{target.name}</strong><small>{new URL(target.url).hostname.replace("www.", "")}</small></span><StatusDot state="Recording" /></button>) : <div className="inactive-rail"><Play size={18} />{t.overview.noActiveRecordings}</div>}</div>
    <div className="dashboard-grid"><section className="watch-section"><div className="section-heading"><div><h2>{t.nav.watchList} <span>({payload.targets.length})</span></h2><p>{t.overview.nextGlobalCheck(formatTime(payload.engine.nextGlobalCheckAt))}</p></div><button type="button" className="quiet-action" onClick={payload.engine.running ? onPauseAll : onResumeAll}>{payload.engine.running ? <><Pause size={15} />{t.overview.pauseAll}</> : <><Play size={15} />{t.overview.resume}</>}</button></div><WatchTable targets={payload.targets} selectedId={selected?.id ?? null} onSelect={onSelect} onCheck={onCheck} onToggle={onCheck} onRemove={() => undefined} /></section><Inspector target={selected} job={selectedJob} onPause={onPauseAll} onStop={() => selectedJob && onStop(selectedJob.id)} onOpen={onOpenDownloads} /></div>
    <footer className="status-footer"><span><i className={payload.engine.running ? "online" : "offline"} />{payload.engine.running ? t.overview.serviceRunning : t.overview.servicePaused}</span><span>{t.overview.authorisedOnly}</span></footer>
  </section>;
}

export default function App() {
  const [payload, setPayload] = useState<BootstrapPayload>(demoPayload);
  const [view, setView] = useState<View>("overview");
  const [selectedId, setSelectedId] = useState<string | null>(demoPayload.targets[0]?.id ?? null);
  const [filter, setFilter] = useState("");
  const [showAdd, setShowAdd] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [loading, setLoading] = useState(isDesktop);
  const [availableRelease, setAvailableRelease] = useState<AvailableRelease | null>(null);
  const [localePreview, setLocalePreview] = useState<Locale | null>(null);
  const locale = localePreview ?? payload.settings.locale;
  const t = translations[locale];

  const refresh = useCallback(async () => {
    if (!isDesktop) return;
    const next = await api.bootstrap();
    setPayload(next);
    setSelectedId((current) => current && next.targets.some((target) => target.id === current) ? current : next.targets[0]?.id ?? null);
  }, []);

  useEffect(() => {
    void refresh().catch((reason) => setMessage(String(reason))).finally(() => setLoading(false));
    if (!isDesktop) return;
    let unlisten: (() => void) | undefined;
    void api.listenEngine((engine) => {
      setPayload((current) => {
        if (current.settings.notificationsEnabled && engine.activeRecordings > current.engine.activeRecordings) {
          void api.notify("Live Downloader", t.toast.recordingStartedBackground);
        }
        return { ...current, engine };
      });
      void refresh();
    }).then((dispose) => { unlisten = dispose; });
    return () => unlisten?.();
  }, [refresh, t.toast.recordingStartedBackground]);

  useEffect(() => {
    if (!isDesktop) return;
    let cancelled = false;
    void (async () => {
      const [installedVersion, response] = await Promise.all([getVersion(), fetch(releasesApiUrl, { headers: { Accept: "application/vnd.github+json" } })]);
      if (!response.ok) return;
      const release = await response.json() as GitHubRelease;
      if (cancelled || release.draft || release.prerelease || typeof release.tag_name !== "string" || typeof release.html_url !== "string") return;
      if (isNewerRelease(release.tag_name, installedVersion)) setAvailableRelease({ version: release.tag_name, url: release.html_url });
    })().catch(() => undefined);
    return () => { cancelled = true; };
  }, []);

  useEffect(() => {
    const importLegacy = () => void action(async () => { await api.importLegacy(); await refresh(); }, t.toast.legacyImported);
    window.addEventListener("legacy-import", importLegacy);
    return () => window.removeEventListener("legacy-import", importLegacy);
  }, [refresh, t.toast.legacyImported]);

  useEffect(() => {
    document.documentElement.dataset.theme = payload.settings.theme;
    document.documentElement.lang = locale;
  }, [locale, payload.settings.theme]);

  const selected = payload.targets.find((target) => target.id === selectedId) ?? null;
  const filteredTargets = useMemo(() => payload.targets.filter((target) => `${target.name} ${target.url} ${target.state}`.toLowerCase().includes(filter.toLowerCase())), [filter, payload.targets]);

  async function action(work: () => Promise<void>, success?: string) {
    try { await work(); if (success) setMessage(success); } catch (reason) { setMessage(reason instanceof Error ? reason.message : String(reason)); }
  }

  const updatePayload = (updates: Partial<BootstrapPayload>) => setPayload((current) => ({ ...current, ...updates }));
  const updateSettings = async (settings: AppSettings) => {
    setLocalePreview(settings.locale);
    await action(async () => {
      if (isDesktop) {
        await api.setAutostart(settings.startWithWindows);
        await api.saveSettings(settings);
        await refresh();
      } else updatePayload({ settings });
    }, t.toast.settingsSaved);
  };
  const addStream = async (input: { name: string; url: string }) => {
    if (isDesktop) { await api.addTarget(input); await refresh(); return; }
    const target: WatchTarget = { id: crypto.randomUUID(), name: input.name, url: input.url, enabled: true, state: "Watching", statusDetail: "Waiting for live stream", nextCheckAt: new Date(Date.now() + 300_000).toISOString(), lastCheckedAt: null, lastRecordingAt: null, activeJobId: null, createdAt: new Date().toISOString() };
    setPayload((current) => ({ ...current, targets: [...current.targets, target], engine: { ...current.engine, enabledTargets: current.engine.enabledTargets + 1 } }));
    setSelectedId(target.id);
  };
  const openDownloads = () => void action(async () => { if (isDesktop) await api.openDownloads(); }, t.toast.downloadsOpened);
  const openAvailableRelease = () => void action(async () => {
    const url = availableRelease?.url ?? releasesPageUrl;
    if (isDesktop) await api.openUrl(url);
    else window.open(url, "_blank", "noopener,noreferrer");
  });
  const pauseAll = () => void action(async () => { if (isDesktop) { await api.pauseAll(); await refresh(); } else updatePayload({ engine: { ...payload.engine, running: false } }); }, t.toast.monitoringPaused);
  const resumeAll = () => void action(async () => { if (isDesktop) { await api.startEngine(); await refresh(); } else updatePayload({ engine: { ...payload.engine, running: true } }); }, t.toast.monitoringResumed);
  const checkNow = (target: WatchTarget) => void action(async () => { if (isDesktop) { await api.checkNow(target.id); await refresh(); } else setMessage(t.toast.checking(target.name)); });
  const stopJob = (jobId: string) => void action(async () => { if (isDesktop) { await api.stopRecording(jobId); await refresh(); } else setMessage(t.toast.recordingStopped); }, t.toast.recordingStopping);
  const removeTarget = (target: WatchTarget) => void action(async () => { if (isDesktop) { await api.removeTarget(target.id); await refresh(); } else setPayload((current) => ({ ...current, targets: current.targets.filter((item) => item.id !== target.id) })); }, t.toast.removed(target.name));
  const toggleTarget = (target: WatchTarget) => void action(async () => { if (isDesktop) { await api.updateTarget({ ...target, enabled: !target.enabled }); await refresh(); } else setPayload((current) => ({ ...current, targets: current.targets.map((item) => item.id === target.id ? { ...item, enabled: !item.enabled, state: !item.enabled ? "Watching" : "Cancelled" } : item) })); });
  const previewLocale = (nextLocale: Locale) => { setLocalePreview(nextLocale); setMessage(null); };
  const navigate = (nextView: View) => { if (nextView !== "settings") setLocalePreview(null); setView(nextView); };

  return <I18nProvider locale={locale}><div className="app-shell">
    <Sidebar active={view} onNavigate={navigate} onOpenDownloads={openDownloads} downloadDirectory={payload.settings.downloadDirectory} />
    <main className="main-content">
      {loading && <div className="loading-layer">{t.overview.starting}</div>}
      {view === "overview" && <>{availableRelease && <ReleaseNotice release={availableRelease} onOpen={openAvailableRelease} onDismiss={() => setAvailableRelease(null)} />}<Overview payload={payload} selected={selected} jobs={payload.jobs} onAdd={() => setShowAdd(true)} onCheck={checkNow} onOpenDownloads={openDownloads} onPauseAll={pauseAll} onResumeAll={resumeAll} onSelect={(target) => setSelectedId(target.id)} onStop={stopJob} /></>}
      {view === "watch-list" && <section className="list-view"><header className="topbar"><div><h1>{t.list.title}</h1><p>{t.list.description}</p></div><button type="button" className="primary-action" onClick={() => setShowAdd(true)}><Plus size={18} />{t.common.addStream}</button></header><div className="list-toolbar"><label className="search-field"><Search size={17} /><input value={filter} onChange={(event) => setFilter(event.target.value)} placeholder={t.list.filterPlaceholder} /></label><span>{t.list.sources(filteredTargets.length)}</span></div><WatchTable targets={filteredTargets} selectedId={selectedId} onSelect={(target) => setSelectedId(target.id)} onCheck={checkNow} onToggle={toggleTarget} onRemove={removeTarget} /></section>}
      {view === "history" && <section className="history-view"><header className="topbar"><div><h1>{t.history.title}</h1><p>{t.history.description}</p></div><button type="button" className="secondary-action" onClick={openDownloads}><FolderOpen size={16} />{t.common.openDownloads}</button></header><div className="history-list">{payload.jobs.map((job) => <article key={job.id} className="history-row"><Avatar label={job.targetName} /><div className="history-title"><strong>{job.targetName}</strong><span>{new Date(job.startedAt).toLocaleString(localeTag[locale])}</span></div><StatusDot state={job.state} /><p>{localizeRuntimeText(job.message, t)}</p><button type="button" className="quiet-action" disabled={!job.outputPath} onClick={() => isDesktop && void api.revealRecording(job.id)}>{job.outputPath ? t.history.revealFile : t.history.noFileYet}</button></article>)}</div></section>}
      {view === "settings" && <SettingsPanel settings={payload.settings} onSave={updateSettings} onLocalePreview={previewLocale} />}
    </main>
    <button type="button" className="mobile-menu" onClick={() => navigate(view === "watch-list" ? "overview" : "watch-list")} aria-label={t.common.switchView}><Menu size={20} /></button>
    {message && <button type="button" className="toast" onClick={() => setMessage(null)}><CircleAlert size={17} />{message}</button>}
    {showAdd && <AddStreamDialog onClose={() => setShowAdd(false)} onSubmit={addStream} />}
  </div></I18nProvider>;
}
