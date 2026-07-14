use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::{DateTime, Duration as ChronoDuration, Local, Utc};
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::{process::CommandEvent, ShellExt};
use tokio::{process::Command, sync::Semaphore};
use tokio_util::sync::CancellationToken;

use crate::{
    database::Database,
    legacy::is_http_url,
    models::{format_time, now, AppSettings, EngineSummary, RecordingJob, WatchTarget},
};

#[derive(Debug)]
struct ActiveJob {
    target_id: String,
    cancellation: CancellationToken,
}

#[derive(Debug)]
struct OutputCandidate {
    path: PathBuf,
    normalized_name: String,
    modified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default)]
struct EngineRuntime {
    running: bool,
    scheduler_started: bool,
    active_jobs: HashMap<String, ActiveJob>,
}

#[derive(Clone)]
pub struct RecordingEngine {
    app: AppHandle,
    database: Arc<Database>,
    runtime: Arc<Mutex<EngineRuntime>>,
    probe_limiter: Arc<Semaphore>,
}

impl RecordingEngine {
    pub fn new(app: AppHandle, database: Arc<Database>) -> Self {
        Self {
            app,
            database,
            runtime: Arc::new(Mutex::new(EngineRuntime::default())),
            probe_limiter: Arc::new(Semaphore::new(12)),
        }
    }

    pub fn database(&self) -> Arc<Database> {
        self.database.clone()
    }

    pub fn summary(&self) -> Result<EngineSummary, String> {
        let targets = self.database.enabled_targets()?;
        let runtime = self
            .runtime
            .lock()
            .map_err(|_| "Engine lock poisoned".to_owned())?;
        let settings = self.database.settings()?;
        Ok(EngineSummary {
            running: runtime.running,
            active_recordings: runtime.active_jobs.len(),
            enabled_targets: targets.len(),
            next_global_check_at: if runtime.running {
                Some(format_time(
                    now() + ChronoDuration::seconds(settings.probe_interval_seconds as i64),
                ))
            } else {
                None
            },
            sidecar_status: self.sidecar_status(&settings),
        })
    }

    pub fn reconcile_output_paths(&self, output_directory: &Path) -> Result<(), String> {
        if !output_directory.is_dir() {
            return Ok(());
        }

        let jobs = self.database.list_jobs(200)?;
        let assigned_paths = jobs
            .iter()
            .filter_map(|job| job.output_path.as_ref())
            .map(PathBuf::from)
            .collect::<HashSet<_>>();
        let mut candidates = recording_output_candidates(output_directory)?;
        candidates.retain(|candidate| !assigned_paths.contains(&candidate.path));

        for job in jobs.iter().filter(|job| {
            job.state == "Completed" && job.output_path.as_deref().map_or(true, str::is_empty)
        }) {
            let Some(target) = self.database.target(&job.target_id)? else {
                continue;
            };
            let Some((index, _)) = candidates
                .iter()
                .enumerate()
                .filter_map(|(index, candidate)| {
                    legacy_candidate_score(candidate, &target, job).map(|score| (index, score))
                })
                .max_by_key(|(_, score)| *score)
            else {
                continue;
            };

            let candidate = candidates.swap_remove(index);
            self.database
                .set_job_output_path(&job.id, candidate.path.to_string_lossy().as_ref())?;
        }

        Ok(())
    }

    pub fn start(&self) -> Result<EngineSummary, String> {
        {
            let mut runtime = self
                .runtime
                .lock()
                .map_err(|_| "Engine lock poisoned".to_owned())?;
            runtime.running = true;
            if !runtime.scheduler_started {
                runtime.scheduler_started = true;
                let engine = self.clone();
                tauri::async_runtime::spawn(async move { engine.scheduler_loop().await });
            }
        }
        self.emit_change();
        self.summary()
    }

    pub async fn pause_all(&self) -> Result<EngineSummary, String> {
        let active_tokens = {
            let mut runtime = self
                .runtime
                .lock()
                .map_err(|_| "Engine lock poisoned".to_owned())?;
            runtime.running = false;
            runtime
                .active_jobs
                .values()
                .map(|job| job.cancellation.clone())
                .collect::<Vec<_>>()
        };
        for token in active_tokens {
            token.cancel();
        }
        self.emit_change();
        self.summary()
    }

