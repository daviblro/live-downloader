use std::{
    collections::HashSet,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::{DateTime, Duration as ChronoDuration, Local, Utc};
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::{process::CommandEvent, ShellExt};
use tokio::{
    process::Command,
    sync::{Notify, Semaphore},
};
use tokio_util::sync::CancellationToken;

use super::{
    probe::{classify_probe_output, ProbeOutcome, LIVE_STATUS_TEMPLATE, MAX_CONCURRENT_PROBES},
    recorder::{ActiveJob, PartialOutput},
    scheduler::EngineRuntime,
};
use crate::{
    domain::{
        format_time, now, AppSettings, EngineSummary, RecordingJob, RecordingState, WatchTarget,
    },
    legacy::is_http_url,
    persistence::Database,
    platform::{
        disk::disk_usage_for,
        process::{hide_console_window, kill_process_tree},
    },
};

const PROBE_TIMEOUT: Duration = Duration::from_secs(45);
const PROCESS_EXIT_TIMEOUT: Duration = Duration::from_secs(10);
/// A recording that ran at least this long is followed by a quick re-check, so
/// a dropped connection or a briefly interrupted broadcast resumes promptly
/// instead of waiting for the next scheduled check.
const RESUME_MIN_RECORDING: ChronoDuration = ChronoDuration::seconds(60);
const RESUME_CHECK_DELAY: Duration = Duration::from_secs(15);
const MIN_FREE_DISK_BYTES: u64 = 1024 * 1024 * 1024;
const USER_STOPPED_DETAIL: &str = "Recording stopped by the user; waiting for the next broadcast";
const NOT_LIVE_DETAIL: &str = "Waiting for the stream to go live";

struct ProbeProcessOutput {
    success: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[derive(Debug)]
struct OutputCandidate {
    path: PathBuf,
    normalized_name: String,
    modified_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct RecordingEngine {
    app: AppHandle,
    database: Arc<Database>,
    runtime: Arc<Mutex<EngineRuntime>>,
    probe_limiter: Arc<Semaphore>,
    wake: Arc<Notify>,
}

impl RecordingEngine {
    pub fn new(app: AppHandle, database: Arc<Database>) -> Self {
        Self {
            app,
            database,
            runtime: Arc::new(Mutex::new(EngineRuntime::default())),
            probe_limiter: Arc::new(Semaphore::new(MAX_CONCURRENT_PROBES)),
            wake: Arc::new(Notify::new()),
        }
    }

    pub fn settings_changed(&self, schedule_changed: bool) {
        if schedule_changed {
            self.wake.notify_one();
        }
        self.emit_change();
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
            next_global_check_at: runtime
                .running
                .then_some(runtime.next_tick_at)
                .flatten()
                .map(format_time),
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
            job.state == RecordingState::Completed
                && job.output_path.as_deref().map_or(true, str::is_empty)
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
        let resumed = {
            let mut runtime = self
                .runtime
                .lock()
                .map_err(|_| "Engine lock poisoned".to_owned())?;
            // The scheduler's first pass runs immediately; only a resume from
            // pause needs to interrupt its sleep.
            let resumed = !runtime.running && runtime.scheduler_started;
            runtime.running = true;
            if !runtime.scheduler_started {
                runtime.scheduler_started = true;
                let engine = self.clone();
                tauri::async_runtime::spawn(async move { engine.scheduler_loop().await });
            }
            resumed
        };
        if resumed {
            self.wake.notify_one();
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
            runtime.next_tick_at = None;
            runtime.queued_targets.clear();
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
        // A manual check is an explicit request to record again after a stop.
        if let Ok(mut runtime) = self.runtime.lock() {
            runtime.suppressed_targets.remove(&target_id);
        }
        self.spawn_probe(target);
        Ok(())
    }

    /// Checks a newly added, re-enabled or edited target right away instead of
    /// waiting for the next scheduled check.
    pub fn check_soon(&self, target_id: &str) {
        let running = self
            .runtime
            .lock()
            .map(|runtime| runtime.running)
            .unwrap_or(false);
        if !running {
            return;
        }
        if let Ok(Some(target)) = self.database.target(target_id) {
            if target.enabled {
                self.spawn_probe(target);
            }
        }
    }

    pub async fn stop_job(&self, job_id: String) -> Result<(), String> {
        let token = {
            let mut runtime = self
                .runtime
                .lock()
                .map_err(|_| "Engine lock poisoned".to_owned())?;
            let (target_id, token) = runtime
                .active_jobs
                .get(&job_id)
                .map(|job| (job.target_id.clone(), job.cancellation.clone()))
                .ok_or_else(|| "That recording is no longer active.".to_owned())?;
            // Without this the next scheduled check would immediately start
            // recording the same broadcast again.
            runtime.suppressed_targets.insert(target_id);
            token
        };
        token.cancel();
        Ok(())
    }

    pub fn cancel_target(&self, target_id: &str) {
        let tokens = self
            .runtime
            .lock()
            .ok()
            .map(|mut runtime| {
                runtime.queued_targets.retain(|queued| queued != target_id);
                runtime.probing_targets.remove(target_id);
                runtime.starting_targets.remove(target_id);
                runtime.suppressed_targets.remove(target_id);
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

    /// Stops every recorder process tree synchronously. Used when the
    /// application exits, when spawned cleanup tasks will no longer run.
    pub fn shutdown(&self) {
        let jobs = self
            .runtime
            .lock()
            .map(|mut runtime| {
                runtime.running = false;
                runtime.queued_targets.clear();
                runtime.active_jobs.drain().collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for (job_id, job) in jobs {
            job.cancellation.cancel();
            kill_process_tree(job.pid);
            let output_path = recover_partial_output(&job.output.directory, &job.output.timestamp)
                .map(|path| path.to_string_lossy().into_owned());
            let _ = self.database.finish_job(
                &job_id,
                "Interrupted",
                "Recording was interrupted when the application stopped",
                output_path.as_deref(),
            );
            let _ = self
                .database
                .target_recording_finished(&job.target_id, "Previous recording was interrupted");
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
                tokio::select! {
                    _ = self.wake.notified() => {}
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                }
                continue;
            }

            let settings = match self.database.settings() {
                Ok(settings) => settings,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };
            let interval = Duration::from_secs(settings.probe_interval_seconds.max(30));
            if let Ok(mut runtime) = self.runtime.lock() {
                runtime.next_tick_at =
                    Some(now() + ChronoDuration::seconds(interval.as_secs() as i64));
            }
            let targets = self.database.enabled_targets().unwrap_or_default();
            for target in targets {
                self.spawn_probe(target);
            }
            self.emit_change();
            tokio::select! {
                _ = self.wake.notified() => {}
                _ = tokio::time::sleep(interval) => {}
            }
        }
    }

    async fn probe_target(&self, target: WatchTarget) {
        if !self.reserve_probe(&target.id) {
            return;
        }
        let target_id = target.id.clone();
        self.probe_target_reserved(target).await;
        if let Ok(mut runtime) = self.runtime.lock() {
            runtime.probing_targets.remove(&target_id);
        }
        self.drain_queue();
    }

    async fn probe_target_reserved(&self, target: WatchTarget) {
        let permit = match self.probe_limiter.clone().acquire_owned().await {
            Ok(permit) => permit,
            Err(_) => return,
        };
        if self.has_active_or_starting_job(&target.id) {
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

        let probe_result = self.probe(&settings, &target.url).await;
        drop(permit);
        match probe_result {
            Ok(ProbeOutcome::Live) if self.is_suppressed(&target.id) => {
                let _ = self.database.set_target_status(
                    &target.id,
                    "Watching",
                    USER_STOPPED_DETAIL,
                    Some(&next_check),
                    None,
                );
                self.emit_change();
            }
            Ok(ProbeOutcome::Live) => {
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
            Ok(ProbeOutcome::Offline) => {
                if let Ok(mut runtime) = self.runtime.lock() {
                    runtime.suppressed_targets.remove(&target.id);
                }
                let _ = self.database.set_target_status(
                    &target.id,
                    "Watching",
                    NOT_LIVE_DETAIL,
                    Some(&next_check),
                    None,
                );
                self.emit_change();
            }
            Ok(ProbeOutcome::Failed(error)) => {
                let _ = self.database.set_target_status(
                    &target.id,
                    "Needs attention",
                    &error,
                    Some(&next_check),
                    None,
                );
                self.emit_change();
            }
            Err(error) => {
                let _ = self.database.set_target_status(
                    &target.id,
                    "Retrying",
                    &error,
                    Some(&next_check),
                    None,
                );
                self.emit_change();
            }
        }
    }

    /// Runs the live check. `Err` means the check itself could not complete
    /// (timeout or the tool could not start) and will be retried.
    async fn probe(&self, settings: &AppSettings, url: &str) -> Result<ProbeOutcome, String> {
        if !is_http_url(url) {
            return Ok(ProbeOutcome::Failed(
                "The stream URL must be an absolute HTTP or HTTPS URL.".to_owned(),
            ));
        }
        let mut args = vec![
            "--no-cache".to_owned(),
            "--no-warnings".to_owned(),
            "--playlist-items".to_owned(),
            "1".to_owned(),
            "--print".to_owned(),
            LIVE_STATUS_TEMPLATE.to_owned(),
        ];
        append_managed_ffmpeg_location(&mut args);
        args.push(url.to_owned());
        let source = match self.command_source(settings) {
            Ok(source) => source,
            Err(error) => return Ok(ProbeOutcome::Failed(error)),
        };
        let output = match source {
            CommandSource::Bundled => self.run_sidecar_probe(args).await,
            CommandSource::External(path) => run_external_probe(&path, args).await,
        };
        let output = match output {
            Ok(Some(output)) => output,
            Ok(None) => {
                return Err("Live check timed out; the next scheduled check will retry".to_owned())
            }
            Err(error) => return Ok(ProbeOutcome::Failed(error)),
        };
        Ok(classify_probe_output(
            output.success,
            &output.stdout,
            &output.stderr,
        ))
    }

    /// Returns `Ok(None)` when the check timed out; the process tree is killed
    /// so timed-out checks do not accumulate in the background.
    async fn run_sidecar_probe(
        &self,
        args: Vec<String>,
    ) -> Result<Option<ProbeProcessOutput>, String> {
        let (mut events, child) = self
            .app
            .shell()
            .sidecar("yt-dlp")
            .map_err(|error| format!("Managed yt-dlp sidecar is unavailable: {error}"))?
            .args(args)
            .spawn()
            .map_err(|error| format!("Could not run managed yt-dlp: {error}"))?;
        let pid = child.pid();
        let collect = async {
            let mut output = ProbeProcessOutput {
                success: false,
                stdout: Vec::new(),
                stderr: Vec::new(),
            };
            while let Some(event) = events.recv().await {
                match event {
                    CommandEvent::Stdout(line) => {
                        output.stdout.extend_from_slice(&line);
                        output.stdout.push(b'\n');
                    }
                    CommandEvent::Stderr(line) => {
                        output.stderr.extend_from_slice(&line);
                        output.stderr.push(b'\n');
                    }
                    CommandEvent::Terminated(payload) => {
                        output.success = payload.code == Some(0);
                        break;
                    }
                    CommandEvent::Error(error) => {
                        output.stderr.extend_from_slice(error.as_bytes());
                        break;
                    }
                    _ => {}
                }
            }
            output
        };
        match tokio::time::timeout(PROBE_TIMEOUT, collect).await {
            Ok(output) => Ok(Some(output)),
            Err(_) => {
                kill_process_tree(pid);
                let _ = child.kill();
                Ok(None)
            }
        }
    }

    async fn start_recording(
        &self,
        target: WatchTarget,
        settings: AppSettings,
    ) -> Result<(), String> {
        let target_id = target.id.clone();
        let queued = {
            let mut runtime = self
                .runtime
                .lock()
                .map_err(|_| "Engine lock poisoned".to_owned())?;
            let already_active = runtime
                .active_jobs
                .values()
                .any(|job| job.target_id == target_id)
                || runtime.starting_targets.contains(&target_id);
            if already_active {
                return Ok(());
            }
            if runtime.active_jobs.len() + runtime.starting_targets.len()
                >= settings.max_concurrent_recordings.max(1)
            {
                if !runtime.queued_targets.contains(&target_id) {
                    runtime.queued_targets.push_back(target_id.clone());
                }
                true
            } else {
                runtime.starting_targets.insert(target_id.clone());
                false
            }
        };

        if queued {
            let next_check = format_time(now() + ChronoDuration::seconds(30));
            self.database.set_target_status(
                &target_id,
                "Queued",
                "Waiting for a recording slot",
                Some(&next_check),
                None,
            )?;
            self.emit_change();
            return Ok(());
        }

        let result = self.start_recording_reserved(target, settings).await;
        if result.is_err() {
            if let Ok(mut runtime) = self.runtime.lock() {
                runtime.starting_targets.remove(&target_id);
            }
        }
        result
    }

    async fn start_recording_reserved(
        &self,
        target: WatchTarget,
        settings: AppSettings,
    ) -> Result<(), String> {
        let source = self.command_source(&settings)?;

        let output_directory = Path::new(&settings.download_directory);
        std::fs::create_dir_all(output_directory)
            .map_err(|error| format!("Could not create the download directory: {error}"))?;
        if let Some(usage) = disk_usage_for(output_directory) {
            if usage.available_bytes < MIN_FREE_DISK_BYTES {
                return Err(
                    "Less than 1 GB is free on the recording drive; free up space to resume recording"
                        .to_owned(),
                );
            }
        }
        let local_timestamp = Local::now()
            .format(recording_timestamp_format(settings.locale.as_str()))
            .to_string();
        let output_template = recording_output_template(output_directory, &local_timestamp);
        let partial_output = PartialOutput {
            directory: output_directory.to_path_buf(),
            timestamp: local_timestamp,
        };
        let job = self.database.create_job(&target.id, None)?;
        let output_path_receipt =
            std::env::temp_dir().join(format!("live-downloader-{}.path", job.id));
        let _ = std::fs::remove_file(&output_path_receipt);
        let mut args = vec![
            "--no-cache".to_owned(),
            "--newline".to_owned(),
            "--continue".to_owned(),
            // MPEG-TS stays playable if the recorder is stopped or crashes
            // before yt-dlp can remux the file.
            "--hls-use-mpegts".to_owned(),
            "--output".to_owned(),
            output_template.to_string_lossy().to_string(),
            "--print-to-file".to_owned(),
            "after_move:filepath".to_owned(),
            output_path_receipt.to_string_lossy().to_string(),
        ];
        append_managed_ffmpeg_location(&mut args);
        args.push(target.url.clone());
        let cancellation = CancellationToken::new();

        let spawned = match source {
            CommandSource::Bundled => self
                .app
                .shell()
                .sidecar("yt-dlp")
                .map_err(|error| format!("Managed yt-dlp sidecar is unavailable: {error}"))
                .and_then(|command| {
                    command
                        .args(args)
                        .spawn()
                        .map_err(|error| format!("Could not start managed yt-dlp: {error}"))
                })
                .map(|(events, child)| RecorderProcess::Sidecar { events, child }),
            CommandSource::External(path) => {
                let mut command = Command::new(path);
                command.args(args);
                hide_console_window(&mut command);
                command
                    .spawn()
                    .map_err(|error| {
                        format!("Could not start the configured yt-dlp executable: {error}")
                    })
                    .and_then(|child| {
                        if child.id().is_some() {
                            Ok(RecorderProcess::External(Box::new(child)))
                        } else {
                            Err("yt-dlp did not return a process id".to_owned())
                        }
                    })
            }
        };
        let process = match spawned {
            Ok(process) => process,
            Err(error) => {
                let _ = self.database.finish_job(&job.id, "Failed", &error, None);
                return Err(error);
            }
        };
        let pid = process.pid();
        self.database.set_job_pid(&job.id, pid)?;
        self.register_active(
            &job,
            &target,
            pid,
            cancellation.clone(),
            partial_output.clone(),
        )?;
        let engine = self.clone();
        tauri::async_runtime::spawn(async move {
            let outcome = process.wait(cancellation).await;
            engine
                .finish_recording(
                    &job,
                    &target.id,
                    outcome,
                    &output_path_receipt,
                    &partial_output,
                )
                .await;
        });
        self.emit_change();
        Ok(())
    }

    fn register_active(
        &self,
        job: &RecordingJob,
        target: &WatchTarget,
        pid: u32,
        cancellation: CancellationToken,
        output: PartialOutput,
    ) -> Result<(), String> {
        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| "Engine lock poisoned".to_owned())?;
        runtime.starting_targets.remove(&target.id);
        runtime.active_jobs.insert(
            job.id.clone(),
            ActiveJob {
                target_id: target.id.clone(),
                pid,
                cancellation,
                output,
            },
        );
        drop(runtime);
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
        partial_output: &PartialOutput,
    ) {
        let still_tracked = self
            .runtime
            .lock()
            .map(|runtime| runtime.active_jobs.contains_key(&job.id))
            .unwrap_or(false);
        if !still_tracked {
            // `shutdown` already finalised this job.
            return;
        }
        let state = if outcome.starts_with("Recording completed") {
            "Completed"
        } else if outcome.starts_with("Cancelled") {
            "Cancelled"
        } else {
            "Failed"
        };
        let output_path = take_output_path_receipt(output_path_receipt)
            .or_else(|| {
                recover_partial_output(&partial_output.directory, &partial_output.timestamp)
            })
            .map(|path| path.to_string_lossy().into_owned());
        let _ = self
            .database
            .finish_job(&job.id, state, &outcome, output_path.as_deref());
        let _ = self.database.target_recording_finished(target_id, &outcome);
        // Released only after the database reflects the finished job, so a
        // concurrent check cannot start a recording whose status is then
        // overwritten above.
        if let Ok(mut runtime) = self.runtime.lock() {
            runtime.active_jobs.remove(&job.id);
        }
        self.emit_change();
        self.drain_queue();

        let recorded_for = parse_job_time(&job.started_at).map(|started| now() - started);
        if state != "Cancelled"
            && recorded_for.is_some_and(|elapsed| elapsed >= RESUME_MIN_RECORDING)
        {
            let engine = self.clone();
            let target_id = target_id.to_owned();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(RESUME_CHECK_DELAY).await;
                engine.check_soon(&target_id);
            });
        }
    }

    fn drain_queue(&self) {
        let Ok(settings) = self.database.settings() else {
            return;
        };
        let target_ids = {
            let mut runtime = match self.runtime.lock() {
                Ok(runtime) => runtime,
                Err(_) => return,
            };
            if !runtime.running {
                return;
            }
            let available = settings
                .max_concurrent_recordings
                .max(1)
                .saturating_sub(runtime.active_jobs.len() + runtime.starting_targets.len());
            (0..available)
                .filter_map(|_| runtime.queued_targets.pop_front())
                .collect::<Vec<_>>()
        };
        for target_id in target_ids {
            let Ok(Some(target)) = self.database.target(&target_id) else {
                continue;
            };
            if target.enabled {
                self.spawn_probe(target);
            }
        }
    }

    fn spawn_probe(&self, target: WatchTarget) {
        let engine = self.clone();
        let future: Pin<Box<dyn Future<Output = ()> + Send>> =
            Box::pin(async move { engine.probe_target(target).await });
        tauri::async_runtime::spawn(future);
    }

    fn reserve_probe(&self, target_id: &str) -> bool {
        self.runtime
            .lock()
            .map(|mut runtime| {
                let busy = runtime.probing_targets.contains(target_id)
                    || runtime.starting_targets.contains(target_id)
                    || runtime
                        .queued_targets
                        .iter()
                        .any(|queued| queued == target_id)
                    || runtime
                        .active_jobs
                        .values()
                        .any(|job| job.target_id == target_id);
                if busy {
                    false
                } else {
                    runtime.probing_targets.insert(target_id.to_owned());
                    true
                }
            })
            .unwrap_or(false)
    }

    fn is_suppressed(&self, target_id: &str) -> bool {
        self.runtime
            .lock()
            .map(|runtime| runtime.suppressed_targets.contains(target_id))
            .unwrap_or(false)
    }

    fn has_active_or_starting_job(&self, target_id: &str) -> bool {
        self.runtime
            .lock()
            .map(|runtime| {
                runtime.starting_targets.contains(target_id)
                    || runtime
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

enum RecorderProcess {
    Sidecar {
        events: tauri::async_runtime::Receiver<CommandEvent>,
        child: tauri_plugin_shell::process::CommandChild,
    },
    External(Box<tokio::process::Child>),
}

impl RecorderProcess {
    fn pid(&self) -> u32 {
        match self {
            Self::Sidecar { child, .. } => child.pid(),
            Self::External(child) => child.id().unwrap_or_default(),
        }
    }

    /// Waits for yt-dlp to exit or for cancellation. Cancelling terminates the
    /// whole process tree and waits for it to exit so the partial file is
    /// released before it is salvaged.
    async fn wait(self, cancellation: CancellationToken) -> String {
        let pid = self.pid();
        match self {
            Self::Sidecar { mut events, child } => {
                tokio::select! {
                    outcome = wait_for_sidecar_termination(&mut events) => outcome,
                    _ = cancellation.cancelled() => {
                        kill_process_tree(pid);
                        let _ = tokio::time::timeout(
                            PROCESS_EXIT_TIMEOUT,
                            wait_for_sidecar_termination(&mut events),
                        )
                        .await;
                        let _ = child.kill();
                        "Cancelled by the user".to_owned()
                    }
                }
            }
            Self::External(mut child) => {
                tokio::select! {
                    status = child.wait() => match status {
                        Ok(status) if status.success() => "Recording completed".to_owned(),
                        Ok(status) => format!("yt-dlp exited with code {:?}", status.code()),
                        Err(error) => format!("yt-dlp could not be monitored: {error}"),
                    },
                    _ = cancellation.cancelled() => {
                        kill_process_tree(pid);
                        let _ = child.start_kill();
                        let _ = tokio::time::timeout(PROCESS_EXIT_TIMEOUT, child.wait()).await;
                        "Cancelled by the user".to_owned()
                    }
                }
            }
        }
    }
}

async fn run_external_probe(
    path: &Path,
    args: Vec<String>,
) -> Result<Option<ProbeProcessOutput>, String> {
    let mut command = Command::new(path);
    command
        .args(args)
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true);
    hide_console_window(&mut command);
    match tokio::time::timeout(PROBE_TIMEOUT, command.output()).await {
        Ok(Ok(output)) => Ok(Some(ProbeProcessOutput {
            success: output.status.success(),
            stdout: output.stdout,
            stderr: output.stderr,
        })),
        Ok(Err(error)) => Err(format!(
            "Could not run the configured yt-dlp executable: {error}"
        )),
        Err(_) => Ok(None),
    }
}

/// Finds the file of a recording whose yt-dlp process ended without reporting
/// a final path. A stopped recorder leaves a `.part` file behind; it is renamed
/// so it shows up as a normal, playable recording.
fn recover_partial_output(directory: &Path, timestamp: &str) -> Option<PathBuf> {
    let marker = format!(" - {timestamp}.");
    let mut finished = Vec::new();
    let mut partial = Vec::new();
    for entry in std::fs::read_dir(directory).ok()?.filter_map(Result::ok) {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.contains(&marker) || !path.is_file() {
            continue;
        }
        let size = entry.metadata().map(|metadata| metadata.len()).unwrap_or(0);
        if name.ends_with(".part") {
            partial.push((size, path));
        } else if is_recording_file(&path) {
            finished.push((size, path));
        }
    }
    if let Some((_, path)) = finished.into_iter().max_by_key(|(size, _)| *size) {
        return Some(path);
    }
    let (size, part) = partial.into_iter().max_by_key(|(size, _)| *size)?;
    if size == 0 {
        return None;
    }
    let completed = salvaged_file_name(&part)?;
    if completed.exists() {
        return None;
    }
    std::fs::rename(&part, &completed).ok()?;
    Some(completed)
}

/// `name.mp4.part` becomes `name.ts` when the data is MPEG-TS (the live
/// default), otherwise `name.mp4`.
fn salvaged_file_name(part: &Path) -> Option<PathBuf> {
    let without_part = part.with_extension("");
    if is_mpeg_ts(part) {
        Some(without_part.with_extension("ts"))
    } else {
        Some(without_part)
    }
}

fn is_mpeg_ts(path: &Path) -> bool {
    use std::io::Read;

    const PACKET: usize = 188;
    let mut header = [0_u8; PACKET * 2 + 1];
    std::fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut header))
        .is_ok()
        && header[0] == 0x47
        && header[PACKET] == 0x47
        && header[PACKET * 2] == 0x47
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
        recording_output_template, recording_timestamp_format, recover_partial_output,
        take_output_path_receipt, OutputCandidate,
    };
    use crate::domain::{RecordingJob, RecordingState, TargetState, WatchTarget};

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
            state: TargetState::Watching,
            status_detail: String::new(),
            next_check_at: None,
            last_checked_at: None,
            last_recording_at: None,
            active_job_id: None,
            created_at: "2026-07-13T00:00:00Z".to_owned(),
            provider_user_id: None,
            avatar_url: None,
        };
        let job = RecordingJob {
            id: "job-1".to_owned(),
            target_id: target.id.clone(),
            target_name: target.name.clone(),
            target_avatar_url: None,
            state: RecordingState::Completed,
            started_at: "2026-07-13T21:42:15Z".to_owned(),
            finished_at: Some("2026-07-13T22:00:00Z".to_owned()),
            output_path: None,
            file_exists: false,
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

    fn temporary_directory(label: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("live-downloader-{label}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).expect("temp directory should be created");
        directory
    }

    #[test]
    fn salvages_a_stopped_mpeg_ts_recording_as_a_ts_file() {
        let directory = temporary_directory("salvage-ts");
        let timestamp = "07-13-2026 22-42-15";
        let part = directory.join(format!("channel - {timestamp}.mp4.part"));
        let mut packets = vec![0_u8; 188 * 3];
        for offset in [0, 188, 376] {
            packets[offset] = 0x47;
        }
        std::fs::write(&part, packets).expect("partial file should be created");
        std::fs::write(directory.join("other - 01-01-2026 00-00-00.mp4.part"), b"x")
            .expect("unrelated file should be created");

        let recovered = recover_partial_output(&directory, timestamp);

        let expected = directory.join(format!("channel - {timestamp}.ts"));
        assert_eq!(recovered, Some(expected.clone()));
        assert!(expected.is_file());
        assert!(!part.exists());
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn salvage_prefers_a_finished_file_and_ignores_empty_parts() {
        let directory = temporary_directory("salvage-finished");
        let timestamp = "07-13-2026 22-42-15";
        let finished = directory.join(format!("channel - {timestamp}.mp4"));
        std::fs::write(&finished, b"done").expect("finished file should be created");
        std::fs::write(
            directory.join(format!("channel - {timestamp}.mp4.part")),
            b"partial",
        )
        .expect("partial file should be created");
        assert_eq!(
            recover_partial_output(&directory, timestamp),
            Some(finished.clone())
        );

        std::fs::remove_file(&finished).expect("finished file should be removed");
        let empty_timestamp = "07-14-2026 10-00-00";
        std::fs::write(
            directory.join(format!("channel - {empty_timestamp}.mp4.part")),
            b"",
        )
        .expect("empty partial file should be created");
        assert_eq!(recover_partial_output(&directory, empty_timestamp), None);
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn salvaged_non_ts_partial_keeps_its_container_extension() {
        let directory = temporary_directory("salvage-mp4");
        let timestamp = "07-13-2026 22-42-15";
        std::fs::write(
            directory.join(format!("channel - {timestamp}.mp4.part")),
            b"not an mpeg-ts stream",
        )
        .expect("partial file should be created");
        assert_eq!(
            recover_partial_output(&directory, timestamp),
            Some(directory.join(format!("channel - {timestamp}.mp4")))
        );
        let _ = std::fs::remove_dir_all(directory);
    }
}
