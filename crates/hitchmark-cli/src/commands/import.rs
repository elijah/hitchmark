//! `hk import` — import links and bookmarks from NDJSON or JSON.
use std::path::Path;

#[derive(clap::Parser)]
pub struct ImportArgs {
    /// Input file (use - for stdin)
    pub file: String,

    /// Input format: ndjson (default) or json (array)
    #[arg(long, default_value = "ndjson", value_parser = ["ndjson", "json"])]
    pub format: String,

    /// Validate and report what would be imported without writing
    #[arg(long)]
    pub dry_run: bool,
}

pub fn execute(_args: ImportArgs, _store_path: &Path) -> anyhow::Result<()> {
    anyhow::bail!("hk import: not yet implemented (coming in next commit)")
}
