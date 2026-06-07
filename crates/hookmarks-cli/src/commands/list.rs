//! `hk list` — list all links for a URI.

use hookmarks_core::LinkStore;
use std::path::PathBuf;

#[derive(clap::Parser)]
pub struct ListArgs {
    /// URI to query (or file path)
    pub uri: String,

    /// Output as JSON (machine-readable)
    #[arg(long)]
    pub json: bool,
}

pub fn execute(args: ListArgs, store_path: &PathBuf) -> anyhow::Result<()> {
    let store = LinkStore::open(store_path)?;

    let uri = if args.uri.starts_with("hook://") {
        args.uri.clone()
    } else {
        crate::path::path_to_uri(&args.uri)?.to_string()
    };

    let links = store.list_links(&uri)?;

    if args.json {
        // JSON output for machine consumption (Obsidian bridge, scripts, etc.)
        let json: Vec<_> = links
            .iter()
            .map(|l| {
                serde_json::json!({
                    "source": l.source,
                    "target": l.target,
                    "note": l.note,
                    "created_at": l.created_at,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

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
