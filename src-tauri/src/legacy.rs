use std::path::{Path, PathBuf};

use serde::Deserialize;
use url::Url;

use crate::{database::Database, models::LegacyImportResult};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LegacyConfig {
    #[serde(default)]
    streams: Vec<String>,
    #[serde(default)]
    yt_dlp_path: Option<String>,
    #[serde(default)]
    check_interval_seconds: Option<u64>,
    #[serde(default)]
    download_directory: Option<String>,
}

pub fn find_legacy_config() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let direct = cwd.join("config.json");
    if direct.is_file() && looks_like_legacy_config(&direct) {
        return Some(direct);
    }

    cwd.parent()
        .map(|parent| parent.join("config.json"))
        .filter(|path| path.is_file() && looks_like_legacy_config(path))
}

pub fn import_legacy_config(
    database: &Database,
    config_path: &Path,
) -> Result<LegacyImportResult, String> {
    let contents = std::fs::read_to_string(config_path).map_err(|error| error.to_string())?;
    let legacy: LegacyConfig = serde_json::from_str(&contents)
        .map_err(|error| format!("Could not read the legacy config: {error}"))?;

    let mut imported = 0;
    let mut skipped = 0;
    for raw_url in legacy.streams {
        let url = raw_url.trim();
        if !is_http_url(url) {
            skipped += 1;
            continue;
        }
        let name = Url::parse(url)
            .ok()
            .and_then(|parsed| parsed.host_str().map(str::to_owned))
            .unwrap_or_else(|| "Imported stream".to_owned());
        match database.insert_target(&name, url) {
            Ok(_) => imported += 1,
            Err(error) if error.contains("already") => skipped += 1,
            Err(error) => return Err(error),
        }
    }

    let mut settings = database.settings()?;
    let mut settings_imported = false;
    if let Some(directory) = legacy
        .download_directory
        .filter(|value| !value.trim().is_empty())
    {
        settings.download_directory = directory;
        settings_imported = true;
    }
    if let Some(interval) = legacy.check_interval_seconds.filter(|value| *value >= 30) {
        settings.probe_interval_seconds = interval;
        settings_imported = true;
    }
    if let Some(path) = legacy.yt_dlp_path.filter(|value| !value.trim().is_empty()) {
        settings.external_ytdlp_path = Some(path);
        settings_imported = true;
    }
    if settings_imported {
        database.save_settings(&settings)?;
    }

    Ok(LegacyImportResult {
        imported,
        skipped,
        source_path: config_path.to_string_lossy().to_string(),
        settings_imported,
    })
}

pub fn is_http_url(value: &str) -> bool {
    Url::parse(value)
        .map(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
        .unwrap_or(false)
}

fn looks_like_legacy_config(path: &Path) -> bool {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str::<serde_json::Value>(&contents).ok())
        .is_some_and(|value| value.get("Streams").is_some())
}

#[cfg(test)]
mod tests {
    use super::is_http_url;

    #[test]
    fn accepts_absolute_http_urls_only() {
        assert!(is_http_url("https://www.twitch.tv/example"));
        assert!(is_http_url("http://localhost:8080/live"));
        assert!(!is_http_url("ftp://example.com/archive"));
        assert!(!is_http_url("https://"));
        assert!(!is_http_url("not a URL"));
    }
}
