//! `hk link` — create a bidirectional link between two URIs.

use hitchmark_core::LinkStore;
use std::path::PathBuf;

#[derive(clap::Parser)]
pub struct LinkArgs {
    /// First URI (or file path)
    pub uri_a: String,

    /// Second URI (or file path)
    pub uri_b: String,

    /// Optional note attached to the link
    #[arg(long)]
    pub note: Option<String>,
}

pub fn execute(args: LinkArgs, store_path: &PathBuf) -> anyhow::Result<()> {
    let store = LinkStore::open(store_path)?;

    // Expand file paths to hook:// URIs if needed
    let uri_a = expand_uri(&args.uri_a)?;
    let uri_b = expand_uri(&args.uri_b)?;

    match store.create_link(&uri_a, &uri_b, args.note.as_deref()) {
        Ok(()) => {
            println!("✓ Linked:");
            println!("  {uri_a}");
            println!("  {uri_b}");
            if let Some(note) = args.note {
                println!("  Note: {note}");
            }
        }
        Err(hitchmark_core::Error::LinkAlreadyExists { .. }) => {
            eprintln!("ℹ️  These resources are already linked.");
            eprintln!("   {uri_a}");
            eprintln!("   {uri_b}");
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

fn expand_uri(uri_or_path: &str) -> anyhow::Result<String> {
    if uri_or_path.starts_with("hook://") {
        Ok(uri_or_path.to_string())
    } else {
        // Assume it's a file path
        let hook_uri = crate::path::path_to_uri(uri_or_path)?;
        Ok(hook_uri.to_string())
    }
}
