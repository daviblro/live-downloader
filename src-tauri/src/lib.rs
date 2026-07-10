mod database;
mod engine;
mod legacy;
mod models;

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use database::Database;
use engine::RecordingEngine;
use legacy::{find_legacy_config, import_legacy_config, is_http_url};
use models::{
    AppSettings, BootstrapPayload, CreateTargetInput, EngineSummary, LegacyImportResult,
    RecordingJob, UpdateTargetInput, WatchTarget,
};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, State, WindowEvent,
};

pub struct AppState {
    database: Arc<Database>,
    engine: RecordingEngine,
    is_quitting: AtomicBool,
}

#[tauri::command]
async fn bootstrap(state: State<'_, AppState>) -> Result<BootstrapPayload, String> {
    Ok(BootstrapPayload {
        settings: state.database.settings()?,
        targets: state.database.list_targets()?,
        jobs: state.database.list_jobs(60)?,
        engine: state.engine.summary()?,
        legacy_config_available: find_legacy_config().is_some(),
    })
}

#[tauri::command]
async fn add_target(
    input: CreateTargetInput,
    state: State<'_, AppState>,
) -> Result<WatchTarget, String> {
    let name = input.name.trim();
    let url = input.url.trim();
    if name.is_empty() {
        return Err("Give the stream a short, recognizable name.".to_owned());
    }
    if !is_http_url(url) {
        return Err("Enter an absolute HTTP or HTTPS stream URL.".to_owned());
    }
    let target = state.database.insert_target(name, url)?;
    let _ = state.engine.start().await;
    Ok(target)
}

#[tauri::command]
async fn update_target(input: UpdateTargetInput, state: State<'_, AppState>) -> Result<(), String> {
    let mut target = state
        .database
        .target(&input.id)?
        .ok_or_else(|| "That stream no longer exists.".to_owned())?;
    if input.name.trim().is_empty() || !is_http_url(input.url.trim()) {
        return Err("Provide a name and an absolute HTTP or HTTPS stream URL.".to_owned());
    }
    target.name = input.name.trim().to_owned();
    target.url = input.url.trim().to_owned();
    target.enabled = input.enabled;
    if !target.enabled {
        state.engine.cancel_target(&target.id);
    }
    state.database.update_target(&target)
}

#[tauri::command]
async fn remove_target(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.engine.cancel_target(&id);
    state.database.remove_target(&id)
}

#[tauri::command]
async fn start_engine(state: State<'_, AppState>) -> Result<EngineSummary, String> {
    state.engine.start().await
}

#[tauri::command]
async fn pause_all(state: State<'_, AppState>) -> Result<EngineSummary, String> {
    state.engine.pause_all().await
}

#[tauri::command]
async fn check_target_now(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.engine.check_now(id).await
}

#[tauri::command]
async fn stop_recording(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.engine.stop_job(job_id).await
}

#[tauri::command]
async fn save_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    if !matches!(settings.locale.as_str(), "en" | "pt-BR") {
        return Err("Choose a supported application language.".to_owned());
    }
    let directory = PathBuf::from(&settings.download_directory);
    if settings.download_directory.trim().is_empty() {
        return Err("Choose a download directory.".to_owned());
    }
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create the download directory: {error}"))?;
    state.database.save_settings(&settings)?;
    let _ = state
        .engine
        .summary()
        .and_then(|summary| state.engine.database().settings().map(|_| summary));
    Ok(settings)
}

#[tauri::command]
async fn list_history(state: State<'_, AppState>) -> Result<Vec<RecordingJob>, String> {
    state.database.list_jobs(200)
}

#[tauri::command]
async fn import_legacy(state: State<'_, AppState>) -> Result<LegacyImportResult, String> {
    let path = find_legacy_config().ok_or_else(|| {
        "No legacy config.json was found in the current or parent folder.".to_owned()
    })?;
    let result = import_legacy_config(&state.database, &path)?;
    let _ = state.engine.start().await;
    Ok(result)
}

