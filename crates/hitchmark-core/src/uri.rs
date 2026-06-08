//! URI parsing and validation for hook:// scheme.
//!
//! Normalizes URIs and provides type-safe handling of file, bookmark, and callback variants.

use crate::error::{Error, Result};
use base64::{engine::general_purpose, Engine};
use std::path::PathBuf;

/// A parsed hook:// URI
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookUri {
    /// Type of URI (file, bookmark, or callback)
    pub uri_type: UriType,
    /// Optional fragment (e.g., #para-abc123)
    pub fragment: Option<String>,
}

/// The kind of hook:// URI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UriType {
    /// File reference: hook://file/<base64url-path>
    File(PathBuf),
    /// Bookmark reference: hook://bookmark/<uuid>
    Bookmark(String),
    /// x-callback-url: hook://x-callback-url/<action>
    XCallbackUrl(String),
}

impl HookUri {
    /// Parse a hook:// URI string
    pub fn parse(uri: &str) -> Result<Self> {
        if !uri.starts_with("hook://") {
            return Err(Error::InvalidUri("Missing 'hook://' prefix".into()));
        }

        let rest = &uri[7..];
        let (body, fragment) = match rest.split_once('#') {
            Some((b, f)) => (b, Some(f.to_string())),
            None => (rest, None),
        };

        let uri_type = if let Some(encoded_path) = body.strip_prefix("file/") {
            let decoded = general_purpose::URL_SAFE_NO_PAD
                .decode(encoded_path)
                .map_err(|_| Error::InvalidUri("Invalid base64 in file path".into()))?;
            let path_str = String::from_utf8(decoded)
                .map_err(|_| Error::InvalidUri("Path is not valid UTF-8".into()))?;
            UriType::File(PathBuf::from(path_str))
        } else if let Some(id) = body.strip_prefix("bookmark/") {
            UriType::Bookmark(id.to_string())
        } else if let Some(action) = body.strip_prefix("x-callback-url/") {
            UriType::XCallbackUrl(action.to_string())
        } else {
            return Err(Error::InvalidUri(format!("Unknown URI type: {body}")));
        };

        Ok(HookUri { uri_type, fragment })
    }
}

impl std::fmt::Display for HookUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let body = match &self.uri_type {
            UriType::File(path) => {
                let path_str = path.to_string_lossy();
                let encoded = general_purpose::URL_SAFE_NO_PAD.encode(path_str.as_bytes());
                format!("hook://file/{encoded}")
            }
            UriType::Bookmark(id) => format!("hook://bookmark/{id}"),
            UriType::XCallbackUrl(action) => format!("hook://x-callback-url/{action}"),
        };

        match &self.fragment {
            Some(frag) => write!(f, "{body}#{frag}"),
            None => write!(f, "{body}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_uri() {
        let uri = "hook://file/L3Zhci9sb2cvdGV4dC50eHQ#para-abc123";
        let parsed = HookUri::parse(uri).unwrap();
        assert_eq!(parsed.fragment, Some("para-abc123".to_string()));
    }

    #[test]
    fn test_parse_bookmark_uri() {
        let uri = "hook://bookmark/550e8400-e29b-41d4-a716-446655440000";
        let parsed = HookUri::parse(uri).unwrap();
        match parsed.uri_type {
            UriType::Bookmark(id) => assert_eq!(id, "550e8400-e29b-41d4-a716-446655440000"),
            _ => panic!("Expected bookmark URI"),
        }
    }

    #[test]
    fn test_roundtrip_file_uri() {
        let original = "hook://file/L2Zvby9iYXIudHh0#para-xyz";
        let parsed = HookUri::parse(original).unwrap();
        let serialized = parsed.to_string();
        assert_eq!(serialized, original);
    }

    #[test]
    fn test_roundtrip_path_with_spaces() {
        // Paths with spaces must survive encode → decode
        let path = "/Users/alice/My Documents/project notes.md";
        let uri = HookUri {
            uri_type: UriType::File(std::path::PathBuf::from(path)),
            fragment: None,
        };
        let serialized = uri.to_string();
        let parsed = HookUri::parse(&serialized).unwrap();
        match parsed.uri_type {
            UriType::File(p) => assert_eq!(p.to_string_lossy(), path),
            _ => panic!("Expected File URI"),
        }
    }

    #[test]
    fn test_roundtrip_unicode_path() {
        let path = "/Users/ألعاب/文档/résumé.md";
        let uri = HookUri {
            uri_type: UriType::File(std::path::PathBuf::from(path)),
            fragment: None,
        };
        let serialized = uri.to_string();
        let parsed = HookUri::parse(&serialized).unwrap();
        match parsed.uri_type {
            UriType::File(p) => assert_eq!(p.to_string_lossy(), path),
            _ => panic!("Expected File URI"),
        }
    }

    #[test]
    fn test_missing_scheme_is_error() {
        assert!(HookUri::parse("file:///foo/bar").is_err());
        assert!(HookUri::parse("https://example.com").is_err());
        assert!(HookUri::parse("").is_err());
    }

    #[test]
    fn test_unknown_authority_is_error() {
        assert!(HookUri::parse("hook://unknown/abc").is_err());
    }

    #[test]
    fn test_invalid_base64_is_error() {
        // "!!" is not valid base64url
        assert!(HookUri::parse("hook://file/!!invalid!!").is_err());
    }

    #[test]
    fn test_fragment_without_path() {
        let uri = "hook://file/L2Zvby9iYXIudHh0";
        let parsed = HookUri::parse(uri).unwrap();
        assert!(parsed.fragment.is_none());
    }

    #[test]
    fn test_x_callback_url() {
        let uri = "hook://x-callback-url/create-link";
        let parsed = HookUri::parse(uri).unwrap();
        match parsed.uri_type {
            UriType::XCallbackUrl(action) => assert_eq!(action, "create-link"),
            _ => panic!("Expected XCallbackUrl"),
        }
    }
}
