//! `hk bookmark` — manage stored bookmark URIs.

use hitchmark_core::LinkStore;
use std::path::Path;

#[derive(clap::Parser)]
pub struct BookmarkArgs {
    #[command(subcommand)]
    pub command: BookmarkCmd,
}

#[derive(clap::Subcommand)]
pub enum BookmarkCmd {
    /// Create a bookmark URI for a file (idempotent)
    Create {
        /// Path to file
        path: String,
    },

    /// Show the file path a bookmark currently points to
    Show {
        /// Bookmark UUID or full hook://bookmark/<uuid> URI
        id: String,
    },

    /// Update the file path for an existing bookmark (after renaming/moving)
    Update {
        /// Bookmark UUID or full hook://bookmark/<uuid> URI
        id: String,
        /// New file path
        path: String,
    },

    /// List all stored bookmarks
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

pub fn execute(args: BookmarkArgs, store_path: &Path) -> anyhow::Result<()> {
    let store = LinkStore::open(store_path)?;

    match args.command {
        BookmarkCmd::Create { path } => {
            let canonical = std::fs::canonicalize(&path)
                .map_err(|e| anyhow::anyhow!("Cannot resolve path '{path}': {e}"))?;
            let path_str = canonical
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Path is not valid UTF-8"))?;
            let id = store.store_bookmark(path_str)?;
            println!("hook://bookmark/{id}");
        }

        BookmarkCmd::Show { id } => {
            let id = strip_bookmark_prefix(&id);
            match store.lookup_bookmark(id)? {
                Some(path) => println!("{path}"),
                None => anyhow::bail!("Bookmark '{id}' not found"),
            }
        }

        BookmarkCmd::Update { id, path } => {
            let id = strip_bookmark_prefix(&id).to_string();
            let canonical = std::fs::canonicalize(&path)
                .map_err(|e| anyhow::anyhow!("Cannot resolve path '{path}': {e}"))?;
            let path_str = canonical
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Path is not valid UTF-8"))?;
            if store.update_bookmark_path(&id, path_str)? {
                println!("Updated bookmark {id} → {path_str}");
            } else {
                anyhow::bail!("Bookmark '{id}' not found");
            }
        }

        BookmarkCmd::List { json } => {
            let bookmarks = store.list_bookmarks()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&bookmarks)?);
            } else {
                if bookmarks.is_empty() {
                    println!("No bookmarks stored.");
                } else {
                    for b in &bookmarks {
                        println!("hook://bookmark/{}\t{}", b.id, b.file_path);
                    }
                }
            }
        }
    }

    Ok(())
}

fn strip_bookmark_prefix(s: &str) -> &str {
    s.strip_prefix("hook://bookmark/").unwrap_or(s)
}
