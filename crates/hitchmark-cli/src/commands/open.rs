//! `hk open` — resolve and open a hook:// URI.

use hitchmark_core::{HookUri, LinkStore, UriType};
use std::path::Path;

#[derive(clap::Parser)]
pub struct OpenArgs {
    /// Hook URI to open
    pub uri: String,
}

pub fn execute(args: OpenArgs, store_path: &Path) -> anyhow::Result<()> {
    let hook_uri = HookUri::parse(&args.uri)?;

    match hook_uri.uri_type {
        UriType::File(path) => {
            if path.exists() {
                opener::open(&path)?;
                println!("Opened: {}", path.display());
            } else {
                anyhow::bail!("File not found: {}", path.display());
            }
        }
        UriType::Bookmark(id) => {
            let store = LinkStore::open(store_path)?;
            match store.lookup_bookmark(&id)? {
                Some(file_path) => {
                    let path = std::path::PathBuf::from(&file_path);
                    if path.exists() {
                        opener::open(&path)?;
                        println!("Opened: {file_path}");
                    } else {
                        anyhow::bail!(
                            "Bookmark {id} points to '{file_path}' but the file no longer exists.\n\
                             If you moved the file, update the bookmark with:\n\
                             hk bookmark update {id} <new-path>"
                        );
                    }
                }
                None => {
                    anyhow::bail!(
                        "Bookmark '{id}' not found in the local store.\n\
                         Bookmark URIs are only resolvable on the machine where they were created."
                    );
                }
            }
        }
        UriType::XCallbackUrl(action) => {
            anyhow::bail!(
                "x-callback-url actions are not yet supported (action: {action})"
            );
        }
    }

    Ok(())
}
