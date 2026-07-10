import { createContext, createElement, useContext, useMemo, type ReactNode } from "react";
import type { Locale } from "./types";

export const localeOptions: ReadonlyArray<{ value: Locale; nativeName: string }> = [
  { value: "en", nativeName: "English" },
  { value: "pt-BR", nativeName: "Português (Brasil)" },
];

export const localeTag: Record<Locale, string> = {
  en: "en-US",
  "pt-BR": "pt-BR",
};

export interface Translation {
  nav: Record<"overview" | "watchList" | "history" | "settings", string>;
  sidebar: Record<"primaryNavigation" | "diskHealth" | "good" | "libraryReady" | "recordingPath", string>;
  overview: {
    serviceReady: string;
    monitoringPaused: string;
    recordings: (count: number) => string;
    watching: (count: number) => string;
    importLegacyTitle: string;
    importLegacyBody: string;
    activeRecordings: string;
    noActiveRecordings: string;
    nextGlobalCheck: (value: string) => string;
    pauseAll: string;
    resume: string;
    serviceRunning: string;
    servicePaused: string;
    authorisedOnly: string;
    notScheduled: string;
    starting: string;
  };
  common: {
    addStream: string;
    downloads: string;
    openDownloads: string;
    actions: string;
    source: string;
    state: string;
    cancel: string;
    saveChanges: string;
    saving: string;
    saved: string;
    pause: string;
    resume: string;
    remove: string;
    switchView: string;
    checkNow: (name: string) => string;
    pauseSource: (name: string) => string;
    resumeSource: (name: string) => string;
    removeSource: (name: string) => string;
  };
  list: {
    title: string;
    description: string;
    filterPlaceholder: string;
    sources: (count: number) => string;
  };
  history: {
    title: string;
    description: string;
    revealFile: string;
    noFileYet: string;
  };
  settings: {
    title: string;
    description: string;
    appearance: string;
    language: string;
    theme: string;
    useWindowsSetting: string;
    dark: string;
    light: string;
    notifications: string;
    notificationsDescription: string;
    recordingLibrary: string;
    downloadDirectory: string;
    keepDiagnostics: string;
    days: (count: number) => string;
    monitoring: string;
    checkEvery: string;
    seconds: string;
    concurrentRecordings: string;
    slots: string;
    windowsStartup: string;
    startWithWindows: string;
    startWithWindowsDescription: string;
    launchToTray: string;
    launchToTrayDescription: string;
    savedLocally: string;
  };
  dialog: {
    title: string;
    description: string;
    close: string;
    sourceName: string;
    sourceNamePlaceholder: string;
    streamUrl: string;
    adding: string;
  };
  inspector: {
    selectSource: string;
    recordingActivity: string;
    active: string;
    healthyRecordingActivity: string;
    healthySignal: string;
    monitorStandingBy: string;
    currentFile: string;
    preparingFile: string;
    process: string;
    noActiveProcess: string;
    status: string;
    pauseMonitoring: string;
    stopRecording: string;
    openRecordingFolder: string;
  };
  table: {
    emptyTitle: string;
    emptyDescription: string;
    nextCheck: string;
    lastRecording: string;
  };
  release: {
    available: (version: string) => string;
    description: string;
    viewRelease: string;
    dismiss: string;
  };
  toast: {
    notifications: string;
    close: string;
    settingsSaved: string;
    downloadsOpened: string;
    legacyImported: string;
    monitoringPaused: string;
    monitoringResumed: string;
    checking: (name: string) => string;
    recordingStopping: string;
    recordingStopped: string;
    recordingStartedBackground: string;
    removed: (name: string) => string;
  };
  states: Record<string, string>;
  runtime: Record<string, string>;
}

