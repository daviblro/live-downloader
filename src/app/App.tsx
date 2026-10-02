import { getVersion } from "@tauri-apps/api/app";
import { Menu } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { ToastContainer, toast, type Theme } from "react-toastify";
import { HelpPage } from "../features/help/components/HelpPage";
import { OverviewPage } from "../features/monitoring/components/OverviewPage";
import { HistoryPage } from "../features/recordings/components/HistoryPage";
import { SettingsPage } from "../features/settings/components/SettingsPage";
import { AvailableRelease, ReleaseNotice } from "../features/updates/components/ReleaseNotice";
import { AddStreamDialog } from "../features/watch-targets/components/AddStreamDialog";
import { WatchListPage } from "../features/watch-targets/components/WatchListPage";
import { isNewerRelease, type GitHubRelease } from "../features/updates/release";
import type { AppSettings, Locale, RecordingJob, WatchTarget } from "../shared/contracts";
import { I18nProvider, translations } from "../shared/i18n";
import { api, isDesktop } from "../shared/tauri/client";
import { Sidebar, type View } from "./layout/Sidebar";
import { useAppData } from "./useAppData";

const releasesApiUrl = "https://api.github.com/repos/daviblro/live-downloader/releases/latest";
const releasesPageUrl = "https://github.com/daviblro/live-downloader/releases";
const issuesPageUrl = "https://github.com/daviblro/live-downloader/issues/new/choose";

function useToastTheme(theme: AppSettings["theme"]): Theme {
  const [systemTheme, setSystemTheme] = useState<Theme>(() =>
    window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark",
  );
  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: light)");
    const updateTheme = () => setSystemTheme(mediaQuery.matches ? "light" : "dark");
    mediaQuery.addEventListener("change", updateTheme);
    return () => mediaQuery.removeEventListener("change", updateTheme);
  }, []);
  return theme === "system" ? systemTheme : theme;
}

