//! Error types for hitchmark-core.

use thiserror::Error;

/// Hitchmark result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types.
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid hook:// URI
    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    /// Purple ID error
    #[error("Purple ID error: {0}")]
    PurpleError(String),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Link already exists between the two URIs
    #[error("Link already exists between {uri_a} and {uri_b}")]
    LinkAlreadyExists {
        /// First URI in the existing link
        uri_a: String,
        /// Second URI in the existing link
        uri_b: String,
    },

    /// Bookmark with this UUID already exists in the store
    #[error("Bookmark '{id}' already exists in the store")]
    BookmarkAlreadyExists {
        /// The conflicting bookmark UUID
        id: String,
    },

    /// Database schema version mismatch — database was created by a newer version
    #[error("Database schema version {found} is newer than supported version {supported}. Please upgrade hitchmark.")]
    SchemaTooNew {
        /// Schema version found in the database
        found: u32,
        /// Maximum schema version this build supports
        supported: u32,
    },

    /// Other errors
    #[error("{0}")]
    Other(String),
}