const english: Translation = {
  nav: { overview: "Overview", watchList: "Watch list", history: "History", settings: "Settings" },
  sidebar: { primaryNavigation: "Primary navigation", diskHealth: "Disk health", good: "Good", libraryReady: "Library is ready for recordings", recordingPath: "Recording path" },
  overview: {
    serviceReady: "Recording service is ready", monitoringPaused: "Monitoring is paused",
    recordings: (count) => `${count} recording${count === 1 ? "" : "s"}`,
    watching: (count) => `${count} watching`, importLegacyTitle: "Import your PowerShell watch list",
    importLegacyBody: "We found a legacy config.json in this workspace.", activeRecordings: "Active recordings",
    noActiveRecordings: "No recordings are active. Start monitoring to begin.", nextGlobalCheck: (value) => `Next global check: ${value}`,
    pauseAll: "Pause all", resume: "Resume", serviceRunning: "Service running", servicePaused: "Service paused",
    authorisedOnly: "Authorised streams only", notScheduled: "Not scheduled", starting: "Starting Live Downloader…",
  },
  common: {
    addStream: "Add stream", downloads: "Downloads", openDownloads: "Open downloads", actions: "Actions", source: "Source", state: "State",
    cancel: "Cancel", saveChanges: "Save changes", saving: "Saving…", saved: "Saved", pause: "Pause", resume: "Resume", remove: "Remove", switchView: "Switch view",
    checkNow: (name) => `Check ${name} now`, pauseSource: (name) => `Pause ${name}`, resumeSource: (name) => `Resume ${name}`, removeSource: (name) => `Remove ${name}`,
  },
  list: { title: "Watch list", description: "Monitor, pause, and check every source from one place.", filterPlaceholder: "Filter by source, URL, or state", sources: (count) => `${count} source${count === 1 ? "" : "s"}` },
  history: { title: "Recording history", description: "Recent completed, cancelled, and failed recording attempts.", revealFile: "Reveal file", noFileYet: "No file yet" },
  settings: {
    title: "Settings", description: "Control the local engine, recording library, appearance, and background behaviour.", appearance: "Appearance", language: "Language", theme: "Theme", useWindowsSetting: "Use Windows setting", dark: "Dark", light: "Light", notifications: "Notifications", notificationsDescription: "Tell me when recording starts, ends, or needs attention.", recordingLibrary: "Recording library", downloadDirectory: "Download directory", keepDiagnostics: "Keep diagnostics", days: (count) => `${count} days`, monitoring: "Monitoring", checkEvery: "Check each source every", seconds: "seconds", concurrentRecordings: "Concurrent recordings", slots: "slots", windowsStartup: "Windows startup", startWithWindows: "Start with Windows", startWithWindowsDescription: "Launch Live Downloader when you sign in.", launchToTray: "Launch to tray", launchToTrayDescription: "Keep the dashboard hidden at sign-in while monitoring is ready.", savedLocally: "Settings were saved locally.",
  },
  dialog: { title: "Add stream", description: "Live Downloader will validate the URL before it is watched.", close: "Close", sourceName: "Source name", sourceNamePlaceholder: "e.g. Northernlight", streamUrl: "Stream URL", adding: "Adding…" },
  inspector: { selectSource: "Select a source to inspect its recording activity.", recordingActivity: "Recording activity", active: "Active", healthyRecordingActivity: "Healthy recording activity", healthySignal: "Healthy signal", monitorStandingBy: "Monitor is standing by", currentFile: "Current file", preparingFile: "Preparing the recording file", process: "Process", noActiveProcess: "No active process", status: "Status", pauseMonitoring: "Pause monitoring", stopRecording: "Stop recording", openRecordingFolder: "Open recording folder" },
  table: { emptyTitle: "Your watch list is empty.", emptyDescription: "Add a public stream URL to begin monitoring.", nextCheck: "Next check", lastRecording: "Last recording" },
  release: { available: (version) => `Live Downloader ${version} is available`, description: "Download the latest installer from GitHub Releases when you are ready.", viewRelease: "View release", dismiss: "Dismiss update notice" },
  toast: { notifications: "Notifications", close: "Close notification", settingsSaved: "Settings saved.", downloadsOpened: "Downloads folder opened.", legacyImported: "Legacy watch list imported.", monitoringPaused: "Monitoring paused and active recordings are stopping.", monitoringResumed: "Monitoring resumed.", checking: (name) => `Checking ${name} now.`, recordingStopping: "Recording is stopping.", recordingStopped: "Recording stopped in preview.", recordingStartedBackground: "A stream started recording in the background.", removed: (name) => `${name} removed.` },
  states: { Recording: "Recording", Watching: "Watching", Checking: "Checking", Queued: "Queued", Retrying: "Retrying", "Needs attention": "Needs attention", Completed: "Completed", Failed: "Failed", Cancelled: "Cancelled" },
  runtime: { "Managed yt-dlp + FFmpeg sidecars": "Managed yt-dlp + FFmpeg tools", "Managed yt-dlp sidecar": "Managed yt-dlp tool", "External yt-dlp ready": "External yt-dlp ready", "External yt-dlp path is unavailable": "External yt-dlp path is unavailable", "Waiting for live stream": "Waiting for the stream to go live", "Recording in the background": "Recording in the background", "Recording started": "Recording started", "Recording completed": "Recording completed", "Preparing the recording file": "Preparing the recording file" },
};

