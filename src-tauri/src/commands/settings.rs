use std::path::PathBuf;

use tauri::State;
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;

use crate::{app::AppState, domain::AppSettings};

#[tauri::command]
pub async fn update_settings(
    settings: AppSettings,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    if settings.download_directory.trim().is_empty() {
        return Err("Choose a download directory.".to_owned());
    }
    if !(30..=86_400).contains(&settings.probe_interval_seconds) {
        return Err("Check the streams every 30 to 86400 seconds.".to_owned());
    }
    if !(1..=16).contains(&settings.max_concurrent_recordings) {
        return Err("Allow between 1 and 16 concurrent recordings.".to_owned());
    }
    let previous = state.database.settings()?;
    std::fs::create_dir_all(PathBuf::from(&settings.download_directory))
        .map_err(|error| format!("Could not create the download directory: {error}"))?;
    let autolaunch = app.autolaunch();
    let was_enabled = autolaunch.is_enabled().map_err(|error| error.to_string())?;
    if settings.start_with_windows != was_enabled {
        if settings.start_with_windows {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        }
        .map_err(|error| error.to_string())?;
    }
    if let Err(error) = state.database.save_settings(&settings) {
        if settings.start_with_windows != was_enabled {
            let _ = if was_enabled {
                autolaunch.enable()
            } else {
                autolaunch.disable()
            };
        }
        return Err(error);
    }
    state
        .engine
        .settings_changed(previous.probe_interval_seconds != settings.probe_interval_seconds);
    Ok(settings)
}
