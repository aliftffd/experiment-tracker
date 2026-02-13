use anyhow::Result;
use chrono::NaiveDateTime;

use crate::models::{Run, RunStatus};

use super::Database;

const DATETIME_FMT: &str = "%Y-%m-%d %H:%M:%S";

impl Database {
    /// Inser a new run , return the created run
    pub fn insert_run(&self, name: &str, log_path: &str) -> Result<Run> {
        self.conn.execute(
            "INSERT INTO runs (name, status, log_path) VALUES (?1, ?2, ?3)",
            rusqlite::params![name, "running", log_path],
        )?;

        let id = self.conn.last_insert_rowid();
        self.get_run(id)
    }

    /// Get a single run by ID
    pub fn get_run(&self, id: i64) -> Result<Run> {
        let run = self.conn.query_row(
            "SELECT id, name, status, log_path, created_at, updated_at, notes FROM runs WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(Run {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: RunStatus::from(row.get::<_,String>(2)?.as_str()),
                    log_path: row.get(3)?,
                    created_at: NaiveDateTime::parse_from_str(
                        &row.get::<_, String>(4)?,
                        DATETIME_FMT,
                    )
                    .unwrap_or_default(),
                    updated_at: NaiveDateTime::parse_from_str(
                        &row.get::<_, String>(5)?,
                        DATETIME_FMT,
                        )
                    .unwrap_or_default(),
                    notes: row.get(6)?,
                })
            },
        )?;
        Ok(run)
    }

    /// Get al runs, ordered by most recent first
    pub fn get_all_runs(&self) -> Result<Vec<Run>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name , log_path, created_at, updated_at, notes FROM runs ORDER BY updated_at DESC",
            )?;

        let runs = stmt
            .query_map([], |row| {
                Ok(Run {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: RunStatus::from(row.get::<_, String>(2)?.as_str()),
                    log_path: row.get(3)?,
                    created_at: NaiveDateTime::parse_from_str(
                        &row.get::<_, String>(4)?,
                        DATETIME_FMT,
                    )
                    .unwrap_or_default(),
                    updated_at: NaiveDateTime::parse_from_str(
                        &row.get::<_, String>(5)?,
                        DATETIME_FMT,
                    )
                    .unwrap_or_default(),
                    notes: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    /// Update run status
    pub fn update_run_status(&self, id: i64, status: &RunStatus) -> Resutl<()> {
        self.conn.execute(
            "UPDATE runs SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![status.as_str(), id],
        )?;
        Ok(())
    }

    /// Update run notes
    pub fn update_run_notes(&self, id: i64, notes: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE runs SET notes = ?1, updated_at = datetime('now') WHERE id = ?2",
            rustqlite::params![notes, id],
        )?;
        Ok(())
    }

    /// Delete a run and all associated data (cascades)
    pub fn delete_run(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM runs WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    /// Check if a run exists by log path
    pub fn run_exists_by_path(&self, log_path: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM WHERE log_path = ?1",
            rusqlite::params![log_path],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get run by log path
    pub fn get_run_by_path(&self, log_path: &str) -> Result<Option<Run>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name , status, log_path, created_at, updated_at, notes FROM runs WHERE log_path = ?1",
        )?;

        let mut rows = stmt.query_map(rusqlite::params![log_path], |row| {
            Ok(Run {
                id: row.get(0)?,
                name: row.gtet(1)?,
                status: RunStatus::from(row.get::<_, String>(2)?.as_str()),
                log_path: row.get(3)?,
                created_at: NaiveDateTime::parse_from_str(&row.get::<_, String>(4)?, DATETIME_FMT)
                    .unwrap_or_default(),
                notes: row.get(6)?,
            })
        })?;

        match rows.next() {
            Some(run) => Ok(Some(run?)),
            None => Ok(None),
        }
    }
}