    pub async fn check_now(&self, target_id: String) -> Result<(), String> {
        let target = self
            .database
            .target(&target_id)?
            .ok_or_else(|| "That stream no longer exists.".to_owned())?;
        if !target.enabled {
            return Err("Enable the stream before checking it.".to_owned());
        }
        let engine = self.clone();
        tauri::async_runtime::spawn(async move { engine.probe_target(target).await });
        Ok(())
    }

    pub async fn stop_job(&self, job_id: String) -> Result<(), String> {
        let token = self
            .runtime
            .lock()
            .map_err(|_| "Engine lock poisoned".to_owned())?
            .active_jobs
            .get(&job_id)
            .map(|job| job.cancellation.clone())
            .ok_or_else(|| "That recording is no longer active.".to_owned())?;
        token.cancel();
        Ok(())
    }

    pub fn cancel_target(&self, target_id: &str) {
        let tokens = self
            .runtime
            .lock()
            .ok()
            .map(|runtime| {
                runtime
                    .active_jobs
                    .values()
                    .filter(|job| job.target_id == target_id)
                    .map(|job| job.cancellation.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for token in tokens {
            token.cancel();
        }
    }

    async fn scheduler_loop(self) {
        loop {
            let running = self
                .runtime
                .lock()
                .map(|runtime| runtime.running)
                .unwrap_or(false);
            if !running {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }

            let settings = match self.database.settings() {
                Ok(settings) => settings,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };
            let targets = self.database.enabled_targets().unwrap_or_default();
            for target in targets {
                let engine = self.clone();
                tauri::async_runtime::spawn(async move { engine.probe_target(target).await });
            }
            self.emit_change();
            tokio::time::sleep(Duration::from_secs(settings.probe_interval_seconds.max(30))).await;
        }
    }

    async fn probe_target(&self, target: WatchTarget) {
        let permit = match self.probe_limiter.clone().acquire_owned().await {
            Ok(permit) => permit,
            Err(_) => return,
        };
        if self.has_active_job(&target.id) {
            drop(permit);
            return;
        }

        let settings = match self.database.settings() {
            Ok(settings) => settings,
            Err(_) => return,
        };
        let next_check = format_time(
            now() + ChronoDuration::seconds(settings.probe_interval_seconds.max(30) as i64),
        );
        let _ = self.database.set_target_status(
            &target.id,
            "Checking",
            "Checking whether the stream is live",
            Some(&next_check),
            None,
        );
        self.emit_change();

        let probe_result =
            tokio::time::timeout(Duration::from_secs(30), self.probe(&settings, &target.url)).await;
        drop(permit);
        match probe_result {
            Ok(Ok(true)) => {
                let target_id = target.id.clone();
                if let Err(error) = self.start_recording(target, settings).await {
                    let _ = self.database.set_target_status(
                        &target_id,
                        "Needs attention",
                        &error,
                        Some(&next_check),
                        None,
                    );
                    self.emit_change();
                }
            }
            Ok(Ok(false)) => {
                let _ = self.database.set_target_status(
                    &target.id,
                    "Watching",
                    "Waiting for the stream to go live",
                    Some(&next_check),
                    None,
                );
                self.emit_change();
            }
            Ok(Err(error)) => {
                let _ = self.database.set_target_status(
                    &target.id,
                    "Needs attention",
                    &error,
                    Some(&next_check),
                    None,
                );
                self.emit_change();
            }
            Err(_) => {
                let _ = self.database.set_target_status(
                    &target.id,
                    "Retrying",
                    "Live check timed out; the next scheduled check will retry",
                    Some(&next_check),
                    None,
                );
                self.emit_change();
            }
        }
    }

    async fn probe(&self, settings: &AppSettings, url: &str) -> Result<bool, String> {
        if !is_http_url(url) {
            return Err("The stream URL must be an absolute HTTP or HTTPS URL.".to_owned());
        }
        let mut args = vec![
            "--no-cache".to_owned(),
            "--simulate".to_owned(),
            "--quiet".to_owned(),
            "--no-warnings".to_owned(),
        ];
        append_managed_ffmpeg_location(&mut args);
        args.push(url.to_owned());
        match self.command_source(settings)? {
            CommandSource::Bundled => {
                let output = self
                    .app
                    .shell()
                    .sidecar("yt-dlp")
                    .map_err(|error| format!("Managed yt-dlp sidecar is unavailable: {error}"))?
                    .args(args)
                    .output()
                    .await
                    .map_err(|error| format!("Could not run managed yt-dlp: {error}"))?;
                Ok(output.status.success())
            }
            CommandSource::External(path) => {
                let output = Command::new(path)
                    .args(args)
                    .output()
                    .await
                    .map_err(|error| {
                        format!("Could not run the configured yt-dlp executable: {error}")
                    })?;
                Ok(output.status.success())
            }
        }
    }

    async fn start_recording(
        &self,
        target: WatchTarget,
        settings: AppSettings,
    ) -> Result<(), String> {
        let source = self.command_source(&settings)?;
        let active_count = self
            .runtime
            .lock()
            .map_err(|_| "Engine lock poisoned".to_owned())?
            .active_jobs
            .len();
        if active_count >= settings.max_concurrent_recordings.max(1) {
            let next_check = format_time(now() + ChronoDuration::seconds(30));
            self.database.set_target_status(
                &target.id,
                "Queued",
                "Waiting for a recording slot",
                Some(&next_check),
                None,
            )?;
            self.emit_change();
            return Ok(());
        }

        let output_directory = Path::new(&settings.download_directory);
        std::fs::create_dir_all(output_directory)
            .map_err(|error| format!("Could not create the download directory: {error}"))?;
        let local_timestamp = Local::now()
            .format(recording_timestamp_format(&settings.locale))
            .to_string();
        let output_template = recording_output_template(output_directory, &local_timestamp);
        let job = self.database.create_job(&target.id, None)?;
        let output_path_receipt =
            std::env::temp_dir().join(format!("live-downloader-{}.path", job.id));
        let _ = std::fs::remove_file(&output_path_receipt);
        let mut args = vec![
            "--no-cache".to_owned(),
            "--newline".to_owned(),
            "--continue".to_owned(),
            "--output".to_owned(),
            output_template.to_string_lossy().to_string(),
            "--print-to-file".to_owned(),
            "after_move:filepath".to_owned(),
            output_path_receipt.to_string_lossy().to_string(),
        ];
        append_managed_ffmpeg_location(&mut args);
        args.push(target.url.clone());
        let cancellation = CancellationToken::new();

        match source {
            CommandSource::Bundled => {
                let (mut events, child) = self
                    .app
                    .shell()
                    .sidecar("yt-dlp")
                    .map_err(|error| format!("Managed yt-dlp sidecar is unavailable: {error}"))?
                    .args(args)
                    .spawn()
                    .map_err(|error| format!("Could not start managed yt-dlp: {error}"))?;
                let pid = child.pid();
                self.database.set_job_pid(&job.id, pid)?;
                self.register_active(&job, &target, pid, cancellation.clone())?;
                let engine = self.clone();
                let output_path_receipt = output_path_receipt.clone();
                tauri::async_runtime::spawn(async move {
                    let outcome = tokio::select! {
                        event = wait_for_sidecar_termination(&mut events) => event,
                        _ = cancellation.cancelled() => {
                            let _ = child.kill();
                            "Cancelled by the user".to_owned()
                        }
                    };
                    engine
                        .finish_recording(&job, &target.id, outcome, &output_path_receipt)
                        .await;
                });
            }
            CommandSource::External(path) => {
                let mut child = Command::new(path).args(args).spawn().map_err(|error| {
                    format!("Could not start the configured yt-dlp executable: {error}")
                })?;
                let pid = child
                    .id()
                    .ok_or_else(|| "yt-dlp did not return a process id".to_owned())?;
                self.database.set_job_pid(&job.id, pid)?;
                self.register_active(&job, &target, pid, cancellation.clone())?;
                let engine = self.clone();
                let output_path_receipt = output_path_receipt.clone();
                tauri::async_runtime::spawn(async move {
                    let outcome = tokio::select! {
                        status = child.wait() => match status {
                            Ok(status) if status.success() => "Recording completed".to_owned(),
                            Ok(status) => format!("yt-dlp exited with code {:?}", status.code()),
                            Err(error) => format!("yt-dlp could not be monitored: {error}"),
                        },
                        _ = cancellation.cancelled() => {
                            let _ = child.start_kill();
                            let _ = child.wait().await;
                            "Cancelled by the user".to_owned()
                        }
                    };
                    engine
                        .finish_recording(&job, &target.id, outcome, &output_path_receipt)
                        .await;
                });
            }
        }
        self.emit_change();
        Ok(())
    }

    fn register_active(
        &self,
        job: &RecordingJob,
        target: &WatchTarget,
        _pid: u32,
        cancellation: CancellationToken,
    ) -> Result<(), String> {
        self.runtime
            .lock()
            .map_err(|_| "Engine lock poisoned".to_owned())?
            .active_jobs
            .insert(
                job.id.clone(),
                ActiveJob {
                    target_id: target.id.clone(),
                    cancellation,
                },
            );
        self.database.set_target_status(
            &target.id,
            "Recording",
            "Recording in the background",
            None,
            Some(&job.id),
        )?;
        Ok(())
    }

    async fn finish_recording(
        &self,
        job: &RecordingJob,
        target_id: &str,
        outcome: String,
        output_path_receipt: &Path,
    ) {
        let state = if outcome.starts_with("Recording completed") {
            "Completed"
        } else if outcome.starts_with("Cancelled") {
            "Cancelled"
        } else {
            "Failed"
        };
        let output_path = take_output_path_receipt(output_path_receipt)
            .map(|path| path.to_string_lossy().into_owned());
        let _ = self
            .database
            .finish_job(&job.id, state, &outcome, output_path.as_deref());
        let _ = self.database.target_recording_finished(target_id, &outcome);
        if let Ok(mut runtime) = self.runtime.lock() {
            runtime.active_jobs.remove(&job.id);
        }
        self.emit_change();
    }

    fn has_active_job(&self, target_id: &str) -> bool {
        self.runtime
            .lock()
            .map(|runtime| {
                runtime
                    .active_jobs
                    .values()
                    .any(|job| job.target_id == target_id)
            })
            .unwrap_or(false)
    }

    fn command_source(&self, settings: &AppSettings) -> Result<CommandSource, String> {
        if let Some(path) = settings
            .external_ytdlp_path
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            let path = std::path::PathBuf::from(path);
            if path.is_file() {
                return Ok(CommandSource::External(path));
            }
            return Err(format!(
                "The configured yt-dlp executable was not found at {}",
                path.display()
            ));
        }
        Ok(CommandSource::Bundled)
    }

    fn sidecar_status(&self, settings: &AppSettings) -> String {
        match settings.external_ytdlp_path.as_ref() {
            Some(path) if Path::new(path).is_file() => "External yt-dlp ready".to_owned(),
            Some(_) => "External yt-dlp path is unavailable".to_owned(),
            None if managed_ffmpeg_directory().is_some() => {
                "Managed yt-dlp + FFmpeg sidecars".to_owned()
            }
            None => "Managed yt-dlp sidecar".to_owned(),
        }
    }

    fn emit_change(&self) {
        if let Ok(summary) = self.summary() {
            let _ = self.app.emit("engine://changed", summary);
        }
    }
}

fn recording_output_template(output_directory: &Path, local_timestamp: &str) -> PathBuf {
    output_directory.join(format!("%(channel,uploader)s - {local_timestamp}.%(ext)s"))
}

fn recording_timestamp_format(locale: &str) -> &'static str {
    if locale == "pt-BR" {
        "%d-%m-%Y %H-%M-%S"
    } else {
        "%m-%d-%Y %H-%M-%S"
    }
}

fn take_output_path_receipt(receipt: &Path) -> Option<PathBuf> {
    let contents = std::fs::read_to_string(receipt).ok();
    let _ = std::fs::remove_file(receipt);
    let path = PathBuf::from(
        contents?
            .lines()
            .rev()
            .map(str::trim)
            .find(|line| !line.is_empty())?,
    );
    path.is_file().then_some(path)
}

fn recording_output_candidates(output_directory: &Path) -> Result<Vec<OutputCandidate>, String> {
    let entries = std::fs::read_dir(output_directory)
        .map_err(|error| format!("Could not inspect the download directory: {error}"))?;
    Ok(entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() || !is_recording_file(&path) {
                return None;
            }
            let normalized_name = normalize_name(path.file_stem()?.to_string_lossy().as_ref());
            let modified_at = entry
                .metadata()
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .map(DateTime::<Utc>::from);
            Some(OutputCandidate {
                path,
                normalized_name,
                modified_at,
            })
        })
        .collect())
}

