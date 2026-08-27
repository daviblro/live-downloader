use rusqlite::Connection;

pub const LATEST_SCHEMA_VERSION: u32 = 2;

pub fn run(connection: &mut Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )
        .map_err(|error| error.to_string())?;

    let mut version: u32 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if version > LATEST_SCHEMA_VERSION {
        return Err(format!(
            "Database schema version {version} is newer than supported version {LATEST_SCHEMA_VERSION}"
        ));
    }

    if version == 0 {
        let transaction = connection
            .transaction()
            .map_err(|error| error.to_string())?;
        let has_baseline: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'watch_targets')",
                [],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if has_baseline {
            transaction
                .execute_batch("PRAGMA user_version = 1;")
                .map_err(|error| error.to_string())?;
        } else {
            transaction
                .execute_batch(
                    "CREATE TABLE settings (
                    key TEXT PRIMARY KEY NOT NULL,
                    value TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
                CREATE TABLE watch_targets (
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
                CREATE TABLE recording_jobs (
                    id TEXT PRIMARY KEY NOT NULL,
                    target_id TEXT NOT NULL REFERENCES watch_targets(id) ON DELETE CASCADE,
                    state TEXT NOT NULL,
                    started_at TEXT NOT NULL,
                    finished_at TEXT,
                    output_path TEXT,
                    message TEXT NOT NULL,
                    process_id INTEGER
                );
                CREATE INDEX idx_recording_jobs_target_started
                    ON recording_jobs(target_id, started_at DESC);
                CREATE INDEX idx_watch_targets_enabled
                    ON watch_targets(enabled, next_check_at);
                PRAGMA user_version = 1;",
                )
                .map_err(|error| error.to_string())?;
        }
        transaction.commit().map_err(|error| error.to_string())?;
        version = 1;
    }

    if version == 1 {
        let transaction = connection
            .transaction()
            .map_err(|error| error.to_string())?;
        transaction
            .execute_batch(
                "ALTER TABLE watch_targets ADD COLUMN deleted_at TEXT;
                 CREATE INDEX idx_watch_targets_active_enabled
                    ON watch_targets(deleted_at, enabled, next_check_at);
                 PRAGMA user_version = 2;",
            )
            .map_err(|error| error.to_string())?;
        transaction.commit().map_err(|error| error.to_string())?;
    }

    Ok(())
}
