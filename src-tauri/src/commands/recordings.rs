use std::path::{Path, PathBuf};

use tauri::State;

use crate::{
    app::AppState,
    domain::RecordingJob,
    platform::shell::{open_directory, reveal_file},
};

#[tauri::command]
pub async fn list_history(
    limit: Option<usize>,
    offset: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<RecordingJob>, String> {
    let settings = state.database.settings()?;
    let _ = state
        .engine
        .reconcile_output_paths(Path::new(&settings.download_directory));
    state
        .database
        .list_jobs_page(limit.unwrap_or(50).clamp(1, 200), offset.unwrap_or(0))
}

#[tauri::command]
pub async fn clear_history(state: State<'_, AppState>) -> Result<usize, String> {
    state.database.clear_history()
}

#[tauri::command]
pub async fn open_download_directory(state: State<'_, AppState>) -> Result<(), String> {
    let settings = state.database.settings()?;
    open_directory(Path::new(&settings.download_directory))
}

#[tauri::command]
pub async fn reveal_recording(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut job = state
        .database
        .job(&job_id)?
        .ok_or_else(|| "That recording no longer exists.".to_owned())?;
    if job.output_path.is_none() {
        let settings = state.database.settings()?;
        let _ = state
            .engine
            .reconcile_output_paths(Path::new(&settings.download_directory));
        job = state
            .database
            .job(&job_id)?
            .ok_or_else(|| "That recording no longer exists.".to_owned())?;
    }
    let path = job
        .output_path
        .ok_or_else(|| "The recording file has not been located yet.".to_owned())?;
    reveal_file(&PathBuf::from(path))
}
