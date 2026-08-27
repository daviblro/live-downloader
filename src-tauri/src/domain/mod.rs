use chrono::{DateTime, Utc};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Locale {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "pt-BR")]
    PortugueseBrazil,
}

impl Locale {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::PortugueseBrazil => "pt-BR",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetState {
    Watching,
    Checking,
    Recording,
    Queued,
    Retrying,
    #[serde(rename = "Needs attention")]
    NeedsAttention,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingState {
    Recording,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
}

fn parse_state<T>(value: ValueRef<'_>, parse: impl FnOnce(&str) -> Option<T>) -> FromSqlResult<T> {
    let text = value.as_str()?;
    parse(text).ok_or_else(|| {
        FromSqlError::Other(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Unknown persisted state: {text}"),
        )))
    })
}

impl FromSql for TargetState {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        parse_state(value, |value| match value {
            "Watching" => Some(Self::Watching),
            "Checking" => Some(Self::Checking),
            "Recording" => Some(Self::Recording),
            "Queued" => Some(Self::Queued),
            "Retrying" => Some(Self::Retrying),
            "Needs attention" => Some(Self::NeedsAttention),
            "Disabled" => Some(Self::Disabled),
            _ => None,
        })
    }
}

impl FromSql for RecordingState {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        parse_state(value, |value| match value {
            "Recording" => Some(Self::Recording),
            "Completed" => Some(Self::Completed),
            "Failed" => Some(Self::Failed),
            "Cancelled" => Some(Self::Cancelled),
            "Interrupted" => Some(Self::Interrupted),
            _ => None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default = "default_locale")]
    pub locale: Locale,
    pub theme: Theme,
    pub download_directory: String,
    pub probe_interval_seconds: u64,
    pub max_concurrent_recordings: usize,
    pub start_with_windows: bool,
    pub notifications_enabled: bool,
    pub external_ytdlp_path: Option<String>,
}

fn default_locale() -> Locale {
    Locale::English
}

impl Default for AppSettings {
    fn default() -> Self {
        let download_directory = directories::UserDirs::new()
            .and_then(|dirs| dirs.download_dir().map(|path| path.join("Live Downloader")))
            .unwrap_or_else(|| std::path::PathBuf::from("downloads"));

        Self {
            locale: default_locale(),
            theme: Theme::System,
            download_directory: download_directory.to_string_lossy().to_string(),
            probe_interval_seconds: 300,
            max_concurrent_recordings: 3,
            start_with_windows: false,
            notifications_enabled: true,
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
    pub state: TargetState,
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
    pub state: RecordingState,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub output_path: Option<String>,
    pub file_exists: bool,
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
