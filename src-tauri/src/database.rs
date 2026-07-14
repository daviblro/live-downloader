use std::{path::Path, sync::Mutex};

use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::models::{format_time, now, AppSettings, RecordingJob, WatchTarget};

pub struct Database {
    connection: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        let connection = Connection::open(path).map_err(|error| error.to_string())?;
        connection
            .execute_batch(
                "
                PRAGMA journal_mode = WAL;
                PRAGMA foreign_keys = ON;
                PRAGMA busy_timeout = 5000;

                CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY NOT NULL,
                    value TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS watch_targets (
                    id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    url TEXT NOT NULL UNIQUE,
                    enabled INTEGER NOT NULL DEFAULT 1,
                    state TEXT NOT NULL DEFAULT 'Watching',
                    status_detail TEXT NOT NULL DEFAULT 'Waiting for live stream',
                    next_check_at TEXT,
                    last_checked_at TEXT,
                    last_recording_at TEXT,
                    active_job_id TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS recording_jobs (
                    id TEXT PRIMARY KEY NOT NULL,
                    target_id TEXT NOT NULL REFERENCES watch_targets(id) ON DELETE CASCADE,
                    state TEXT NOT NULL,
                    started_at TEXT NOT NULL,
                    finished_at TEXT,
                    output_path TEXT,
                    message TEXT NOT NULL,
                    process_id INTEGER
                );

                CREATE INDEX IF NOT EXISTS idx_recording_jobs_target_started
                    ON recording_jobs(target_id, started_at DESC);
                CREATE INDEX IF NOT EXISTS idx_watch_targets_enabled
                    ON watch_targets(enabled, next_check_at);
                ",
            )
            .map_err(|error| error.to_string())?;

        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn settings(&self) -> Result<AppSettings, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        let value: Option<String> = connection
            .query_row(
                "SELECT value FROM settings WHERE key = 'app_settings'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;

        match value {
            Some(value) => serde_json::from_str(&value).map_err(|error| error.to_string()),
            None => {
                let settings = AppSettings::default();
                drop(connection);
                self.save_settings(&settings)?;
                Ok(settings)
            }
        }
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), String> {
        let json = serde_json::to_string(settings).map_err(|error| error.to_string())?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        connection
            .execute(
                "INSERT INTO settings (key, value, updated_at) VALUES ('app_settings', ?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![json, format_time(now())],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn list_targets(&self) -> Result<Vec<WatchTarget>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, url, enabled, state, status_detail, next_check_at,
                        last_checked_at, last_recording_at, active_job_id, created_at
                 FROM watch_targets ORDER BY enabled DESC, name COLLATE NOCASE",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok(WatchTarget {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    enabled: row.get::<_, i64>(3)? != 0,
                    state: row.get(4)?,
                    status_detail: row.get(5)?,
                    next_check_at: row.get(6)?,
                    last_checked_at: row.get(7)?,
                    last_recording_at: row.get(8)?,
                    active_job_id: row.get(9)?,
                    created_at: row.get(10)?,
                })
            })
            .map_err(|error| error.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn enabled_targets(&self) -> Result<Vec<WatchTarget>, String> {
        Ok(self
            .list_targets()?
            .into_iter()
            .filter(|target| target.enabled)
            .collect())
    }

    pub fn target(&self, id: &str) -> Result<Option<WatchTarget>, String> {
        Ok(self
            .list_targets()?
            .into_iter()
            .find(|target| target.id == id))
    }

    pub fn insert_target(&self, name: &str, url: &str) -> Result<WatchTarget, String> {
        let id = Uuid::new_v4().to_string();
        let timestamp = format_time(now());
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        connection
            .execute(
                "INSERT INTO watch_targets (id, name, url, enabled, state, status_detail, next_check_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 1, 'Watching', 'Waiting for live stream', ?4, ?5, ?5)",
                params![id, name, url, timestamp, timestamp],
            )
            .map_err(|error| {
                if error.to_string().contains("UNIQUE") {
                    "That stream URL is already in the watch list.".to_owned()
                } else {
                    error.to_string()
                }
            })?;
        drop(connection);
        self.target(&id)?
            .ok_or_else(|| "Created target was not found".to_owned())
    }

    pub fn update_target(&self, target: &WatchTarget) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        connection
            .execute(
                "UPDATE watch_targets SET name = ?2, url = ?3, enabled = ?4, updated_at = ?5 WHERE id = ?1",
                params![
                    target.id,
                    target.name,
                    target.url,
                    i64::from(target.enabled),
                    format_time(now())
                ],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn remove_target(&self, id: &str) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        connection
            .execute("DELETE FROM watch_targets WHERE id = ?1", params![id])
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn set_target_status(
        &self,
        id: &str,
        state: &str,
        detail: &str,
        next_check_at: Option<&str>,
        active_job_id: Option<&str>,
    ) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        connection
            .execute(
                "UPDATE watch_targets
                 SET state = ?2, status_detail = ?3, next_check_at = ?4, last_checked_at = ?5,
                     active_job_id = ?6, updated_at = ?5
                 WHERE id = ?1",
                params![
                    id,
                    state,
                    detail,
                    next_check_at,
                    format_time(now()),
                    active_job_id
                ],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn create_job(
        &self,
        target_id: &str,
        output_path: Option<&str>,
    ) -> Result<RecordingJob, String> {
        let id = Uuid::new_v4().to_string();
        let timestamp = format_time(now());
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        connection
            .execute(
                "INSERT INTO recording_jobs (id, target_id, state, started_at, output_path, message)
                 VALUES (?1, ?2, 'Recording', ?3, ?4, 'Recording started')",
                params![id, target_id, timestamp, output_path],
            )
            .map_err(|error| error.to_string())?;
        drop(connection);
        self.job(&id)?
            .ok_or_else(|| "Created job was not found".to_owned())
    }

    pub fn job(&self, id: &str) -> Result<Option<RecordingJob>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        connection
            .query_row(
                "SELECT jobs.id, jobs.target_id, targets.name, jobs.state, jobs.started_at,
                        jobs.finished_at, jobs.output_path, jobs.message, jobs.process_id
                 FROM recording_jobs jobs JOIN watch_targets targets ON targets.id = jobs.target_id
                 WHERE jobs.id = ?1",
                params![id],
                |row| {
                    Ok(RecordingJob {
                        id: row.get(0)?,
                        target_id: row.get(1)?,
                        target_name: row.get(2)?,
                        state: row.get(3)?,
                        started_at: row.get(4)?,
                        finished_at: row.get(5)?,
                        output_path: row.get(6)?,
                        message: row.get(7)?,
                        process_id: row.get::<_, Option<i64>>(8)?.map(|value| value as u32),
                    })
                },
            )
            .optional()
            .map_err(|error| error.to_string())
    }

    pub fn list_jobs(&self, limit: usize) -> Result<Vec<RecordingJob>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        let mut statement = connection
            .prepare(
                "SELECT jobs.id, jobs.target_id, targets.name, jobs.state, jobs.started_at,
                        jobs.finished_at, jobs.output_path, jobs.message, jobs.process_id
                 FROM recording_jobs jobs JOIN watch_targets targets ON targets.id = jobs.target_id
                 ORDER BY jobs.started_at DESC LIMIT ?1",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map(params![limit as i64], |row| {
                Ok(RecordingJob {
                    id: row.get(0)?,
                    target_id: row.get(1)?,
                    target_name: row.get(2)?,
                    state: row.get(3)?,
                    started_at: row.get(4)?,
                    finished_at: row.get(5)?,
                    output_path: row.get(6)?,
                    message: row.get(7)?,
                    process_id: row.get::<_, Option<i64>>(8)?.map(|value| value as u32),
                })
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn finish_job(
        &self,
        id: &str,
        state: &str,
        message: &str,
        output_path: Option<&str>,
    ) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        connection
            .execute(
                "UPDATE recording_jobs
                 SET state = ?2, message = ?3, finished_at = ?4,
                     output_path = COALESCE(?5, output_path)
                 WHERE id = ?1",
                params![id, state, message, format_time(now()), output_path],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn set_job_output_path(&self, id: &str, output_path: &str) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        connection
            .execute(
                "UPDATE recording_jobs SET output_path = ?2 WHERE id = ?1",
                params![id, output_path],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn set_job_pid(&self, id: &str, pid: u32) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        connection
            .execute(
                "UPDATE recording_jobs SET process_id = ?2 WHERE id = ?1",
                params![id, pid],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn target_recording_finished(&self, target_id: &str, message: &str) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        let timestamp = format_time(now());
        connection
            .execute(
                "UPDATE watch_targets
                 SET state = 'Watching', status_detail = ?2, active_job_id = NULL,
                     last_recording_at = ?3, next_check_at = ?3, updated_at = ?3
                 WHERE id = ?1",
                params![target_id, message, timestamp],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Database;

    #[test]
    fn persists_targets_settings_and_job_lifecycle() {
        let path =
            std::env::temp_dir().join(format!("live-downloader-test-{}.db", uuid::Uuid::new_v4()));
        let database = Database::open(&path).expect("database should open");

        let mut settings = database
            .settings()
            .expect("default settings should be written");
        assert_eq!(settings.locale, "en");
        settings.locale = "pt-BR".to_owned();
        settings.probe_interval_seconds = 90;
        database
            .save_settings(&settings)
            .expect("settings should save");
        assert_eq!(
            database
                .settings()
                .expect("settings should load")
                .probe_interval_seconds,
            90
        );
        assert_eq!(
            database.settings().expect("settings should load").locale,
            "pt-BR"
        );

        let target = database
            .insert_target("Example", "https://www.twitch.tv/example")
            .expect("target should insert");
        assert_eq!(
            database.list_targets().expect("targets should list").len(),
            1
        );
        assert!(database
            .insert_target("Duplicate", "https://www.twitch.tv/example")
            .is_err());

        let job = database
            .create_job(&target.id, None)
            .expect("job should start");
        database
            .set_job_pid(&job.id, 42)
            .expect("pid should persist");
        let output_path = path.with_extension("mp4");
        database
            .finish_job(
                &job.id,
                "Completed",
                "Recording completed",
                output_path.to_str(),
            )
            .expect("job should finish");
        let history = database.list_jobs(10).expect("history should list");
        assert_eq!(history[0].state, "Completed");
        assert_eq!(history[0].process_id, Some(42));
        assert_eq!(history[0].output_path.as_deref(), output_path.to_str());

        drop(database);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
    }

    #[test]
    fn defaults_locale_for_existing_settings_without_one() {
        let path =
            std::env::temp_dir().join(format!("live-downloader-test-{}.db", uuid::Uuid::new_v4()));
        let database = Database::open(&path).expect("database should open");
        let mut legacy_settings = serde_json::to_value(crate::models::AppSettings::default())
            .expect("default settings should serialize");
        legacy_settings
            .as_object_mut()
            .expect("settings should be an object")
            .remove("locale");
        let legacy_json =
            serde_json::to_string(&legacy_settings).expect("legacy settings should serialize");
        database
            .connection
            .lock()
            .expect("database should unlock")
            .execute(
                "INSERT INTO settings (key, value, updated_at) VALUES ('app_settings', ?1, ?2)",
                rusqlite::params![legacy_json, "2026-07-10T00:00:00Z"],
            )
            .expect("legacy settings should insert");

        assert_eq!(
            database
                .settings()
                .expect("legacy settings should load")
                .locale,
            "en"
        );

        drop(database);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
    }
}
