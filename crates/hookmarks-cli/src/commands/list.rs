//! `hk list` — list all links for a URI.

use hookmarks_core::LinkStore;
use std::path::PathBuf;

#[derive(clap::Parser)]
pub struct ListArgs {
    /// URI to query (or file path)
    pub uri: String,
}

pub fn execute(args: ListArgs, store_path: &PathBuf) -> anyhow::Result<()> {
    let store = LinkStore::open(store_path)?;

    let uri = if args.uri.starts_with("hook://") {
        args.uri.clone()
    } else {
        crate::path::path_to_uri(&args.uri)?.to_string()
    };

    let links = store.list_links(&uri)?;

    if links.is_empty() {
        println!("No links found for: {uri}");
        return Ok(());
    }

    println!("Links for: {uri}\n");
    for (i, link) in links.iter().enumerate() {
        println!(
            "{}. {}",
            i + 1,
            if link.source == uri {
                &link.target
            } else {
                &link.source
            }
        );
        if let Some(note) = &link.note {
            println!("   Note: {note}");
        }
        println!("   Created: {}\n", link.created_at);
    }

    Ok(())
}