#[tauri::command]
async fn open_download_directory(state: State<'_, AppState>) -> Result<(), String> {
    let settings = state.database.settings()?;
    let directory = PathBuf::from(settings.download_directory);
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    std::process::Command::new("explorer.exe")
        .arg(directory)
        .spawn()
        .map_err(|error| format!("Could not open the download directory: {error}"))?;
    Ok(())
}

#[tauri::command]
async fn reveal_recording(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let job = state
        .database
        .job(&job_id)?
        .ok_or_else(|| "That recording no longer exists.".to_owned())?;
    let path = job
        .output_path
        .ok_or_else(|| "The recording file has not been located yet.".to_owned())?;
    std::process::Command::new("explorer.exe")
        .args(["/select,", &path])
        .spawn()
        .map_err(|error| format!("Could not reveal the recording: {error}"))?;
    Ok(())
}

fn tray_icon() -> Image<'static> {
    let mut rgba = vec![0u8; 32 * 32 * 4];
    for y in 0..32 {
        for x in 0..32 {
            let index = ((y * 32 + x) * 4) as usize;
            let inside = (x as i32 - 16).pow(2) + (y as i32 - 16).pow(2) < 190;
            if inside {
                rgba[index] = 31;
                rgba[index + 1] = 122;
                rgba[index + 2] = 255;
                rgba[index + 3] = 255;
            }
        }
    }
    Image::new_owned(rgba, 32, 32)
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show dashboard", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pause all recordings", true, None::<&str>)?;
    let resume = MenuItem::with_id(app, "resume", "Resume monitoring", true, None::<&str>)?;
    let downloads = MenuItem::with_id(app, "downloads", "Open downloads", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Exit Live Downloader", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[&show, &pause, &resume, &downloads, &separator, &quit],
    )?;

    TrayIconBuilder::with_id("main-tray")
        .icon(tray_icon())
        .tooltip("Live Downloader")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "pause" => {
                let engine = app.state::<AppState>().engine.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = engine.pause_all().await;
                });
            }
            "resume" => {
                let engine = app.state::<AppState>().engine.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = engine.start().await;
                });
            }
            "downloads" => {
                let database = app.state::<AppState>().database.clone();
                tauri::async_runtime::spawn(async move {
                    if let Ok(settings) = database.settings() {
                        let _ = std::process::Command::new("explorer.exe")
                            .arg(settings.download_directory)
                            .spawn();
                    }
                });
            }
            "quit" => {
                app.state::<AppState>()
                    .is_quitting
                    .store(true, Ordering::Relaxed);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(event, TrayIconEvent::Click { .. }) {
                show_main_window(&tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn application_data_directory() -> Result<PathBuf, String> {
    directories::ProjectDirs::from("app", "Live Downloader", "Live Downloader")
        .map(|directories| directories.data_local_dir().to_path_buf())
        .ok_or_else(|| "Windows application data directory is unavailable.".to_owned())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let start_in_background = std::env::args().any(|argument| argument == "--background");
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            show_main_window(app)
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let data_directory =
                application_data_directory().map_err(|error| std::io::Error::other(error))?;
            let database = Arc::new(
                Database::open(&data_directory.join("live-downloader.db"))
                    .map_err(std::io::Error::other)?,
            );
            let engine = RecordingEngine::new(app.handle().clone(), database.clone());
            app.manage(AppState {
                database,
                engine,
                is_quitting: AtomicBool::new(false),
            });
            build_tray(app)?;
            if !start_in_background {
                show_main_window(app.handle());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            add_target,
            update_target,
            remove_target,
            start_engine,
            pause_all,
            check_target_now,
            stop_recording,
            save_settings,
            list_history,
            import_legacy,
            open_download_directory,
            reveal_recording
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                if !state.is_quitting.load(Ordering::Relaxed) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        });

    builder
        .run(tauri::generate_context!())
        .expect("error while running Live Downloader");
}
