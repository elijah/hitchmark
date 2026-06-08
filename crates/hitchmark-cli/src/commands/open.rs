//! `hk open` — resolve and open a hook:// URI.

use hitchmark_core::{HookUri, UriType};

#[derive(clap::Parser)]
pub struct OpenArgs {
    /// Hook URI to open
    pub uri: String,
}

pub fn execute(args: OpenArgs) -> anyhow::Result<()> {
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
            println!("Bookmark URIs require metadata lookup (not yet implemented)");
            println!("Bookmark ID: {id}");
        }
        UriType::XCallbackUrl(action) => {
            println!("x-callback-url not yet supported");
            println!("Action: {action}");
        }
    }

    Ok(())
}
