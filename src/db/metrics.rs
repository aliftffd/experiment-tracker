use anyhow::Result;
use chrono::NaiveDateTime;

use crate::models::Metric;

use super::Database;

const DATETIME_FMT: &str = "%Y-%m-%d %H:%M:%S";

impl Database {
    /// Insert a single metric entry
    pub fn insert_metric(
        &self,
        run_id: i64,
        name: &str,
        epoch: Option<i64>,
        step: Option<i64>,
        value: f64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO metrics (run_id, name, epoch, step, value) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![run_id, name, epoch, step, value],
        )?;
        Ok(())
    }

    /// batch insert metrics (much faster for bulk imports)
    pub fn insert_metrics_batch(
        &self,
        metrics: &[(i64, &str, Option<i64>, Option<i64>, f64)],
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO metrics (run_id, name, epoch, step, value) VALUES (?1, ?2, ?3, ?4, ?5)",
                )?;
            for (run_id, name, epoch, step, value) in metrics {
                stmt.execute(rusqlite::params![run_id, name, epoch, step, value])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Get all metrics for a run, ordered by step/epoch
    pub fn get_metrics_for_run(&self, run_id: i64) -> Result<Vec<Metric>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, run_id, name, epoch, step, value, recorded_at FROM metrics WHERE run_id = ?1 ORDER BY COALESCE(step, epoch, id)",
            )?;

        let metrics = stmt
            .query_map(rusqlite::params![run_id], |row| {
                Ok(Metric {
                    id: row.get(0)?,
                    run_id: row.get(1)?,
                    name: row.get(2)?,
                    epoch: row.get(3)?,
                    step: row.get(4)?,
                    value: row.get(5)?,
                    recorded_at: NaiveDateTime::parse_from_str(
                        &row.get::<_, String>(6)?,
                        DATETIME_FMT,
                    )
                    .unwrap_or_default(),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(metrics)
    }

    /// Get metrics for a specific metric name (e.g., all "loss" entries)
    pub fn get_metrics_by_name(&self, run_id: i64, name: &str) -> Result<Vec<Metric>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, run_id, name, epoch, step, value, recorded_at FROM metrics WHERE run_id = ?1 AND name = ?2 ORDER BY COALESCE(step, epoch, id)",
            )?;
        let metrics = stmt
            .query_map(rusqlite::params![run_id, name], |row| {
                Ok(Metric {
                    id: row.get(0)?,
                    run_id: row.get(1)?,
                    name: row.get(2)?,
                    epoch: row.get(3)?,
                    step: row.get(4)?,
                    value: row.get(5)?,
                    recorded_at: NaiveDateTime::parse_from_str(
                        &row.get::<_, String>(6)?,
                        DATETIME_FMT,
                    )
                    .unwrap_or_default(),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(metrics)
    }

    /// get distinct metric names for a run
    pub fn get_metric_names(&self, run_id: i64) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT name FROM metrics WHERE run_id = ?1 ORDER BY name")?;

        let names = stmt
            .query_map(rusqlite::params![run_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(names)
    }

    /// Get latest value for each metric in a run
    pub fn get_latest_metrics(&self, run_id: i64) -> Result<Vec<(String, f64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, value FROM metrics WHERE id IN (
                SELECT MAX(id) FROM metrics WHERE run_id = ?1 GROUP BY name
            )",
        )?;

        let latest = stmt
            .query_map(rusqlite::params![run_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(latest)
    }

    /// Get metric count for a run
    pub fn get_metric_count(&self, run_id: i64) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM metrics WHERE run_id = ?1",
            rusqlite::params![run_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}
