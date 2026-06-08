//! Hookmarks core library: URI parsing, storage, and purple-number generation.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod purple;
pub mod store;
pub mod uri;

pub use error::{Error, Result};
pub use purple::{PurpleId, PurpleNumberGenerator};
pub use purple::split_paragraphs;
pub use store::LinkStore;
pub use uri::{HookUri, UriType};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
