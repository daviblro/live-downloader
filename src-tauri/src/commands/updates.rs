use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, State};
use tauri_plugin_updater::{Updater, UpdaterExt};

use crate::app::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    pub configured: bool,
    pub update: Option<AvailableUpdate>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableUpdate {
    pub version: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(
    tag = "event",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum UpdateProgress {
    Started { content_length: Option<u64> },
    Progress { chunk_length: usize },
    Installing,
}

/// Release builds embed the updater public key through a generated config
/// overlay (see `scripts/write-updater-config.mjs`). Builds without it keep the
/// manual GitHub Releases notice.
fn updater_configured(app: &AppHandle) -> bool {
    app.config()
        .plugins
        .0
        .get("updater")
        .and_then(|updater| updater.get("pubkey"))
        .and_then(|pubkey| pubkey.as_str())
        .is_some_and(|pubkey| !pubkey.trim().is_empty())
}

fn updater(app: &AppHandle) -> Result<Option<Updater>, String> {
    if !updater_configured(app) {
        return Ok(None);
    }
    app.updater_builder()
        .build()
        .map(Some)
        .map_err(|error| format!("Could not prepare the updater: {error}"))
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<UpdateCheck, String> {
    let Some(updater) = updater(&app)? else {
        return Ok(UpdateCheck {
            configured: false,
            update: None,
        });
    };
    let update = updater
        .check()
        .await
        .map_err(|error| format!("Could not check for updates: {error}"))?;
    Ok(UpdateCheck {
        configured: true,
        update: update.map(|update| AvailableUpdate {
            version: update.version,
            notes: update.body,
        }),
    })
}

#[tauri::command]
pub async fn install_update(
    app: AppHandle,
    state: State<'_, AppState>,
    on_event: Channel<UpdateProgress>,
) -> Result<(), String> {
    let updater = updater(&app)?
        .ok_or_else(|| "In-app updates are not available in this build.".to_owned())?;
    let update = updater
        .check()
        .await
        .map_err(|error| format!("Could not check for updates: {error}"))?
        .ok_or_else(|| "Live Downloader is already up to date.".to_owned())?;

    let mut started = false;
    let bytes = update
        .download(
            |chunk_length, content_length| {
                if !started {
                    started = true;
                    let _ = on_event.send(UpdateProgress::Started { content_length });
                }
                let _ = on_event.send(UpdateProgress::Progress { chunk_length });
            },
            || {},
        )
        .await
        .map_err(|error| format!("Could not download the update: {error}"))?;

    let _ = on_event.send(UpdateProgress::Installing);
    // On Windows the installer exits the process without running the normal
    // exit handlers, so recorders are stopped and their files salvaged first.
    state.engine.shutdown();
    if let Err(error) = update.install(bytes) {
        let _ = state.engine.start();
        return Err(format!("Could not install the update: {error}"));
    }
    app.restart();
}
