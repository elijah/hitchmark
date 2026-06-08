//! Hitchmark daemon: Linux background service for hook:// URI handling.
//!
//! On Linux, registers as a DBus service (`org.hitchmark.Daemon`) and handles
//! `hook://` URIs via xdg-open integration.
//!
//! DBus interface `org.hitchmark.Daemon1`:
//!   - `OpenUri(uri: String)` — resolve and open a hook:// URI
//!   - `CreateLink(a: String, b: String, note: String)` — create bidirectional link
//!   - `ListLinks(uri: String)` — return all links for a resource
//!   - `FileToUri(path: String)` — convert file path to hook:// URI

#[cfg(target_os = "linux")]
mod linux {
    use hitchmark_core::{LinkStore, UriType};
    use std::path::PathBuf;
    use zbus::{connection, interface};

    /// DBus service struct — holds state shared across method calls.
    struct HitchmarkDaemon {
        store: LinkStore,
    }

    impl HitchmarkDaemon {
        fn new() -> zbus::Result<Self> {
            let store_path = Self::default_store_path();
            let store = LinkStore::open(&store_path)
                .map_err(|e| zbus::Error::Failure(e.to_string()))?;
            Ok(Self { store })
        }

        fn default_store_path() -> PathBuf {
            xdg::BaseDirectories::with_prefix("hookmarks")
                .unwrap_or_else(|_| xdg::BaseDirectories::new().unwrap())
                .get_data_file("store.db")
                .unwrap_or_else(|| {
                    let mut p = dirs_path();
                    p.push("hookmarks");
                    p.push("store.db");
                    p
                })
        }
    }

    fn dirs_path() -> PathBuf {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".local/share"))
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
    }

    #[interface(name = "org.hitchmark.Daemon1")]
    impl HitchmarkDaemon {
        /// Resolve and open a hook:// URI using xdg-open.
        async fn open_uri(&self, uri: String) -> zbus::fdo::Result<String> {
            let parsed = hitchmark_core::HookUri::parse(&uri)
                .map_err(|e| zbus::fdo::Error::InvalidArgs(e.to_string()))?;

            let target_path = match &parsed.uri_type {
                UriType::File(path) => path.to_string_lossy().to_string(),
                UriType::Bookmark(id) => {
                    return Err(zbus::fdo::Error::NotSupported(format!(
                        "Bookmark URIs not yet supported: {id}"
                    )));
                }
                UriType::XCallbackUrl(action) => {
                    return Err(zbus::fdo::Error::NotSupported(format!(
                        "x-callback-url not supported: {action}"
                    )));
                }
            };

            // Open the resolved path with xdg-open
            let status = tokio::process::Command::new("xdg-open")
                .arg(&target_path)
                .status()
                .await
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

            if status.success() {
                Ok(format!("Opened: {target_path}"))
            } else {
                Err(zbus::fdo::Error::Failed(format!(
                    "xdg-open failed with status: {status}"
                )))
            }
        }

        /// Create a bidirectional link between two URIs.
        async fn create_link(
            &mut self,
            uri_a: String,
            uri_b: String,
            note: String,
        ) -> zbus::fdo::Result<String> {
            let note_opt = if note.is_empty() { None } else { Some(note.as_str()) };
            self.store
                .create_link(&uri_a, &uri_b, note_opt)
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
            Ok(format!("Link created: {uri_a} <-> {uri_b}"))
        }

        /// Return all links for a URI as a newline-separated string.
        async fn list_links(&self, uri: String) -> zbus::fdo::Result<Vec<String>> {
            let links = self
                .store
                .list_links(&uri)
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

            let lines = links
                .iter()
                .map(|l| {
                    let note = l.note.as_deref().unwrap_or("");
                    format!("{}\t{}\t{}", l.source, l.target, note)
                })
                .collect();

            Ok(lines)
        }

        /// Convert a file path to a hook:// URI string.
        async fn file_to_uri(&self, path: String) -> zbus::fdo::Result<String> {
            let abs = if path.starts_with('/') {
                PathBuf::from(&path)
            } else {
                std::env::current_dir()
                    .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?
                    .join(&path)
            };

            let uri = hitchmark_core::HookUri {
                uri_type: UriType::File(abs),
                fragment: None,
            };
            Ok(uri.to_string())
        }
    }

    /// Run the daemon — acquires DBus name and serves requests.
    pub async fn run() -> anyhow::Result<()> {
        let daemon = HitchmarkDaemon::new()?;

        let _conn = connection::Builder::session()?
            .name("org.hitchmark.Daemon")?
            .serve_at("/org/hitchmark/Daemon", daemon)?
            .build()
            .await?;

        eprintln!("Hitchmark daemon started — DBus name: org.hitchmark.Daemon");
        eprintln!("Listening for hook:// URI requests...");

        // Block forever — the connection drives the event loop
        std::future::pending::<()>().await;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    linux::run().await
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("hitchmark-daemon is only supported on Linux.");
    eprintln!("On macOS, use the Hookmarks.app menu bar application.");
    std::process::exit(1);
}
