use anyhow::Result;

use crate::models::Tag;

use super::Database;

impl Database {
    // Add a tag to a run
    pub fn add_tag(&self, run_id: i64, tag: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO tags (run_id, tag) VALUES (?1, ?2)",
            rusqlite::params![run_id, tag],
        )?;

        Ok(())
    }

    /// Remove a tag from a run
    pub fn remove_tag(&self, run_id: i64, tag: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM tags WHERE run_id = ?1 AND tag = ?2",
            rusqlite::params![run_id, tag],
        )?;

        Ok(())
    }

    /// get all tags for a run
    pub fn get_tags_for_run(&self, run_id: i64) -> Result<Vec<Tag>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, run_id, tag FROM tags WHERE run_id = ?1 ORDER BY tag")?;

        let tags = stmt
            .query_map(rusqlite::params![run_id], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    run_id: row.get(1)?,
                    tag: row.get(2)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(tags)
    }

    /// Get all unique tags across all runs
    pub fn get_all_tags(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT tag FROM tags ORDER BY tag")?;

        let tags = stmt
            .query_map([], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(tags)
    }

    /// find runs with a specific tag
    pub fn get_runs_by_tag(&self, tag: &str) -> Result<Vec<i64>> {
        let mut stmt = self
            .conn
            .prepare("SELECT run_id FROM tags WHERE tag = ?1")?;

        let ids = stmt
            .query_map(rusqlite::params![tag], |row| row.get(0))?
            .collect::<std::result::Result<Vec<i64>, _>>()?;

        Ok(ids)
    }
}
