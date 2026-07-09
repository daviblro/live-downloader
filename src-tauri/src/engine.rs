use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::Duration as ChronoDuration;
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

    pub async fn start(&self) -> Result<EngineSummary, String> {
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
        let output_template =
            output_directory.join("%(uploader)s_%(upload_date)s_%(title)s.%(ext)s");
        let job = self.database.create_job(&target.id, None)?;
        let mut args = vec![
            "--no-cache".to_owned(),
            "--newline".to_owned(),
            "--continue".to_owned(),
            "--output".to_owned(),
            output_template.to_string_lossy().to_string(),
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
                tauri::async_runtime::spawn(async move {
                    let outcome = tokio::select! {
                        event = wait_for_sidecar_termination(&mut events) => event,
                        _ = cancellation.cancelled() => {
                            let _ = child.kill();
                            "Cancelled by the user".to_owned()
                        }
                    };
                    engine.finish_recording(&job, &target.id, outcome).await;
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
                    engine.finish_recording(&job, &target.id, outcome).await;
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

    async fn finish_recording(&self, job: &RecordingJob, target_id: &str, outcome: String) {
        let state = if outcome.starts_with("Recording completed") {
            "Completed"
        } else if outcome.starts_with("Cancelled") {
            "Cancelled"
        } else {
            "Failed"
        };
        let _ = self.database.finish_job(&job.id, state, &outcome);
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
    use std::path::Path;

    use super::bundled_sidecar_directory_from;

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
}
