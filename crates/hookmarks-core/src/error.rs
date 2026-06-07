//! Error types for hookmarks-core.

use thiserror::Error;

/// Hookmarks result type.
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

    /// Other errors
    #[error("{0}")]
    Other(String),
}
