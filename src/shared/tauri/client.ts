import { Channel, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { openUrl } from "@tauri-apps/plugin-opener";
import type {
  AppSettings,
  BootstrapPayload,
  CreateTargetInput,
  EngineSummary,
  RecordingJob,
  TwitchConnectResult,
  TwitchDeviceAuthorization,
  TwitchStatus,
  UpdateCheck,
  UpdateProgress,
  UpdateTargetInput,
  WatchTarget,
} from "../contracts";
import { ENGINE_CHANGED_EVENT } from "./events";

export const isDesktop = "__TAURI_INTERNALS__" in window;

export const api = {
  bootstrap: () => invoke<BootstrapPayload>("bootstrap"),
  addTarget: (input: CreateTargetInput) => invoke<WatchTarget>("add_target", { input }),
  updateTarget: (input: UpdateTargetInput) => invoke<void>("update_target", { input }),
  removeTarget: (id: string) => invoke<void>("remove_target", { id }),
  startEngine: () => invoke<EngineSummary>("start_engine"),
  pauseAll: () => invoke<EngineSummary>("pause_all"),
  checkNow: (id: string) => invoke<void>("check_target_now", { id }),
  stopRecording: (jobId: string) => invoke<void>("stop_recording", { jobId }),
  updateSettings: (settings: AppSettings) => invoke<AppSettings>("update_settings", { settings }),
  twitchStatus: () => invoke<TwitchStatus>("twitch_status"),
  startTwitchConnect: () => invoke<TwitchDeviceAuthorization>("start_twitch_connect"),
  pollTwitchConnect: (deviceCode: string) =>
    invoke<TwitchConnectResult>("poll_twitch_connect", { deviceCode }),
  disconnectTwitch: () => invoke<void>("disconnect_twitch"),
  history: (limit = 50, offset = 0) => invoke<RecordingJob[]>("list_history", { limit, offset }),
  clearHistory: () => invoke<number>("clear_history"),
  importLegacy: () => invoke("import_legacy"),
  openDownloads: () => invoke<void>("open_download_directory"),
  openUrl: (url: string) => openUrl(url),
  revealRecording: (jobId: string) => invoke<void>("reveal_recording", { jobId }),
  checkForUpdate: () => invoke<UpdateCheck>("check_for_update"),
  installUpdate: (onProgress: (progress: UpdateProgress) => void) => {
    const onEvent = new Channel<UpdateProgress>();
    onEvent.onmessage = onProgress;
    return invoke<void>("install_update", { onEvent });
  },
  listenEngine: (handler: (summary: EngineSummary) => void) =>
    listen<EngineSummary>(ENGINE_CHANGED_EVENT, (event) => handler(event.payload)),
  notify: async (title: string, body: string) => {
    let permitted = await isPermissionGranted();
    if (!permitted) permitted = (await requestPermission()) === "granted";
    if (permitted) sendNotification({ title, body });
  },
};
