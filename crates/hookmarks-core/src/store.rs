//! SQLite-backed link storage.
//!
//! Manages bidirectional links, bookmarks, and purple-number mappings.

use crate::error::Result;
use rusqlite::Connection;
use std::path::Path;

/// A bidirectional link between two resources
#[derive(Debug, Clone)]
pub struct Link {
    /// Source URI
    pub source: String,
    /// Target URI
    pub target: String,
    /// Optional note
    pub note: Option<String>,
    /// Created timestamp (RFC 3339)
    pub created_at: String,
}

/// SQLite-backed store for links and metadata
pub struct LinkStore {
    conn: Connection,
}

impl LinkStore {
    /// Open or create a LinkStore at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let store = LinkStore { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS links (
                id INTEGER PRIMARY KEY,
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                note TEXT,
                created_at TEXT NOT NULL,
                UNIQUE(source, target)
            );

            CREATE TABLE IF NOT EXISTS purple_numbers (
                id INTEGER PRIMARY KEY,
                doc_path TEXT NOT NULL,
                para_id TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                UNIQUE(doc_path, para_id)
            );

            CREATE INDEX IF NOT EXISTS idx_links_source ON links(source);
            CREATE INDEX IF NOT EXISTS idx_links_target ON links(target);
            CREATE INDEX IF NOT EXISTS idx_purple_doc ON purple_numbers(doc_path);
            "#,
        )?;
        Ok(())
    }

    /// Create a bidirectional link
    pub fn create_link(&self, source: &str, target: &str, note: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO links (source, target, note, created_at) VALUES (?, ?, ?, ?)",
            rusqlite::params![source, target, note, now],
        )?;
        Ok(())
    }

    /// List all links for a URI
    pub fn list_links(&self, uri: &str) -> Result<Vec<Link>> {
        let mut stmt = self.conn.prepare(
            "SELECT source, target, note, created_at FROM links WHERE source = ? OR target = ?",
        )?;
        let links = stmt.query_map(rusqlite::params![uri, uri], |row| {
            Ok(Link {
                source: row.get(0)?,
                target: row.get(1)?,
                note: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        let mut result = Vec::new();
        for link in links {
            result.push(link?);
        }
        Ok(result)
    }

    /// Delete a link
    pub fn delete_link(&self, source: &str, target: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM links WHERE (source = ? AND target = ?) OR (source = ? AND target = ?)",
            rusqlite::params![source, target, target, source],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_store_roundtrip() {
        let tmpdir = TempDir::new().unwrap();
        let store_path = tmpdir.path().join("test.db");

        let store = LinkStore::open(&store_path).unwrap();
        store
            .create_link("hook://file/a", "hook://file/b", Some("test note"))
            .unwrap();

        let links = store.list_links("hook://file/a").unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].note, Some("test note".to_string()));
    }
}
