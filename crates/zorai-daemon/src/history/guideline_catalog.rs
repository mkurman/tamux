use super::*;
use rusqlite::params;

#[derive(Debug, Clone)]
pub(crate) struct GuidelineDocumentRecord {
    pub relative_path: String,
    pub excerpt: String,
    pub last_seen_at: i64,
    pub updated_at: i64,
}

impl HistoryStore {
    pub(crate) async fn register_guideline_document(
        &self,
        relative_path: &str,
        excerpt: &str,
        now_ms: i64,
    ) -> Result<()> {
        let relative_path = relative_path.to_string();
        let excerpt = excerpt.to_string();
        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO discoverable_guideline_documents
                        (relative_path, excerpt, last_seen_at, updated_at)
                     VALUES (?1, ?2, ?3, ?3)
                     ON CONFLICT(relative_path) DO UPDATE SET
                        excerpt = excluded.excerpt,
                        last_seen_at = excluded.last_seen_at,
                        updated_at = excluded.updated_at",
                    params![relative_path, excerpt, now_ms],
                )?;
                Ok(())
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub(crate) async fn list_discoverable_guideline_documents(
        &self,
        limit: usize,
    ) -> Result<Vec<GuidelineDocumentRecord>> {
        let limit = limit.clamp(1, 4000) as i64;
        self.interactive_read_conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT relative_path, excerpt, last_seen_at, updated_at
                     FROM discoverable_guideline_documents
                     ORDER BY last_seen_at DESC, relative_path ASC
                     LIMIT ?1",
                )?;
                let rows = stmt.query_map(params![limit], |row| {
                    Ok(GuidelineDocumentRecord {
                        relative_path: row.get(0)?,
                        excerpt: row.get(1)?,
                        last_seen_at: row.get(2)?,
                        updated_at: row.get(3)?,
                    })
                })?;
                Ok(rows.filter_map(|row| row.ok()).collect())
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub(crate) async fn prune_stale_guideline_documents(&self, cutoff_ms: i64) -> Result<usize> {
        self.conn
            .call(move |conn| {
                let removed = conn.execute(
                    "DELETE FROM discoverable_guideline_documents WHERE last_seen_at < ?1",
                    params![cutoff_ms],
                )?;
                Ok(removed)
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}
