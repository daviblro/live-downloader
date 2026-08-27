use std::path::Path;

use tauri::State;

use crate::{
    app::AppState, domain::BootstrapPayload, legacy::find_legacy_config,
    platform::disk::disk_usage_for,
};

#[tauri::command]
pub async fn bootstrap(state: State<'_, AppState>) -> Result<BootstrapPayload, String> {
    let settings = state.database.settings()?;
    let _ = state
        .engine
        .reconcile_output_paths(Path::new(&settings.download_directory));
    Ok(BootstrapPayload {
        disk_usage: disk_usage_for(Path::new(&settings.download_directory)),
        settings,
        targets: state.database.list_targets()?,
        jobs: state.database.list_jobs(20)?,
        engine: state.engine.summary()?,
        legacy_config_available: find_legacy_config().is_some(),
    })
}
