use tauri::State;

use crate::{
    app::AppState,
    domain::LegacyImportResult,
    legacy::{find_legacy_config, import_legacy_config},
};

#[tauri::command]
pub async fn import_legacy(state: State<'_, AppState>) -> Result<LegacyImportResult, String> {
    let path = find_legacy_config().ok_or_else(|| {
        "No legacy config.json was found in the current or parent folder.".to_owned()
    })?;
    let result = import_legacy_config(&state.database, &path)?;
    let _ = state.engine.start();
    Ok(result)
}
