use tauri::State;

use crate::{
    app::AppState,
    twitch::{TwitchConnectResult, TwitchDeviceAuthorization, TwitchStatus},
};

#[tauri::command]
pub async fn twitch_status(state: State<'_, AppState>) -> Result<TwitchStatus, String> {
    state.twitch.status().await
}

#[tauri::command]
pub async fn start_twitch_connect(
    state: State<'_, AppState>,
) -> Result<TwitchDeviceAuthorization, String> {
    state.twitch.start_connect().await
}

#[tauri::command]
pub async fn poll_twitch_connect(
    device_code: String,
    state: State<'_, AppState>,
) -> Result<TwitchConnectResult, String> {
    state.twitch.poll_connect(&device_code).await
}

#[tauri::command]
pub fn disconnect_twitch(state: State<'_, AppState>) -> Result<(), String> {
    state.twitch.disconnect()
}
