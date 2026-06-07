//! `hk delete` — remove a bidirectional link.

use hookmarks_core::LinkStore;
use std::path::PathBuf;

#[derive(clap::Parser)]
pub struct DeleteArgs {
    /// First URI (or file path)
    pub uri_a: String,

    /// Second URI (or file path)
    pub uri_b: String,

    /// Do not ask for confirmation
    #[arg(long, short = 'y')]
    pub yes: bool,
}

pub fn execute(args: DeleteArgs, store_path: &PathBuf) -> anyhow::Result<()> {
    let uri_a = expand_uri(&args.uri_a)?;
    let uri_b = expand_uri(&args.uri_b)?;

    // Confirm unless --yes flag is passed
    if !args.yes {
        eprint!("Delete link between\n  {uri_a}\n  {uri_b}\n? [y/N] ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            eprintln!("Cancelled.");
            return Ok(());
        }
    }

    let store = LinkStore::open(store_path)?;
    store.delete_link(&uri_a, &uri_b)?;
    println!("✓ Link deleted.");

    Ok(())
}

fn expand_uri(uri_or_path: &str) -> anyhow::Result<String> {
    if uri_or_path.starts_with("hook://") {
        Ok(uri_or_path.to_string())
    } else {
        let hook_uri = crate::path::path_to_uri(uri_or_path)?;
        Ok(hook_uri.to_string())
    }
}