export default function App() {
  const { payload, setPayload, updatePayload, loading, refresh } = useAppData();
  const [view, setView] = useState<View>("overview");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [filter, setFilter] = useState("");
  const [showAdd, setShowAdd] = useState(false);
  const [editingTarget, setEditingTarget] = useState<WatchTarget | null>(null);
  const [availableRelease, setAvailableRelease] = useState<AvailableRelease | null>(null);
  const [localePreview, setLocalePreview] = useState<Locale | null>(null);
  const [historyJobs, setHistoryJobs] = useState<RecordingJob[]>([]);
  const locale = localePreview ?? payload.settings.locale;
  const t = translations[locale];
  const toastTheme = useToastTheme(payload.settings.theme);

  useEffect(() => {
    setSelectedId((current) =>
      current && payload.targets.some((target) => target.id === current)
        ? current
        : (payload.targets[0]?.id ?? null),
    );
  }, [payload.targets]);

  useEffect(() => {
    if (!isDesktop) return;
    let cancelled = false;
    void (async () => {
      const [installedVersion, response] = await Promise.all([
        getVersion(),
        fetch(releasesApiUrl, { headers: { Accept: "application/vnd.github+json" } }),
      ]);
      if (!response.ok) return;
      const release = (await response.json()) as GitHubRelease;
      if (
        cancelled ||
        release.draft ||
        release.prerelease ||
        typeof release.tag_name !== "string" ||
        typeof release.html_url !== "string"
      )
        return;
      if (isNewerRelease(release.tag_name, installedVersion))
        setAvailableRelease({ version: release.tag_name, url: release.html_url });
    })().catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    document.documentElement.dataset.theme = payload.settings.theme;
    document.documentElement.lang = locale;
  }, [locale, payload.settings.theme]);

  useEffect(() => {
    if (view !== "history") return;
    if (!isDesktop) {
      setHistoryJobs(payload.jobs);
      return;
    }
    void api
      .history(200, 0)
      .then(setHistoryJobs)
      .catch((reason) => toast.error(String(reason)));
  }, [payload.jobs, view]);

  const selected = payload.targets.find((target) => target.id === selectedId) ?? null;
  const filteredTargets = useMemo(
    () =>
      payload.targets.filter((target) =>
        `${target.name} ${target.url} ${target.state}`.toLowerCase().includes(filter.toLowerCase()),
      ),
    [filter, payload.targets],
  );

  async function action(work: () => Promise<void>, success?: string) {
    try {
      await work();
      if (success) toast.success(success);
    } catch (reason) {
      toast.error(reason instanceof Error ? reason.message : String(reason), {
        autoClose: false,
        closeOnClick: false,
      });
    }
  }

  const updateSettings = async (settings: AppSettings) => {
    setLocalePreview(settings.locale);
    await action(async () => {
      if (isDesktop) {
        await api.updateSettings(settings);
        await refresh();
      } else updatePayload({ settings });
    }, t.toast.settingsSaved);
  };

  const addStream = async (input: { name: string; url: string }) => {
    if (isDesktop) {
      await api.addTarget(input);
      await refresh();
      return;
    }
    const target: WatchTarget = {
      id: crypto.randomUUID(),
      name: input.name,
      url: input.url,
      enabled: true,
      state: "Watching",
      statusDetail: "Waiting for live stream",
      nextCheckAt: new Date(Date.now() + 300_000).toISOString(),
      lastCheckedAt: null,
      lastRecordingAt: null,
      activeJobId: null,
      createdAt: new Date().toISOString(),
      providerUserId: null,
      avatarUrl: null,
    };
    setPayload((current) => ({
      ...current,
      targets: [...current.targets, target],
      engine: { ...current.engine, enabledTargets: current.engine.enabledTargets + 1 },
    }));
    setSelectedId(target.id);
  };

  const editStream = async (target: WatchTarget, input: { name: string; url: string }) => {
    if (isDesktop) {
      await api.updateTarget({ id: target.id, enabled: target.enabled, ...input });
      await refresh();
    } else {
      setPayload((current) => ({
        ...current,
        targets: current.targets.map((item) =>
          item.id === target.id ? { ...item, ...input } : item,
        ),
      }));
    }
    toast.success(t.toast.updated(input.name));
  };

  const openDownloads = () =>
    void action(async () => {
      if (isDesktop) await api.openDownloads();
    }, t.toast.downloadsOpened);
  const openAvailableRelease = () =>
    void action(async () => {
      const url = availableRelease?.url ?? releasesPageUrl;
      if (isDesktop) await api.openUrl(url);
      else window.open(url, "_blank", "noopener,noreferrer");
    });
  const pauseAll = () =>
    void action(async () => {
      if (isDesktop) {
        await api.pauseAll();
        await refresh();
      } else updatePayload({ engine: { ...payload.engine, running: false } });
    }, t.toast.monitoringPaused);
  const resumeAll = () =>
    void action(async () => {
      if (isDesktop) {
        await api.startEngine();
        await refresh();
      } else updatePayload({ engine: { ...payload.engine, running: true } });
    }, t.toast.monitoringResumed);
  const checkNow = (target: WatchTarget) =>
    void action(async () => {
      if (isDesktop) {
        await api.checkNow(target.id);
        await refresh();
      } else toast.info(t.toast.checking(target.name));
    });
  const stopJob = (jobId: string) =>
    void action(async () => {
      if (isDesktop) {
        await api.stopRecording(jobId);
        await refresh();
      }
    }, t.toast.recordingStopping);
  const clearHistory = () => {
    if (!window.confirm(t.history.clearConfirmation)) return;
    void action(async () => {
      if (isDesktop) {
        await api.clearHistory();
        await refresh();
        setHistoryJobs(await api.history(200, 0));
      } else
        setPayload((current) => ({
          ...current,
          jobs: current.jobs.filter((job) => job.state === "Recording"),
        }));
    }, t.toast.historyCleared);
  };
  const revealRecording = (jobId: string) =>
    void action(async () => {
      if (isDesktop) await api.revealRecording(jobId);
    });
  const removeTarget = (target: WatchTarget) => {
    const confirmation =
      target.state === "Recording"
        ? t.toast.removeRecordingConfirmation(target.name)
        : t.toast.removeConfirmation(target.name);
    if (!window.confirm(confirmation)) return;
    void action(async () => {
      if (isDesktop) {
        await api.removeTarget(target.id);
        await refresh();
      } else
        setPayload((current) => ({
          ...current,
          targets: current.targets.filter((item) => item.id !== target.id),
        }));
    }, t.toast.removed(target.name));
  };
  const toggleTarget = (target: WatchTarget) =>
    void action(async () => {
      if (isDesktop) {
        await api.updateTarget({ ...target, enabled: !target.enabled });
        await refresh();
      } else
        setPayload((current) => ({
          ...current,
          targets: current.targets.map((item) =>
            item.id === target.id
              ? { ...item, enabled: !item.enabled, state: !item.enabled ? "Watching" : "Disabled" }
              : item,
          ),
        }));
    });
  const importLegacy = () =>
    void action(async () => {
      await api.importLegacy();
      await refresh();
    }, t.toast.legacyImported);
  const navigate = (nextView: View) => {
    if (nextView !== "settings") setLocalePreview(null);
    setView(nextView);
  };

  return (
    <I18nProvider locale={locale}>
      <div className="app-shell">
        <Sidebar
          active={view}
          onNavigate={navigate}
          onOpenDownloads={openDownloads}
          downloadDirectory={payload.settings.downloadDirectory}
          diskUsage={payload.diskUsage}
        />
        <main className="main-content">
          {loading && <div className="loading-layer">{t.overview.starting}</div>}
          {view === "overview" && (
            <>
              {availableRelease && (
                <ReleaseNotice
                  release={availableRelease}
                  onOpen={openAvailableRelease}
                  onDismiss={() => setAvailableRelease(null)}
                />
              )}
              <OverviewPage
                payload={payload}
                selected={selected}
                jobs={payload.jobs}
                onAdd={() => setShowAdd(true)}
                onCheck={checkNow}
                onEdit={setEditingTarget}
                onImportLegacy={importLegacy}
                onOpenDownloads={openDownloads}
                onPauseAll={pauseAll}
                onRemove={removeTarget}
                onResumeAll={resumeAll}
                onSelect={(target) => setSelectedId(target.id)}
                onStop={stopJob}
                onToggle={toggleTarget}
              />
            </>
          )}
          {view === "watch-list" && (
            <WatchListPage
              targets={filteredTargets}
              selectedId={selectedId}
              filter={filter}
              onFilterChange={setFilter}
              onAdd={() => setShowAdd(true)}
              onSelect={(target) => setSelectedId(target.id)}
              onCheck={checkNow}
              onEdit={setEditingTarget}
              onToggle={toggleTarget}
              onRemove={removeTarget}
            />
          )}
          {view === "history" && (
            <HistoryPage
              jobs={historyJobs}
              onClear={clearHistory}
              onOpenDownloads={openDownloads}
              onReveal={revealRecording}
            />
          )}
          {view === "settings" && (
            <SettingsPage
              settings={payload.settings}
              onSave={updateSettings}
              onLocalePreview={setLocalePreview}
            />
          )}
          {view === "help" && (
            <HelpPage
              onOpenIssues={() =>
                void action(async () => {
                  if (isDesktop) await api.openUrl(issuesPageUrl);
                  else window.open(issuesPageUrl, "_blank", "noopener,noreferrer");
                })
              }
            />
          )}
        </main>
        <button
          type="button"
          className="mobile-menu"
          onClick={() => navigate(view === "watch-list" ? "overview" : "watch-list")}
          aria-label={t.common.switchView}
        >
          <Menu size={20} />
        </button>
        <ToastContainer
          aria-label={t.toast.notifications}
          autoClose={5_000}
          closeButton={({ closeToast }) => (
            <button
              type="button"
              className="toast-close-button"
              onClick={closeToast}
              aria-label={t.toast.close}
            >
              ×
            </button>
          )}
          closeOnClick
          hideProgressBar={false}
          newestOnTop
          pauseOnFocusLoss
          pauseOnHover
          position="bottom-right"
          theme={toastTheme}
        />
        {showAdd && <AddStreamDialog onClose={() => setShowAdd(false)} onSubmit={addStream} />}
        {editingTarget && (
          <AddStreamDialog
            target={editingTarget}
            onClose={() => setEditingTarget(null)}
            onSubmit={(input) => editStream(editingTarget, input)}
          />
        )}
      </div>
    </I18nProvider>
  );
}
