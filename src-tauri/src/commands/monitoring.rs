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
    let mut target = state
        .database
        .target(&id)?
        .ok_or_else(|| "That stream no longer exists.".to_owned())?;
    if let Ok(Some(metadata)) = state.twitch.metadata_for_url(&target.url).await {
        target.provider_user_id = Some(metadata.provider_user_id);
        target.avatar_url = Some(metadata.avatar_url);
        state.database.update_target(&target)?;
    }
    state.engine.check_now(id).await
}

#[tauri::command]
pub async fn stop_recording(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.engine.stop_job(job_id).await
}
