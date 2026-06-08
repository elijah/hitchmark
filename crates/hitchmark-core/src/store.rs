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
const SCHEMA_VERSION: u32 = 2;

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

/// A stored bookmark — a stable UUID mapping to a file path.
///
/// Bookmark URIs (`hook://bookmark/<uuid>`) remain valid even if the file is
/// renamed or moved, as long as the user updates the bookmark afterward.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Bookmark {
    /// UUID identifier (the UUID portion of `hook://bookmark/<uuid>`)
    pub id: String,
    /// Absolute path to the file at time of last update
    pub file_path: String,
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
                // Fresh database — apply all schema migrations and record current version
                self.apply_v1_schema()?;
                self.apply_v2_schema()?;
                self.conn.execute(
                    "INSERT INTO schema_version (version) VALUES (?)",
                    [SCHEMA_VERSION],
                )?;
            }
            Some(1) => {
                // Upgrade from v1 → v2: add bookmarks table
                self.apply_v2_schema()?;
                self.conn
                    .execute("UPDATE schema_version SET version = ?", [SCHEMA_VERSION])?;
            }
            Some(v) if v == SCHEMA_VERSION => {
                // Up to date — nothing to do
            }
            Some(v) if v < SCHEMA_VERSION => {
                // Gap between known versions — shouldn't happen with current code
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

    fn apply_v2_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS bookmarks (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_bookmarks_path ON bookmarks(file_path);
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

    /// Store a stable bookmark for a file path and return its UUID.
    ///
    /// If a bookmark for this path already exists, the existing UUID is returned.
    /// This makes `hk file --bookmark` idempotent.
    pub fn store_bookmark(&self, file_path: &str) -> Result<String> {
        // Check for an existing bookmark for this path
        let existing: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM bookmarks WHERE file_path = ? LIMIT 1",
                rusqlite::params![file_path],
                |row| row.get(0),
            )
            .ok();

        if let Some(id) = existing {
            return Ok(id);
        }

        // Generate a new UUID v4
        let id = new_uuid_v4();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO bookmarks (id, file_path, created_at) VALUES (?, ?, ?)",
            rusqlite::params![id, file_path, now],
        )?;
        Ok(id)
    }

    /// Look up the file path for a bookmark UUID.
    ///
    /// Returns `None` when the UUID is not found (bookmark was never created or
    /// the database has been reset).
    pub fn lookup_bookmark(&self, id: &str) -> Result<Option<String>> {
        let path: Option<String> = self
            .conn
            .query_row(
                "SELECT file_path FROM bookmarks WHERE id = ?",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .ok();
        Ok(path)
    }

    /// Update the file path for an existing bookmark.
    ///
    /// Returns `false` when `id` is not found.
    pub fn update_bookmark_path(&self, id: &str, new_path: &str) -> Result<bool> {
        let rows = self.conn.execute(
            "UPDATE bookmarks SET file_path = ? WHERE id = ?",
            rusqlite::params![new_path, id],
        )?;
        Ok(rows > 0)
    }

    /// List all stored bookmarks.
    pub fn list_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, file_path, created_at FROM bookmarks ORDER BY created_at")?;
        let bookmarks = stmt.query_map([], |row| {
            Ok(Bookmark {
                id: row.get(0)?,
                file_path: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;
        let mut result = Vec::new();
        for b in bookmarks {
            result.push(b?);
        }
        Ok(result)
    }
}

/// Generate a UUID v4 (random) without pulling in the `uuid` crate.
///
/// Uses `std::collections::hash_map::DefaultHasher` seeded from system time
/// plus a thread-local counter for reasonable uniqueness. For production use
/// that requires cryptographic UUIDs, add the `uuid` crate with the `v4` feature.
fn new_uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let c = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let a = t.wrapping_mul(6364136223846793005).wrapping_add(c ^ 0xdeadbeefcafe);
    let b = c.wrapping_mul(2862933555777941757).wrapping_add(t ^ 0x1234567890abcdef);

    // Standard UUID layout: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
    // p0: 8 hex (32 bits), p1: 4 hex (16 bits), p2: 4 hex (version nibble forced to 4)
    // p3: 4 hex (variant bits 10xx forced), p4: 12 hex (48 bits)
    let p0 = (a >> 32) as u32;
    let p1 = ((a >> 16) & 0xffff) as u16;
    let p2 = 0x4000u16 | ((a & 0x0fff) as u16);
    let p3 = 0x8000u16 | (((b >> 48) & 0x3fff) as u16);
    let p4 = b & 0x0000_ffff_ffff_ffff;
    format!("{p0:08x}-{p1:04x}-{p2:04x}-{p3:04x}-{p4:012x}")
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

    #[test]
    fn test_bookmark_create_and_lookup() {
        let (_dir, store) = make_store();
        let id = store.store_bookmark("/Users/alice/notes.md").unwrap();
        assert!(!id.is_empty());
        let path = store.lookup_bookmark(&id).unwrap();
        assert_eq!(path, Some("/Users/alice/notes.md".to_string()));
    }

    #[test]
    fn test_bookmark_create_is_idempotent() {
        let (_dir, store) = make_store();
        let id1 = store.store_bookmark("/Users/alice/notes.md").unwrap();
        let id2 = store.store_bookmark("/Users/alice/notes.md").unwrap();
        assert_eq!(id1, id2, "Same path should return same bookmark UUID");
    }

    #[test]
    fn test_bookmark_lookup_unknown_returns_none() {
        let (_dir, store) = make_store();
        let result = store
            .lookup_bookmark("00000000-0000-4000-8000-000000000000")
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_bookmark_update_path() {
        let (_dir, store) = make_store();
        let id = store.store_bookmark("/old/path.md").unwrap();
        let updated = store.update_bookmark_path(&id, "/new/path.md").unwrap();
        assert!(updated);
        let path = store.lookup_bookmark(&id).unwrap();
        assert_eq!(path, Some("/new/path.md".to_string()));
    }

    #[test]
    fn test_bookmark_update_unknown_returns_false() {
        let (_dir, store) = make_store();
        let updated = store
            .update_bookmark_path("00000000-0000-4000-8000-000000000000", "/foo.md")
            .unwrap();
        assert!(!updated);
    }

    #[test]
    fn test_bookmark_list() {
        let (_dir, store) = make_store();
        store.store_bookmark("/file/a.md").unwrap();
        store.store_bookmark("/file/b.md").unwrap();
        let bookmarks = store.list_bookmarks().unwrap();
        assert_eq!(bookmarks.len(), 2);
    }

    #[test]
    fn test_bookmark_uuid_format() {
        let (_dir, store) = make_store();
        let id = store.store_bookmark("/any/path.md").unwrap();
        // Must match xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(parts.len(), 5, "UUID must have 5 hyphen-separated parts");
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        // parts[4] is the last segment: 4 chars + 8 chars = 12 hex chars
        assert_eq!(parts[4].len(), 12);
    }

    #[test]
    fn test_schema_v1_migrates_to_v2() {
        // Simulate an existing v1 database (no bookmarks table)
        let tmpdir = TempDir::new().unwrap();
        let store_path = tmpdir.path().join("v1.db");
        {
            let conn = Connection::open(&store_path).unwrap();
            conn.execute_batch(
                "CREATE TABLE schema_version (version INTEGER NOT NULL);
                 INSERT INTO schema_version VALUES (1);
                 CREATE TABLE links (
                     id INTEGER PRIMARY KEY,
                     source TEXT NOT NULL,
                     target TEXT NOT NULL,
                     note TEXT,
                     created_at TEXT NOT NULL,
                     UNIQUE(source, target)
                 );",
            )
            .unwrap();
        }
        // Opening should auto-migrate and add the bookmarks table
        let store = LinkStore::open(&store_path).unwrap();
        // Can now create a bookmark
        let id = store.store_bookmark("/migrated/file.md").unwrap();
        assert!(!id.is_empty());
    }
}
