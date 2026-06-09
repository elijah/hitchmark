//! `hk open` — resolve and open a hook:// URI.

use hitchmark_core::{HookUri, LinkStore, UriType};
use hitchmark_core::uri::XCallbackUri;
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
        UriType::XCallbackUrl(xcb) => {
            dispatch_xcallback(xcb, store_path)?;
        }
    }

    Ok(())
}

fn dispatch_xcallback(xcb: XCallbackUri, store_path: &Path) -> anyhow::Result<()> {
    let result = match xcb.action.as_str() {
        "create-link" => action_create_link(&xcb, store_path),
        "open" => action_open(&xcb, store_path),
        "copy-uri" => action_copy_uri(&xcb),
        other => Err(anyhow::anyhow!("Unknown x-callback-url action: '{other}'")),
    };

    match result {
        Ok(()) => {
            if let Some(success_url) = xcb.callback("x-success") {
                let _ = opener::open(success_url);
            }
            Ok(())
        }
        Err(e) => {
            if let Some(error_url) = xcb.callback("x-error") {
                let encoded_msg = percent_encode(&e.to_string());
                let callback = format!("{error_url}?errorMessage={encoded_msg}");
                let _ = opener::open(&callback);
                // Return Ok — the error was communicated via callback
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

fn action_create_link(xcb: &XCallbackUri, store_path: &Path) -> anyhow::Result<()> {
    let source = xcb
        .params
        .get("source")
        .ok_or_else(|| anyhow::anyhow!("create-link: missing 'source' parameter"))?;
    let target = xcb
        .params
        .get("target")
        .ok_or_else(|| anyhow::anyhow!("create-link: missing 'target' parameter"))?;
    let note = xcb.params.get("note").map(String::as_str);

    let store = LinkStore::open(store_path)?;
    store.create_link(source, target, note)?;
    println!("Linked: {source} ↔ {target}");
    Ok(())
}

fn action_open(xcb: &XCallbackUri, store_path: &Path) -> anyhow::Result<()> {
    let uri = xcb
        .params
        .get("uri")
        .ok_or_else(|| anyhow::anyhow!("open: missing 'uri' parameter"))?;
    // Recurse: parse and open the target URI
    let inner = OpenArgs { uri: uri.clone() };
    execute(inner, store_path)
}

fn action_copy_uri(xcb: &XCallbackUri) -> anyhow::Result<()> {
    let path = xcb
        .params
        .get("path")
        .ok_or_else(|| anyhow::anyhow!("copy-uri: missing 'path' parameter"))?;
    let hook_uri = crate::path::path_to_uri(path)?;
    let uri_str = hook_uri.to_string();
    copy_to_clipboard(&uri_str)?;
    println!("Copied: {uri_str}");
    Ok(())
}

fn copy_to_clipboard(text: &str) -> anyhow::Result<()> {
    // Use pbcopy on macOS, xclip/xsel on Linux, clip on Windows
    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        let mut child = std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
        child.wait()?;
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        use std::io::Write;
        // Try xclip first, fall back to xsel
        for cmd in &["xclip -selection clipboard", "xsel --clipboard --input"] {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if let Ok(mut child) = std::process::Command::new(parts[0])
                .args(&parts[1..])
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
                child.wait()?;
                return Ok(());
            }
        }
        anyhow::bail!("Could not find xclip or xsel — install one to use copy-uri");
    }
    #[cfg(target_os = "windows")]
    {
        use std::io::Write;
        let mut child = std::process::Command::new("clip")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
        child.wait()?;
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        anyhow::bail!("copy-uri: clipboard not supported on this platform");
    }
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