const portugueseBrazil: Translation = {
  nav: { overview: "Visão geral", watchList: "Lista de monitoramento", history: "Histórico", settings: "Configurações" },
  sidebar: { primaryNavigation: "Navegação principal", diskHealth: "Saúde do disco", good: "Boa", libraryReady: "A biblioteca está pronta para gravações", recordingPath: "Pasta de gravações" },
  overview: {
    serviceReady: "O serviço de gravação está pronto", monitoringPaused: "O monitoramento está pausado",
    recordings: (count) => `${count} ${count === 1 ? "gravação" : "gravações"}`,
    watching: (count) => `${count} monitorado${count === 1 ? "" : "s"}`, importLegacyTitle: "Importar sua lista do PowerShell",
    importLegacyBody: "Encontramos um config.json legado neste espaço de trabalho.", activeRecordings: "Gravações ativas",
    noActiveRecordings: "Não há gravações ativas. Inicie o monitoramento para começar.", nextGlobalCheck: (value) => `Próxima verificação global: ${value}`,
    pauseAll: "Pausar tudo", resume: "Retomar", serviceRunning: "Serviço em execução", servicePaused: "Serviço pausado",
    authorisedOnly: "Somente streams autorizadas", notScheduled: "Não agendado", starting: "Iniciando o Live Downloader…",
  },
  common: {
    addStream: "Adicionar stream", downloads: "Downloads", openDownloads: "Abrir downloads", actions: "Ações", source: "Fonte", state: "Estado",
    cancel: "Cancelar", saveChanges: "Salvar alterações", saving: "Salvando…", saved: "Salvo", pause: "Pausar", resume: "Retomar", remove: "Remover", switchView: "Alternar visualização",
    checkNow: (name) => `Verificar ${name} agora`, pauseSource: (name) => `Pausar ${name}`, resumeSource: (name) => `Retomar ${name}`, removeSource: (name) => `Remover ${name}`,
  },
  list: { title: "Lista de monitoramento", description: "Monitore, pause e verifique todas as fontes em um só lugar.", filterPlaceholder: "Filtre por fonte, URL ou estado", sources: (count) => `${count} fonte${count === 1 ? "" : "s"}` },
  history: { title: "Histórico de gravações", description: "Tentativas recentes concluídas, canceladas e com falha.", revealFile: "Mostrar arquivo", noFileYet: "Ainda não há arquivo" },
  settings: {
    title: "Configurações", description: "Controle o mecanismo local, a biblioteca de gravações, a aparência e o funcionamento em segundo plano.", appearance: "Aparência", language: "Idioma", theme: "Tema", useWindowsSetting: "Usar configuração do Windows", dark: "Escuro", light: "Claro", notifications: "Notificações", notificationsDescription: "Avise quando uma gravação iniciar, terminar ou precisar de atenção.", recordingLibrary: "Biblioteca de gravações", downloadDirectory: "Pasta de downloads", keepDiagnostics: "Manter diagnósticos", days: (count) => `${count} dias`, monitoring: "Monitoramento", checkEvery: "Verificar cada fonte a cada", seconds: "segundos", concurrentRecordings: "Gravações simultâneas", slots: "vagas", windowsStartup: "Inicialização do Windows", startWithWindows: "Iniciar com o Windows", startWithWindowsDescription: "Inicie o Live Downloader ao entrar no Windows.", launchToTray: "Iniciar na bandeja", launchToTrayDescription: "Mantenha o painel oculto ao entrar no Windows enquanto o monitoramento está pronto.", savedLocally: "As configurações foram salvas localmente.",
  },
  dialog: { title: "Adicionar stream", description: "O Live Downloader validará a URL antes de monitorá-la.", close: "Fechar", sourceName: "Nome da fonte", sourceNamePlaceholder: "ex.: Northernlight", streamUrl: "URL da stream", adding: "Adicionando…" },
  inspector: { selectSource: "Selecione uma fonte para inspecionar a atividade de gravação.", recordingActivity: "Atividade de gravação", active: "Ativa", healthyRecordingActivity: "Atividade de gravação saudável", healthySignal: "Sinal saudável", monitorStandingBy: "O monitor está aguardando", currentFile: "Arquivo atual", preparingFile: "Preparando o arquivo de gravação", process: "Processo", noActiveProcess: "Nenhum processo ativo", status: "Status", pauseMonitoring: "Pausar monitoramento", stopRecording: "Parar gravação", openRecordingFolder: "Abrir pasta de gravações" },
  table: { emptyTitle: "Sua lista de monitoramento está vazia.", emptyDescription: "Adicione uma URL de stream pública para começar a monitorar.", nextCheck: "Próxima verificação", lastRecording: "Última gravação" },
  release: { available: (version) => "O Live Downloader " + version + " está disponível", description: "Baixe o instalador mais recente no GitHub Releases quando estiver pronto.", viewRelease: "Ver lançamento", dismiss: "Fechar aviso de atualização" },
  toast: { notifications: "Notificações", close: "Fechar notificação", settingsSaved: "Configurações salvas.", downloadsOpened: "Pasta de downloads aberta.", legacyImported: "Lista de monitoramento legada importada.", monitoringPaused: "Monitoramento pausado e as gravações ativas estão sendo interrompidas.", monitoringResumed: "Monitoramento retomado.", checking: (name) => `Verificando ${name} agora.`, recordingStopping: "A gravação está sendo interrompida.", recordingStopped: "Gravação interrompida na prévia.", recordingStartedBackground: "Uma stream começou a gravar em segundo plano.", removed: (name) => `${name} removida.` },
  states: { Recording: "Gravando", Watching: "Monitorando", Checking: "Verificando", Queued: "Na fila", Retrying: "Tentando novamente", "Needs attention": "Precisa de atenção", Completed: "Concluída", Failed: "Falhou", Cancelled: "Cancelada" },
  runtime: { "Managed yt-dlp + FFmpeg sidecars": "yt-dlp + FFmpeg gerenciados", "Managed yt-dlp sidecar": "yt-dlp gerenciado", "External yt-dlp ready": "yt-dlp externo pronto", "External yt-dlp path is unavailable": "O caminho do yt-dlp externo não está disponível", "Waiting for live stream": "Aguardando a stream entrar no ar", "Recording in the background": "Gravando em segundo plano", "Recording started": "Gravação iniciada", "Recording completed": "Gravação concluída", "Preparing the recording file": "Preparando o arquivo de gravação" },
};

export const translations: Record<Locale, Translation> = { en: english, "pt-BR": portugueseBrazil };

export function localizeRuntimeText(value: string, translation: Translation): string {
  return translation.runtime[value] ?? translation.states[value] ?? value;
}

interface I18nValue {
  locale: Locale;
  translation: Translation;
}

const I18nContext = createContext<I18nValue>({ locale: "en", translation: english });

export function I18nProvider({ locale, children }: { locale: Locale; children: ReactNode }) {
  const value = useMemo(() => ({ locale, translation: translations[locale] ?? english }), [locale]);
  return createElement(I18nContext.Provider, { value }, children);
}

export function useI18n(): I18nValue {
  return useContext(I18nContext);
}
