import { Check, FolderCog, Link2, Save, Unplug } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import type {
  AppSettings,
  Locale,
  TwitchDeviceAuthorization,
  TwitchStatus,
} from "../../../shared/contracts";
import { localeOptions, useI18n } from "../../../shared/i18n";
import { api, isDesktop } from "../../../shared/tauri/client";

interface SettingsPageProps {
  settings: AppSettings;
  onSave: (settings: AppSettings) => Promise<void>;
  onLocalePreview: (locale: Locale) => void;
}

export function SettingsPage({ settings, onSave, onLocalePreview }: SettingsPageProps) {
  const { translation: t } = useI18n();
  const [draft, setDraft] = useState(settings);
  const [state, setState] = useState<"idle" | "saving" | "saved" | "error">("idle");
  const [error, setError] = useState<string | null>(null);
  const [twitch, setTwitch] = useState<TwitchStatus | null>(null);
  const [twitchAuthorization, setTwitchAuthorization] = useState<TwitchDeviceAuthorization | null>(
    null,
  );
  const [twitchConnecting, setTwitchConnecting] = useState(false);
  const [twitchError, setTwitchError] = useState<string | null>(null);
  const twitchAttempt = useRef(0);

  useEffect(() => setDraft(settings), [settings]);
  useEffect(() => {
    if (!isDesktop) {
      setTwitch({ available: false, connected: false, login: null });
      return;
    }
    void api
      .twitchStatus()
      .then(setTwitch)
      .catch((reason) => setTwitchError(String(reason)));
    return () => {
      twitchAttempt.current += 1;
    };
  }, []);
  const update = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) =>
    setDraft((current) => ({ ...current, [key]: value }));

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

  async function connectTwitch() {
    const attempt = ++twitchAttempt.current;
    setTwitchConnecting(true);
    setTwitchError(null);
    try {
      const authorization = await api.startTwitchConnect();
      if (attempt !== twitchAttempt.current) return;
      setTwitchAuthorization(authorization);
      try {
        await api.openUrl(authorization.verificationUri);
      } catch (reason) {
        if (attempt === twitchAttempt.current)
          setTwitchError(reason instanceof Error ? reason.message : String(reason));
      }
      const deadline = Date.now() + authorization.expiresIn * 1000;
      while (Date.now() < deadline && attempt === twitchAttempt.current) {
        await new Promise((resolve) =>
          window.setTimeout(resolve, Math.max(authorization.interval, 1) * 1000),
        );
        const result = await api.pollTwitchConnect(authorization.deviceCode);
        if (result.connected) {
          setTwitch({ available: true, connected: true, login: result.login });
          setTwitchAuthorization(null);
          setTwitchError(null);
          return;
        }
      }
      if (attempt === twitchAttempt.current) setTwitchError(t.settings.twitchExpired);
    } catch (reason) {
      if (attempt === twitchAttempt.current)
        setTwitchError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      if (attempt === twitchAttempt.current) setTwitchConnecting(false);
    }
  }

  async function disconnectTwitch() {
    twitchAttempt.current += 1;
    setTwitchConnecting(false);
    setTwitchAuthorization(null);
    setTwitchError(null);
    try {
      await api.disconnectTwitch();
      setTwitch({ available: true, connected: false, login: null });
    } catch (reason) {
      setTwitchError(reason instanceof Error ? reason.message : String(reason));
    }
  }

  return (
    <section className="settings-panel">
      <div className="view-heading">
        <div>
          <h1>{t.settings.title}</h1>
          <p>{t.settings.description}</p>
        </div>
        <button
          type="button"
          className="primary-action"
          onClick={save}
          disabled={state === "saving"}
        >
          <Save size={16} />
          {state === "saving"
            ? t.common.saving
            : state === "saved"
              ? t.common.saved
              : t.common.saveChanges}
        </button>
      </div>
      <div className="settings-grid">
        <section className="settings-group">
          <h2>{t.settings.appearance}</h2>
          <label>
            {t.settings.language}
            <select
              value={draft.locale}
              onChange={(event) => {
                const locale = event.target.value as Locale;
                update("locale", locale);
                onLocalePreview(locale);
              }}
            >
              {localeOptions.map(({ value, nativeName }) => (
                <option key={value} value={value}>
                  {nativeName}
                </option>
              ))}
            </select>
          </label>
          <label>
            {t.settings.theme}
            <select
              value={draft.theme}
              onChange={(event) => update("theme", event.target.value as AppSettings["theme"])}
            >
              <option value="system">{t.settings.useWindowsSetting}</option>
              <option value="dark">{t.settings.dark}</option>
              <option value="light">{t.settings.light}</option>
            </select>
          </label>
          <label className="switch-row">
            <span>
              <strong>{t.settings.notifications}</strong>
              <small>{t.settings.notificationsDescription}</small>
            </span>
            <input
              type="checkbox"
              checked={draft.notificationsEnabled}
              onChange={(event) => update("notificationsEnabled", event.target.checked)}
            />
          </label>
        </section>
        <section className="settings-group twitch-settings">
          <h2>{t.settings.twitch}</h2>
          <p>{t.settings.twitchDescription}</p>
          <div className="twitch-connection-state">
            <span className={twitch?.connected ? "connected" : ""} aria-hidden="true" />
            <strong>
              {twitch?.connected
                ? t.settings.twitchConnectedAs(twitch.login ?? "Twitch")
                : twitch?.available === false
                  ? t.settings.twitchUnavailable
                  : t.settings.twitchDisconnected}
            </strong>
          </div>
          {twitchAuthorization && (
            <div className="twitch-activation" role="status">
              <span>{t.settings.activationCode}</span>
              <strong>{twitchAuthorization.userCode}</strong>
              <small>{t.settings.waitingForTwitch}</small>
            </div>
          )}
          {twitch?.connected ? (
            <button type="button" className="secondary-action" onClick={disconnectTwitch}>
              <Unplug size={16} />
              {t.settings.disconnectTwitch}
            </button>
          ) : (
            <button
              type="button"
              className="secondary-action"
              onClick={connectTwitch}
              disabled={!twitch?.available || twitchConnecting}
            >
              <Link2 size={16} />
              {twitchConnecting ? t.settings.waitingForTwitch : t.settings.connectTwitch}
            </button>
          )}
          {twitchError && (
            <p className="form-error" role="alert">
              {twitchError}
            </p>
          )}
        </section>
        <section className="settings-group">
          <h2>{t.settings.recordingLibrary}</h2>
          <label>
            {t.settings.downloadDirectory}
            <div className="input-icon">
              <FolderCog size={17} />
              <input
                value={draft.downloadDirectory}
                onChange={(event) => update("downloadDirectory", event.target.value)}
              />
            </div>
          </label>
        </section>
        <section className="settings-group">
          <h2>{t.settings.monitoring}</h2>
          <div className="number-row">
            <label>
              {t.settings.checkEvery}
              <input
                type="number"
                min="30"
                max="86400"
                value={draft.probeIntervalSeconds}
                onChange={(event) => update("probeIntervalSeconds", Number(event.target.value))}
              />
            </label>
            <span>{t.settings.seconds}</span>
          </div>
          <div className="number-row">
            <label>
              {t.settings.concurrentRecordings}
              <input
                type="number"
                min="1"
                max="16"
                value={draft.maxConcurrentRecordings}
                onChange={(event) => update("maxConcurrentRecordings", Number(event.target.value))}
              />
            </label>
            <span>{t.settings.slots}</span>
          </div>
        </section>
        <section className="settings-group">
          <h2>{t.settings.windowsStartup}</h2>
          <label className="switch-row">
            <span>
              <strong>{t.settings.startWithWindows}</strong>
              <small>{t.settings.startWithWindowsDescription}</small>
            </span>
            <input
              type="checkbox"
              checked={draft.startWithWindows}
              onChange={(event) => update("startWithWindows", event.target.checked)}
            />
          </label>
        </section>
      </div>
      {state === "saved" && (
        <p className="save-feedback">
          <Check size={16} />
          {t.settings.savedLocally}
        </p>
      )}
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}
