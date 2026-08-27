mod app;
mod commands;
mod domain;
mod legacy;
mod monitoring;
mod persistence;
mod platform;

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use app::{build_tray, show_main_window, AppState};
use monitoring::RecordingEngine;
use persistence::Database;
use tauri::{Manager, WindowEvent};

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
            let data_directory = application_data_directory().map_err(std::io::Error::other)?;
            let database = Arc::new(
                Database::open(&data_directory.join("live-downloader.db"))
                    .map_err(std::io::Error::other)?,
            );
            let engine = RecordingEngine::new(app.handle().clone(), database.clone());
            database
                .recover_interrupted_jobs()
                .map_err(std::io::Error::other)?;
            engine.start().map_err(std::io::Error::other)?;
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
            commands::bootstrap::bootstrap,
            commands::targets::add_target,
            commands::targets::update_target,
            commands::targets::remove_target,
            commands::monitoring::start_engine,
            commands::monitoring::pause_all,
            commands::monitoring::check_target_now,
            commands::monitoring::stop_recording,
            commands::settings::update_settings,
            commands::recordings::list_history,
            commands::recordings::clear_history,
            commands::legacy::import_legacy,
            commands::recordings::open_download_directory,
            commands::recordings::reveal_recording
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
