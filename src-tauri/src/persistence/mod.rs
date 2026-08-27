use std::{path::Path, sync::Mutex};

use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::domain::{format_time, now, AppSettings, RecordingJob, WatchTarget};

mod migrations;

pub struct Database {
    connection: Mutex<Connection>,
}

fn map_target(row: &rusqlite::Row<'_>) -> rusqlite::Result<WatchTarget> {
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
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        let mut connection = Connection::open(path).map_err(|error| error.to_string())?;
        migrations::run(&mut connection)?;

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
                 FROM watch_targets
                 WHERE deleted_at IS NULL
                 ORDER BY enabled DESC, name COLLATE NOCASE",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], map_target)
            .map_err(|error| error.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn enabled_targets(&self) -> Result<Vec<WatchTarget>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, url, enabled, state, status_detail, next_check_at,
                    last_checked_at, last_recording_at, active_job_id, created_at
             FROM watch_targets
             WHERE enabled = 1 AND deleted_at IS NULL
             ORDER BY name COLLATE NOCASE",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], map_target)
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn target(&self, id: &str) -> Result<Option<WatchTarget>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        connection
            .query_row(
                "SELECT id, name, url, enabled, state, status_detail, next_check_at,
                    last_checked_at, last_recording_at, active_job_id, created_at
             FROM watch_targets WHERE id = ?1 AND deleted_at IS NULL",
                params![id],
                map_target,
            )
            .optional()
            .map_err(|error| error.to_string())
    }

    pub fn insert_target(&self, name: &str, url: &str) -> Result<WatchTarget, String> {
        let id = Uuid::new_v4().to_string();
        let timestamp = format_time(now());
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        let existing: Option<(String, Option<String>)> = connection
            .query_row(
                "SELECT id, deleted_at FROM watch_targets WHERE url = ?1",
                params![url],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        if let Some((existing_id, deleted_at)) = existing {
            if deleted_at.is_none() {
                return Err("That stream URL is already in the watch list.".to_owned());
            }
            connection
                .execute(
                    "UPDATE watch_targets
                 SET name = ?2, enabled = 1, state = 'Watching',
                     status_detail = 'Waiting for live stream', next_check_at = ?3,
                     active_job_id = NULL, deleted_at = NULL, updated_at = ?3
                 WHERE id = ?1",
                    params![existing_id, name, timestamp],
                )
                .map_err(|error| error.to_string())?;
            drop(connection);
            return self
                .target(&existing_id)?
                .ok_or_else(|| "Restored target was not found".to_owned());
        }
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
                "UPDATE watch_targets SET name = ?2, url = ?3, enabled = ?4, updated_at = ?5
                 WHERE id = ?1 AND deleted_at IS NULL",
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
        let timestamp = format_time(now());
        connection
            .execute(
                "UPDATE watch_targets
                 SET enabled = 0, deleted_at = ?2, active_job_id = NULL, updated_at = ?2
                 WHERE id = ?1 AND deleted_at IS NULL",
                params![id, timestamp],
            )
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
                 WHERE id = ?1 AND deleted_at IS NULL",
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
                    let output_path: Option<String> = row.get(6)?;
                    Ok(RecordingJob {
                        id: row.get(0)?,
                        target_id: row.get(1)?,
                        target_name: row.get(2)?,
                        state: row.get(3)?,
                        started_at: row.get(4)?,
                        finished_at: row.get(5)?,
                        file_exists: output_path
                            .as_deref()
                            .is_some_and(|path| Path::new(path).is_file()),
                        output_path,
                        message: row.get(7)?,
                        process_id: row.get::<_, Option<i64>>(8)?.map(|value| value as u32),
                    })
                },
            )
            .optional()
            .map_err(|error| error.to_string())
    }

    pub fn list_jobs(&self, limit: usize) -> Result<Vec<RecordingJob>, String> {
        self.list_jobs_page(limit, 0)
    }

    pub fn list_jobs_page(&self, limit: usize, offset: usize) -> Result<Vec<RecordingJob>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        let mut statement = connection
            .prepare(
                "SELECT jobs.id, jobs.target_id, targets.name, jobs.state, jobs.started_at,
                        jobs.finished_at, jobs.output_path, jobs.message, jobs.process_id
                 FROM recording_jobs jobs JOIN watch_targets targets ON targets.id = jobs.target_id
                 ORDER BY jobs.started_at DESC LIMIT ?1 OFFSET ?2",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map(params![limit as i64, offset as i64], |row| {
                let output_path: Option<String> = row.get(6)?;
                Ok(RecordingJob {
                    id: row.get(0)?,
                    target_id: row.get(1)?,
                    target_name: row.get(2)?,
                    state: row.get(3)?,
                    started_at: row.get(4)?,
                    finished_at: row.get(5)?,
                    file_exists: output_path
                        .as_deref()
                        .is_some_and(|path| Path::new(path).is_file()),
                    output_path,
                    message: row.get(7)?,
                    process_id: row.get::<_, Option<i64>>(8)?.map(|value| value as u32),
                })
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn clear_history(&self) -> Result<usize, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        connection
            .execute("DELETE FROM recording_jobs WHERE state != 'Recording'", [])
            .map_err(|error| error.to_string())
    }

    pub fn recover_interrupted_jobs(&self) -> Result<usize, String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_owned())?;
        let transaction = connection
            .transaction()
            .map_err(|error| error.to_string())?;
        let timestamp = format_time(now());
        transaction
            .execute(
                "UPDATE watch_targets
                 SET state = 'Watching', status_detail = 'Previous recording was interrupted',
                     active_job_id = NULL, next_check_at = ?1, updated_at = ?1
                 WHERE active_job_id IN (SELECT id FROM recording_jobs WHERE state = 'Recording')",
                params![timestamp],
            )
            .map_err(|error| error.to_string())?;
        let recovered = transaction
            .execute(
                "UPDATE recording_jobs
                 SET state = 'Interrupted', finished_at = ?1,
                     message = 'Recording was interrupted when the application stopped',
                     process_id = NULL
                 WHERE state = 'Recording'",
                params![timestamp],
            )
            .map_err(|error| error.to_string())?;
        transaction.commit().map_err(|error| error.to_string())?;
        Ok(recovered)
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
    use crate::domain::{Locale, RecordingState};

    fn cleanup_database(path: &std::path::Path) {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
    }

    #[test]
    fn creates_and_reopens_latest_schema() {
        let path = std::env::temp_dir().join(format!(
            "live-downloader-migration-{}.db",
            uuid::Uuid::new_v4()
        ));
        let database = Database::open(&path).expect("brand-new database should migrate");
        drop(database);

        let connection =
            rusqlite::Connection::open(&path).expect("database should reopen directly");
        let version: u32 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("version should be readable");
        assert_eq!(version, super::migrations::LATEST_SCHEMA_VERSION);
        drop(connection);

        Database::open(&path).expect("already-migrated database should reopen");
        cleanup_database(&path);
    }

    #[test]
    fn upgrades_v1_database_without_losing_targets() {
        let path =
            std::env::temp_dir().join(format!("live-downloader-v1-{}.db", uuid::Uuid::new_v4()));
        let connection = rusqlite::Connection::open(&path).expect("legacy database should open");
        connection.execute_batch(
            "CREATE TABLE settings (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL, updated_at TEXT NOT NULL);
             CREATE TABLE watch_targets (
                id TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL, url TEXT NOT NULL UNIQUE,
                enabled INTEGER NOT NULL DEFAULT 1, state TEXT NOT NULL DEFAULT 'Watching',
                status_detail TEXT NOT NULL DEFAULT 'Waiting for live stream', next_check_at TEXT,
                last_checked_at TEXT, last_recording_at TEXT, active_job_id TEXT,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
             );
             CREATE TABLE recording_jobs (
                id TEXT PRIMARY KEY NOT NULL,
                target_id TEXT NOT NULL REFERENCES watch_targets(id) ON DELETE CASCADE,
                state TEXT NOT NULL, started_at TEXT NOT NULL, finished_at TEXT,
                output_path TEXT, message TEXT NOT NULL, process_id INTEGER
             );
             CREATE INDEX idx_recording_jobs_target_started ON recording_jobs(target_id, started_at DESC);
             CREATE INDEX idx_watch_targets_enabled ON watch_targets(enabled, next_check_at);
             INSERT INTO watch_targets (id, name, url, enabled, state, status_detail, created_at, updated_at)
             VALUES ('legacy', 'Legacy', 'https://example.com/live', 1, 'Watching', 'Waiting for live stream', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');",
        ).expect("v1 schema should be created");
        drop(connection);

        let database = Database::open(&path).expect("v1 database should migrate");
        assert_eq!(
            database.list_targets().expect("targets should remain")[0].id,
            "legacy"
        );
        drop(database);
        cleanup_database(&path);
    }

    #[test]
    fn rolls_back_failed_migration() {
        let path = std::env::temp_dir().join(format!(
            "live-downloader-broken-{}.db",
            uuid::Uuid::new_v4()
        ));
        let connection = rusqlite::Connection::open(&path).expect("database should open");
        connection
            .execute_batch(
                "CREATE TABLE watch_targets (id TEXT PRIMARY KEY); PRAGMA user_version = 1;",
            )
            .expect("broken v1 schema should be created");
        drop(connection);

        assert!(Database::open(&path).is_err());
        let connection = rusqlite::Connection::open(&path).expect("database should reopen");
        let version: u32 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("version should remain readable");
        assert_eq!(version, 1);
        let has_deleted_at: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info('watch_targets') WHERE name = 'deleted_at')",
            [],
            |row| row.get(0),
        ).expect("column state should be readable");
        assert!(!has_deleted_at);
        drop(connection);
        cleanup_database(&path);
    }

    #[test]
    fn soft_delete_preserves_history_and_readding_restores_target() {
        let path = std::env::temp_dir().join(format!(
            "live-downloader-soft-delete-{}.db",
            uuid::Uuid::new_v4()
        ));
        let database = Database::open(&path).expect("database should open");
        let target = database
            .insert_target("Original", "https://example.com/live")
            .expect("target should insert");
        let job = database
            .create_job(&target.id, None)
            .expect("job should insert");
        database
            .finish_job(&job.id, "Completed", "Done", None)
            .expect("job should finish");

        database
            .remove_target(&target.id)
            .expect("target should soft delete");
        assert!(database
            .list_targets()
            .expect("targets should list")
            .is_empty());
        assert_eq!(
            database.list_jobs(10).expect("history should remain").len(),
            1
        );

        let restored = database
            .insert_target("Restored", "https://example.com/live")
            .expect("target should restore");
        assert_eq!(restored.id, target.id);
        assert_eq!(restored.name, "Restored");
        assert!(restored.enabled);
        assert_eq!(
            database.list_jobs(10).expect("history should still remain")[0].id,
            job.id
        );

        drop(database);
        cleanup_database(&path);
    }

    #[test]
    fn recovers_jobs_interrupted_by_an_abnormal_shutdown() {
        let path = std::env::temp_dir().join(format!(
            "live-downloader-recovery-{}.db",
            uuid::Uuid::new_v4()
        ));
        let database = Database::open(&path).expect("database should open");
        let target = database
            .insert_target("Recovery", "https://example.com/recovery")
            .expect("target should insert");
        let job = database
            .create_job(&target.id, None)
            .expect("job should insert");
        database
            .set_target_status(
                &target.id,
                "Recording",
                "Recording in the background",
                None,
                Some(&job.id),
            )
            .expect("target should become active");

        assert_eq!(
            database
                .recover_interrupted_jobs()
                .expect("recovery should run"),
            1
        );
        let recovered_job = database
            .job(&job.id)
            .expect("job should load")
            .expect("job should exist");
        assert_eq!(recovered_job.state, RecordingState::Interrupted);
        assert!(recovered_job.finished_at.is_some());
        assert_eq!(
            database
                .target(&target.id)
                .expect("target should load")
                .expect("target should exist")
                .active_job_id,
            None
        );

        drop(database);
        cleanup_database(&path);
    }

    #[test]
    fn persists_targets_settings_and_job_lifecycle() {
        let path =
            std::env::temp_dir().join(format!("live-downloader-test-{}.db", uuid::Uuid::new_v4()));
        let database = Database::open(&path).expect("database should open");

        let mut settings = database
            .settings()
            .expect("default settings should be written");
        assert_eq!(settings.locale, Locale::English);
        settings.locale = Locale::PortugueseBrazil;
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
            Locale::PortugueseBrazil
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
        assert_eq!(history[0].state, RecordingState::Completed);
        assert_eq!(history[0].process_id, Some(42));
        assert_eq!(history[0].output_path.as_deref(), output_path.to_str());
        assert!(!history[0].file_exists);

        std::fs::write(&output_path, b"recording").expect("output file should be created");
        assert!(database.list_jobs(10).expect("history should list")[0].file_exists);

        let active_job = database
            .create_job(&target.id, None)
            .expect("active job should start");
        assert_eq!(database.clear_history().expect("history should clear"), 1);
        let remaining = database.list_jobs(10).expect("active jobs should remain");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, active_job.id);

        drop(database);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&output_path);
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
    }

    #[test]
    fn defaults_locale_for_existing_settings_without_one() {
        let path =
            std::env::temp_dir().join(format!("live-downloader-test-{}.db", uuid::Uuid::new_v4()));
        let database = Database::open(&path).expect("database should open");
        let mut legacy_settings = serde_json::to_value(crate::domain::AppSettings::default())
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
            Locale::English
        );

        drop(database);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
    }
}
