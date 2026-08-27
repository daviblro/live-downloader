use tauri::State;

use crate::{app::AppState, domain::EngineSummary};

#[tauri::command]
pub async fn start_engine(state: State<'_, AppState>) -> Result<EngineSummary, String> {
    state.engine.start()
}

#[tauri::command]
pub async fn pause_all(state: State<'_, AppState>) -> Result<EngineSummary, String> {
    state.engine.pause_all().await
}

#[tauri::command]
pub async fn check_target_now(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.engine.check_now(id).await
}

#[tauri::command]
pub async fn stop_recording(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.engine.stop_job(job_id).await
}