fn is_recording_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "avi" | "flv" | "m4v" | "mkv" | "mov" | "mp4" | "ts" | "webm"
            )
        })
}

fn legacy_candidate_score(
    candidate: &OutputCandidate,
    target: &WatchTarget,
    job: &RecordingJob,
) -> Option<i64> {
    let identifiers = target_identifiers(target);
    let matching_identifier = identifiers
        .iter()
        .filter(|identifier| candidate.normalized_name.contains(identifier.as_str()))
        .max_by_key(|identifier| identifier.len())?;
    let started_at = parse_job_time(&job.started_at);
    let finished_at = job.finished_at.as_deref().and_then(parse_job_time);
    let mut date_tokens = Vec::new();
    let mut time_tokens = Vec::new();
    for timestamp in [started_at, finished_at].into_iter().flatten() {
        date_tokens.push(timestamp.format("%Y%m%d").to_string());
        date_tokens.push(timestamp.format("%d%m%Y").to_string());
        date_tokens.push(timestamp.format("%m%d%Y").to_string());
        time_tokens.push(timestamp.format("%H%M").to_string());
        let local = timestamp.with_timezone(&Local);
        date_tokens.push(local.format("%Y%m%d").to_string());
        date_tokens.push(local.format("%d%m%Y").to_string());
        date_tokens.push(local.format("%m%d%Y").to_string());
        time_tokens.push(local.format("%H%M").to_string());
    }

    let has_matching_date = date_tokens
        .iter()
        .any(|token| candidate.normalized_name.contains(token));
    let has_matching_time = time_tokens
        .iter()
        .any(|token| candidate.normalized_name.contains(token));
    let reference_time = finished_at.or(started_at);
    let proximity_seconds = candidate
        .modified_at
        .zip(reference_time)
        .map(|(modified, reference)| (modified - reference).num_seconds().unsigned_abs());
    if !has_matching_date && !proximity_seconds.is_some_and(|seconds| seconds <= 3_600) {
        return None;
    }

    let mut score = 10 + matching_identifier.len() as i64;
    if candidate.normalized_name.starts_with(matching_identifier) {
        score += 5;
    }
    if has_matching_date {
        score += 10;
    }
    if has_matching_time {
        score += 5;
    }
    score += match proximity_seconds {
        Some(0..=300) => 5,
        Some(301..=3_600) => 3,
        _ => 0,
    };
    Some(score)
}

