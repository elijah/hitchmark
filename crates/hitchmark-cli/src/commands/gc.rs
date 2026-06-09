//! `hk gc` — garbage-collect stale links and bookmarks.
//!
//! By default runs in dry-run mode (reports only). Pass `--delete` to remove.
//! Exits with code 1 when stale entries are found, 0 when the store is clean.
//! This makes it usable in CI scripts: `hk gc || echo "store has stale entries"`.

use hitchmark_core::LinkStore;
use std::path::Path;

#[derive(clap::Parser)]
pub struct GcArgs {
    /// Actually delete stale entries (default: dry-run / report only)
    #[arg(long)]
    pub delete: bool,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,

    /// Restrict scan to links or bookmarks only
    #[arg(long, value_name = "TYPE", value_parser = ["links", "bookmarks"])]
    pub only: Option<String>,
}

pub fn execute(args: GcArgs, store_path: &Path) -> anyhow::Result<()> {
    let store = LinkStore::open(store_path)?;

    let scan_links = args.only.as_deref() != Some("bookmarks");
    let scan_bookmarks = args.only.as_deref() != Some("links");

    let (stale_links, link_total) = if scan_links {
        store.scan_stale_links()?
    } else {
        (vec![], 0)
    };

    let (stale_bookmarks, bookmark_total) = if scan_bookmarks {
        store.scan_stale_bookmarks()?
    } else {
        (vec![], 0)
    };

    let stale_link_count = stale_links.len();
    let stale_bookmark_count = stale_bookmarks.len();
    let any_stale = stale_link_count > 0 || stale_bookmark_count > 0;

    if args.json {
        let obj = serde_json::json!({
            "dry_run": !args.delete,
            "links": {
                "checked": link_total,
                "stale": stale_links.iter().map(|l| serde_json::json!({
                    "source": l.source,
                    "target": l.target,
                    "note": l.note,
                    "created_at": l.created_at
                })).collect::<Vec<_>>()
            },
            "bookmarks": {
                "checked": bookmark_total,
                "stale": stale_bookmarks.iter().map(|b| serde_json::json!({
                    "id": b.id,
                    "file_path": b.file_path,
                    "created_at": b.created_at
                })).collect::<Vec<_>>()
            }
        });
        println!("{}", serde_json::to_string_pretty(&obj)?);
    } else {
        if scan_links {
            println!(
                "Links:     {stale_link_count} stale / {link_total} checked"
            );
            for link in &stale_links {
                println!("  STALE  {} ↔ {}", link.source, link.target);
            }
        }
        if scan_bookmarks {
            println!(
                "Bookmarks: {stale_bookmark_count} stale / {bookmark_total} checked"
            );
            for bm in &stale_bookmarks {
                println!("  STALE  hook://bookmark/{} → {}", bm.id, bm.file_path);
            }
        }

        if !any_stale {
            println!("Store is clean.");
        } else if !args.delete {
            println!("\nDry run — pass --delete to remove stale entries.");
        }
    }

    if args.delete && any_stale {
        // Collect unique URIs from stale links (both ends)
        let stale_uris: Vec<String> = stale_links
            .iter()
            .flat_map(|l| [l.source.clone(), l.target.clone()])
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let removed_links = store.delete_links_involving(&stale_uris)?;

        let stale_ids: Vec<String> = stale_bookmarks.iter().map(|b| b.id.clone()).collect();
        let removed_bookmarks = store.delete_bookmarks_by_ids(&stale_ids)?;

        if args.json {
            // Already printed JSON above — print a summary object
            let summary = serde_json::json!({
                "deleted": { "links": removed_links, "bookmarks": removed_bookmarks }
            });
            println!("{}", serde_json::to_string_pretty(&summary)?);
        } else {
            println!(
                "\nDeleted {removed_links} link(s) and {removed_bookmarks} bookmark(s)."
            );
        }
    }

    // Exit 1 if anything stale was found (whether deleted or not)
    if any_stale {
        std::process::exit(1);
    }

    Ok(())
}
