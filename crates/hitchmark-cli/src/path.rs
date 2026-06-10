//! File path to hook:// URI conversion.

use hitchmark_core::{HookUri, UriType};
use std::path::{Component, Path, PathBuf};

/// Convert a file path to a hook:// URI.
///
/// Handles expansion of ~ and relative paths, normalizes to absolute.
pub fn path_to_uri(path: &str) -> anyhow::Result<HookUri> {
    let path_buf = expand_path(path)?;

    // Ensure it's absolute
    let abs_path = if path_buf.is_absolute() {
        path_buf
    } else {
        std::env::current_dir()?.join(path_buf)
    };

    // Normalize and convert to string
    let normalized = normalize_path(&abs_path);
    let path_str = normalized
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Path contains invalid UTF-8"))?;

    // Create the URI
    let uri = HookUri {
        uri_type: UriType::File(PathBuf::from(path_str)),
        fragment: None,
    };

    Ok(uri)
}

/// Expand ~ and relative paths to absolute.
fn expand_path(path: &str) -> anyhow::Result<PathBuf> {
    expand_path_pub(path)
}

/// Expand ~ and relative paths to absolute. (public for use in other modules)
pub fn expand_path_pub(path: &str) -> anyhow::Result<PathBuf> {
    // ~/ expansion (Unix convention; harmless on Windows since paths don't start with ~/)
    if let Some(rest) = path.strip_prefix("~/") {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        return Ok(home.join(rest));
    }
    // Windows: %USERPROFILE% style not expanded here (shell handles it)
    Ok(PathBuf::from(path))
}

/// Normalize a path by resolving . and .. components.
fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            _ => {
                result.push(component);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_home() {
        let expanded = expand_path("~/foo/bar").unwrap();
        assert!(expanded.is_absolute());
        assert!(!expanded.to_string_lossy().contains("~"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_normalize_dots() {
        let path = PathBuf::from("/a/b/../c/./d");
        let normalized = normalize_path(&path);
        assert_eq!(normalized.to_string_lossy(), "/a/c/d");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_normalize_dots_windows() {
        let path = PathBuf::from(r"C:\a\b\..\c\.\d");
        let normalized = normalize_path(&path);
        assert_eq!(normalized.to_string_lossy(), r"C:\a\c\d");
    }
}
