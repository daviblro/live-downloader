import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { disable, enable } from "@tauri-apps/plugin-autostart";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import type { AppSettings, BootstrapPayload, CreateTargetInput, EngineSummary, RecordingJob, UpdateTargetInput, WatchTarget } from "../types";

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
  saveSettings: (settings: AppSettings) => invoke<AppSettings>("save_settings", { settings }),
  history: () => invoke<RecordingJob[]>("list_history"),
  importLegacy: () => invoke("import_legacy"),
  openDownloads: () => invoke<void>("open_download_directory"),
  revealRecording: (jobId: string) => invoke<void>("reveal_recording", { jobId }),
  listenEngine: (handler: (summary: EngineSummary) => void) => listen<EngineSummary>("engine://changed", (event) => handler(event.payload)),
  setAutostart: async (enabled: boolean) => enabled ? enable() : disable(),
  notify: async (title: string, body: string) => {
    let permitted = await isPermissionGranted();
    if (!permitted) permitted = (await requestPermission()) === "granted";
    if (permitted) sendNotification({ title, body });
  },
};