fn target_identifiers(target: &WatchTarget) -> Vec<String> {
    let mut identifiers = vec![normalize_name(&target.name)];
    if let Ok(url) = url::Url::parse(&target.url) {
        if let Some(segments) = url.path_segments() {
            if let Some(segment) = segments
                .rev()
                .map(|segment| segment.trim_start_matches('@'))
                .find(|segment| !segment.is_empty() && !matches!(*segment, "live" | "videos"))
            {
                identifiers.push(normalize_name(segment));
            }
        }
    }
    identifiers.retain(|identifier| identifier.len() >= 3);
    identifiers.sort();
    identifiers.dedup();
    identifiers
}

fn normalize_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn parse_job_time(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.with_timezone(&Utc))
}

enum CommandSource {
    Bundled,
    External(std::path::PathBuf),
}

fn append_managed_ffmpeg_location(args: &mut Vec<String>) {
    if let Some(directory) = managed_ffmpeg_directory() {
        args.push("--ffmpeg-location".to_owned());
        args.push(directory.to_string_lossy().to_string());
    }
}

fn managed_ffmpeg_directory() -> Option<PathBuf> {
    let directory = bundled_sidecar_directory_from(&std::env::current_exe().ok()?)?;
    directory.join("ffmpeg.exe").is_file().then_some(directory)
}

