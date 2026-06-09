//! `hk export` — export links and bookmarks to NDJSON or JSON.
use std::path::Path;

#[derive(clap::Parser)]
pub struct ExportArgs {
    /// Output format: ndjson (default, one record per line) or json (pretty array)
    #[arg(long, default_value = "ndjson", value_parser = ["ndjson", "json"])]
    pub format: String,

    /// Restrict export to links or bookmarks only
    #[arg(long, value_name = "TYPE", value_parser = ["links", "bookmarks"])]
    pub only: Option<String>,

    /// Write to FILE instead of stdout
    #[arg(long, value_name = "FILE")]
    pub out: Option<String>,
}

pub fn execute(_args: ExportArgs, _store_path: &Path) -> anyhow::Result<()> {
    anyhow::bail!("hk export: not yet implemented (coming in next commit)")
}
