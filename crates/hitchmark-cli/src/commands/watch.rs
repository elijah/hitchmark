//! `hk watch` — watch bookmarked file locations and auto-repair on rename/move.
use std::path::Path;

#[derive(clap::Parser)]
pub struct WatchArgs {
    /// Print each filesystem event as it is processed
    #[arg(long)]
    pub verbose: bool,
}

pub fn execute(_args: WatchArgs, _store_path: &Path) -> anyhow::Result<()> {
    anyhow::bail!("hk watch: not yet implemented (coming in feature/hk-watch)")
}