fn bundled_sidecar_directory_from(executable: &Path) -> Option<PathBuf> {
    let directory = executable.parent()?;
    if directory.file_name().is_some_and(|name| name == "deps") {
        directory.parent().map(Path::to_path_buf)
    } else {
        Some(directory.to_path_buf())
    }
}

async fn wait_for_sidecar_termination(
    events: &mut tauri::async_runtime::Receiver<CommandEvent>,
) -> String {
    while let Some(event) = events.recv().await {
        match event {
            CommandEvent::Terminated(payload) => {
                return match payload.code {
                    Some(0) => "Recording completed".to_owned(),
                    Some(code) => format!("yt-dlp exited with code {code}"),
                    None => "yt-dlp process ended".to_owned(),
                };
            }
            CommandEvent::Error(error) => return format!("yt-dlp error: {error}"),
            _ => {}
        }
    }
    "yt-dlp output stream ended unexpectedly".to_owned()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        bundled_sidecar_directory_from, legacy_candidate_score, normalize_name,
        recording_output_template, recording_timestamp_format, take_output_path_receipt,
        OutputCandidate,
    };
    use crate::models::{RecordingJob, WatchTarget};

    #[test]
    fn resolves_the_same_sidecar_directory_as_tauri_shell() {
        assert_eq!(
            bundled_sidecar_directory_from(Path::new(r"C:\app\live-downloader.exe")),
            Some(r"C:\app".into())
        );
        assert_eq!(
            bundled_sidecar_directory_from(Path::new(r"C:\app\deps\live_downloader_tests.exe")),
            Some(r"C:\app".into())
        );
    }

    #[test]
    fn uses_channel_and_windows_safe_timestamp_for_output_name() {
        assert_eq!(
            recording_output_template(Path::new(r"C:\Recordings"), "13-07-2026 22-42-15"),
            PathBuf::from(r"C:\Recordings\%(channel,uploader)s - 13-07-2026 22-42-15.%(ext)s")
        );
        assert_eq!(recording_timestamp_format("en"), "%m-%d-%Y %H-%M-%S");
        assert_eq!(recording_timestamp_format("pt-BR"), "%d-%m-%Y %H-%M-%S");
    }

    #[test]
    fn reads_and_removes_the_final_output_path_receipt() {
        let directory = std::env::temp_dir().join(format!(
            "live-downloader-output-test-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&directory).expect("temp directory should be created");
        let output = directory.join("channel - 13-07-2026 22-42-15.mp4");
        let receipt = directory.join("recording.path");
        std::fs::write(&output, []).expect("output should be created");
        std::fs::write(&receipt, format!("{}\n", output.display()))
            .expect("receipt should be created");

        assert_eq!(take_output_path_receipt(&receipt), Some(output.clone()));
        assert!(!receipt.exists());

        let _ = std::fs::remove_file(output);
        let _ = std::fs::remove_dir(directory);
    }

    #[test]
    fn recognizes_the_previous_output_name_for_completed_jobs() {
        let target = WatchTarget {
            id: "target-1".to_owned(),
            name: "Reage Carlos".to_owned(),
            url: "https://www.twitch.tv/soucarlosdaniel".to_owned(),
            enabled: true,
            state: "Watching".to_owned(),
            status_detail: String::new(),
            next_check_at: None,
            last_checked_at: None,
            last_recording_at: None,
            active_job_id: None,
            created_at: "2026-07-13T00:00:00Z".to_owned(),
        };
        let job = RecordingJob {
            id: "job-1".to_owned(),
            target_id: target.id.clone(),
            target_name: target.name.clone(),
            state: "Completed".to_owned(),
            started_at: "2026-07-13T21:42:15Z".to_owned(),
            finished_at: Some("2026-07-13T22:00:00Z".to_owned()),
            output_path: None,
            message: "Recording completed".to_owned(),
            process_id: None,
        };
        let candidate = OutputCandidate {
            path: PathBuf::from(
                r"C:\Recordings\soucarlosdaniel_20260713_soucarlosdaniel (live) 2026-07-13 22_42.mp4",
            ),
            normalized_name: normalize_name(
                "soucarlosdaniel_20260713_soucarlosdaniel (live) 2026-07-13 22_42",
            ),
            modified_at: None,
        };

        assert!(legacy_candidate_score(&candidate, &target, &job).is_some());
    }
}
