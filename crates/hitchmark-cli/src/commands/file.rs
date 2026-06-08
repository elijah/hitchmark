//! `hk file` — print the hook:// URI for a file.

use hitchmark_core::LinkStore;
use std::path::Path;

#[derive(clap::Parser)]
pub struct FileArgs {
    /// Path to file
    pub path: String,

    /// Store a stable bookmark URI instead of a file URI.
    ///
    /// Returns `hook://bookmark/<uuid>` that resolves even after the file is
    /// moved (as long as the bookmark is updated with `hk bookmark update`).
    #[arg(long)]
    pub bookmark: bool,
}

pub fn execute(args: FileArgs, store_path: &Path) -> anyhow::Result<()> {
    if args.bookmark {
        let canonical = std::fs::canonicalize(&args.path)
            .map_err(|e| anyhow::anyhow!("Cannot resolve path '{}': {e}", args.path))?;
        let path_str = canonical
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Path is not valid UTF-8"))?;
        let store = LinkStore::open(store_path)?;
        let id = store.store_bookmark(path_str)?;
        println!("hook://bookmark/{id}");
    } else {
        let uri = crate::path::path_to_uri(&args.path)?;
        println!("{uri}");
    }
    Ok(())
}
