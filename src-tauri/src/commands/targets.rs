use tauri::State;

use crate::{
    app::AppState,
    domain::{CreateTargetInput, UpdateTargetInput, WatchTarget},
    legacy::is_http_url,
};

#[tauri::command]
pub async fn add_target(
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
    let metadata = state.twitch.metadata_for_url(url).await.ok().flatten();
    let target = state
        .database
        .insert_target_with_metadata(name, url, metadata.as_ref())?;
    let _ = state.engine.start();
    state.engine.check_soon(&target.id);
    Ok(target)
}

#[tauri::command]
pub async fn update_target(
    input: UpdateTargetInput,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut target = state
        .database
        .target(&input.id)?
        .ok_or_else(|| "That stream no longer exists.".to_owned())?;
    if input.name.trim().is_empty() || !is_http_url(input.url.trim()) {
        return Err("Provide a name and an absolute HTTP or HTTPS stream URL.".to_owned());
    }
    let url_changed = target.url != input.url.trim();
    let enabled_now = input.enabled && !target.enabled;
    target.name = input.name.trim().to_owned();
    target.url = input.url.trim().to_owned();
    target.enabled = input.enabled;
    if url_changed || target.avatar_url.is_none() {
        match state.twitch.metadata_for_url(&target.url).await {
            Ok(Some(metadata)) => {
                target.provider_user_id = Some(metadata.provider_user_id);
                target.avatar_url = Some(metadata.avatar_url);
            }
            Ok(None) | Err(_) if url_changed => {
                target.provider_user_id = None;
                target.avatar_url = None;
            }
            Ok(None) | Err(_) => {}
        }
    }
    if !target.enabled {
        state.engine.cancel_target(&target.id);
    }
    state.database.update_target(&target)?;
    if target.enabled && (enabled_now || url_changed) {
        state.engine.check_soon(&target.id);
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_target(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.engine.cancel_target(&id);
    state.database.remove_target(&id)
}
