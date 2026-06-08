//! SQLite-backed link storage.
//!
//! Manages bidirectional links, bookmarks, and purple-number mappings.
//!
//! # Robustness
//!
//! - **WAL mode**: enables concurrent reads and one writer without lock errors.
//! - **busy_timeout**: retries on lock contention for up to 5 seconds.
//! - **Schema versioning**: the `schema_version` table lets future migrations
//!   detect and reject incompatible databases gracefully.
//! - **Duplicate links**: `create_link` returns `Error::LinkAlreadyExists`
//!   instead of exposing a raw UNIQUE constraint violation.

use crate::error::{Error, Result};
use rusqlite::Connection;
use std::path::Path;

/// Current schema version. Increment when making breaking schema changes.
const SCHEMA_VERSION: u32 = 1;

/// A bidirectional link between two resources
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    /// Open or create a LinkStore at the given path.
    ///
    /// Applies WAL journal mode, busy_timeout, and runs schema migrations.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Retry on lock for up to 5 seconds instead of failing immediately
        conn.busy_timeout(std::time::Duration::from_secs(5))?;

        // WAL mode: allows concurrent reads + one writer without lock errors
        conn.pragma_update(None, "journal_mode", "WAL")?;

        // Enforce foreign key constraints
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let store = LinkStore { conn };
        store.run_migrations()?;
        Ok(store)
    }

    fn run_migrations(&self) -> Result<()> {
        // Ensure the version table exists before reading it
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL
            );",
        )?;

        let current_version: Option<u32> = self
            .conn
            .query_row(
                "SELECT version FROM schema_version LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();

        match current_version {
            None => {
                // Fresh database — apply initial schema and record version
                self.apply_v1_schema()?;
                self.conn.execute(
                    "INSERT INTO schema_version (version) VALUES (?)",
                    [SCHEMA_VERSION],
                )?;
            }
            Some(v) if v == SCHEMA_VERSION => {
                // Up to date — nothing to do
            }
            Some(v) if v < SCHEMA_VERSION => {
                // Future: apply incremental migrations here
                // For now, v1 is the only version so this path is unreachable
                let _ = v;
            }
            Some(v) => {
                // Database was written by a newer version of hitchmark
                return Err(Error::SchemaTooNew {
                    found: v,
                    supported: SCHEMA_VERSION,
                });
            }
        }

        Ok(())
    }

    fn apply_v1_schema(&self) -> Result<()> {
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

    /// Create a bidirectional link.
    ///
    /// Returns `Error::LinkAlreadyExists` if the pair is already linked.
    pub fn create_link(&self, source: &str, target: &str, note: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let result = self.conn.execute(
            "INSERT INTO links (source, target, note, created_at) VALUES (?, ?, ?, ?)",
            rusqlite::params![source, target, note, now],
        );

        match result {
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(Error::LinkAlreadyExists {
                    uri_a: source.to_string(),
                    uri_b: target.to_string(),
                })
            }
            Err(e) => Err(Error::DatabaseError(e)),
        }
    }

    /// List all links for a URI (bidirectional).
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

    /// Delete a link (removes regardless of which end is source/target).
    pub fn delete_link(&self, source: &str, target: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM links WHERE (source = ? AND target = ?) OR (source = ? AND target = ?)",
            rusqlite::params![source, target, target, source],
        )?;
        Ok(())
    }

    /// Return the number of links in the store (useful for diagnostics).
    pub fn link_count(&self) -> Result<u64> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM links", [], |row| row.get(0))?;
        Ok(count as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_store() -> (TempDir, LinkStore) {
        let tmpdir = TempDir::new().unwrap();
        let store_path = tmpdir.path().join("test.db");
        let store = LinkStore::open(&store_path).unwrap();
        (tmpdir, store)
    }

    #[test]
    fn test_store_roundtrip() {
        let (_dir, store) = make_store();
        store
            .create_link("hook://file/a", "hook://file/b", Some("test note"))
            .unwrap();

        let links = store.list_links("hook://file/a").unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].note, Some("test note".to_string()));
    }

    #[test]
    fn test_bidirectional_query() {
        let (_dir, store) = make_store();
        store
            .create_link("hook://file/a", "hook://file/b", None)
            .unwrap();

        // Query from either end should find the link
        assert_eq!(store.list_links("hook://file/a").unwrap().len(), 1);
        assert_eq!(store.list_links("hook://file/b").unwrap().len(), 1);
        assert_eq!(store.list_links("hook://file/c").unwrap().len(), 0);
    }

    #[test]
    fn test_duplicate_link_returns_friendly_error() {
        let (_dir, store) = make_store();
        store
            .create_link("hook://file/a", "hook://file/b", None)
            .unwrap();

        let result = store.create_link("hook://file/a", "hook://file/b", None);
        assert!(matches!(result, Err(Error::LinkAlreadyExists { .. })));

        // The error message should be human-readable
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("already exists"));
    }

    #[test]
    fn test_delete_link() {
        let (_dir, store) = make_store();
        store
            .create_link("hook://file/a", "hook://file/b", None)
            .unwrap();
        assert_eq!(store.list_links("hook://file/a").unwrap().len(), 1);

        store
            .delete_link("hook://file/a", "hook://file/b")
            .unwrap();
        assert_eq!(store.list_links("hook://file/a").unwrap().len(), 0);
    }

    #[test]
    fn test_delete_link_symmetric() {
        // delete_link should work regardless of argument order
        let (_dir, store) = make_store();
        store
            .create_link("hook://file/a", "hook://file/b", None)
            .unwrap();

        store
            .delete_link("hook://file/b", "hook://file/a") // reversed
            .unwrap();
        assert_eq!(store.list_links("hook://file/a").unwrap().len(), 0);
    }

    #[test]
    fn test_link_count() {
        let (_dir, store) = make_store();
        assert_eq!(store.link_count().unwrap(), 0);
        store
            .create_link("hook://file/a", "hook://file/b", None)
            .unwrap();
        store
            .create_link("hook://file/a", "hook://file/c", None)
            .unwrap();
        assert_eq!(store.link_count().unwrap(), 2);
    }

    #[test]
    fn test_schema_version_persists() {
        let tmpdir = TempDir::new().unwrap();
        let store_path = tmpdir.path().join("versioned.db");

        // Open once — creates schema
        {
            let _store = LinkStore::open(&store_path).unwrap();
        }

        // Open again — should detect existing schema and not re-run migrations
        let store = LinkStore::open(&store_path).unwrap();
        assert_eq!(store.link_count().unwrap(), 0);
    }

    #[test]
    fn test_schema_too_new_rejected() {
        let tmpdir = TempDir::new().unwrap();
        let store_path = tmpdir.path().join("future.db");

        // Manually write a future schema version
        {
            let conn = Connection::open(&store_path).unwrap();
            conn.execute_batch(
                "CREATE TABLE schema_version (version INTEGER NOT NULL);
                 INSERT INTO schema_version VALUES (9999);",
            )
            .unwrap();
        }

        let result = LinkStore::open(&store_path);
        assert!(matches!(result, Err(Error::SchemaTooNew { .. })));
    }
}
