use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default = "default_locale")]
    pub locale: String,
    pub theme: String,
    pub download_directory: String,
    pub probe_interval_seconds: u64,
    pub max_concurrent_probes: usize,
    pub max_concurrent_recordings: usize,
    pub launch_to_tray: bool,
    pub start_with_windows: bool,
    pub notifications_enabled: bool,
    pub retain_logs_days: u16,
    pub external_ytdlp_path: Option<String>,
}

fn default_locale() -> String {
    "en".to_owned()
}

impl Default for AppSettings {
    fn default() -> Self {
        let download_directory = directories::UserDirs::new()
            .and_then(|dirs| dirs.download_dir().map(|path| path.join("Live Downloader")))
            .unwrap_or_else(|| std::path::PathBuf::from("downloads"));

        Self {
            locale: default_locale(),
            theme: "system".to_owned(),
            download_directory: download_directory.to_string_lossy().to_string(),
            probe_interval_seconds: 300,
            max_concurrent_probes: 6,
            max_concurrent_recordings: 3,
            launch_to_tray: true,
            start_with_windows: false,
            notifications_enabled: true,
            retain_logs_days: 30,
            external_ytdlp_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchTarget {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub state: String,
    pub status_detail: String,
    pub next_check_at: Option<String>,
    pub last_checked_at: Option<String>,
    pub last_recording_at: Option<String>,
    pub active_job_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingJob {
    pub id: String,
    pub target_id: String,
    pub target_name: String,
    pub state: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub output_path: Option<String>,
    pub message: String,
    pub process_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineSummary {
    pub running: bool,
    pub active_recordings: usize,
    pub enabled_targets: usize,
    pub next_global_check_at: Option<String>,
    pub sidecar_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskUsage {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub settings: AppSettings,
    pub disk_usage: Option<DiskUsage>,
    pub targets: Vec<WatchTarget>,
    pub jobs: Vec<RecordingJob>,
    pub engine: EngineSummary,
    pub legacy_config_available: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTargetInput {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTargetInput {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub source_path: String,
    pub settings_imported: bool,
}

pub fn now() -> DateTime<Utc> {
    Utc::now()
}

pub fn format_time(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}
