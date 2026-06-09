//! URI parsing and validation for hook:// scheme.
//!
//! Normalizes URIs and provides type-safe handling of file, bookmark, and callback variants.

use crate::error::{Error, Result};
use base64::{engine::general_purpose, Engine};
use std::collections::HashMap;
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
    /// x-callback-url: hook://x-callback-url/<action>[?params]
    XCallbackUrl(XCallbackUri),
}

/// A parsed x-callback-url payload.
///
/// Spec: <http://x-callback-url.com>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XCallbackUri {
    /// The action name, e.g. `create-link`, `open`, `copy-uri`
    pub action: String,
    /// All query parameters (URL-decoded key=value pairs)
    pub params: HashMap<String, String>,
}

impl XCallbackUri {
    /// Convenience: get an optional callback URL by name (`x-success`, `x-error`, `x-cancel`)
    pub fn callback(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(String::as_str)
    }
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
        } else if let Some(action_and_query) = body.strip_prefix("x-callback-url/") {
            let (action, query) = match action_and_query.split_once('?') {
                Some((a, q)) => (a, q),
                None => (action_and_query, ""),
            };
            let params = parse_query_string(query);
            UriType::XCallbackUrl(XCallbackUri {
                action: action.to_string(),
                params,
            })
        } else {
            return Err(Error::InvalidUri(format!("Unknown URI type: {body}")));
        };

        Ok(HookUri { uri_type, fragment })
    }
}

/// Decode a percent-encoded query string into key=value pairs.
fn parse_query_string(query: &str) -> HashMap<String, String> {
    if query.is_empty() {
        return HashMap::new();
    }
    query
        .split('&')
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            Some((percent_decode(k), percent_decode(v)))
        })
        .collect()
}

/// Minimal percent-decode: handle %XX sequences and + as space.
fn percent_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'+' {
            out.push(' ');
            i += 1;
        } else if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("??"),
                16,
            ) {
                out.push(hex as char);
                i += 3;
                continue;
            }
            out.push(bytes[i] as char);
            i += 1;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
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
            UriType::XCallbackUrl(xcb) => {
                if xcb.params.is_empty() {
                    format!("hook://x-callback-url/{}", xcb.action)
                } else {
                    let qs: String = xcb
                        .params
                        .iter()
                        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
                        .collect::<Vec<_>>()
                        .join("&");
                    format!("hook://x-callback-url/{}?{qs}", xcb.action)
                }
            }
        };
        match &self.fragment {
            Some(frag) => write!(f, "{body}#{frag}"),
            None => write!(f, "{body}"),
        }
    }
}

/// Minimal percent-encode for query string values (encode non-unreserved chars).
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
    fn test_x_callback_url_no_params() {
        let uri = "hook://x-callback-url/create-link";
        let parsed = HookUri::parse(uri).unwrap();
        match parsed.uri_type {
            UriType::XCallbackUrl(xcb) => {
                assert_eq!(xcb.action, "create-link");
                assert!(xcb.params.is_empty());
            }
            _ => panic!("Expected XCallbackUrl"),
        }
    }

    #[test]
    fn test_x_callback_url_with_params() {
        let uri = "hook://x-callback-url/open?uri=hook%3A%2F%2Ffile%2Fabc&x-success=myapp%3A%2F%2Fsuccess";
        let parsed = HookUri::parse(uri).unwrap();
        match parsed.uri_type {
            UriType::XCallbackUrl(xcb) => {
                assert_eq!(xcb.action, "open");
                assert_eq!(xcb.params.get("uri").unwrap(), "hook://file/abc");
                assert_eq!(xcb.callback("x-success").unwrap(), "myapp://success");
            }
            _ => panic!("Expected XCallbackUrl"),
        }
    }

    #[test]
    fn test_x_callback_url_roundtrip() {
        let uri = "hook://x-callback-url/copy-uri";
        let parsed = HookUri::parse(uri).unwrap();
        assert_eq!(parsed.to_string(), uri);
    }

    #[test]
    fn test_query_string_percent_decode() {
        let decoded = super::percent_decode("hello%20world%21");
        assert_eq!(decoded, "hello world!");
    }

    #[test]
    fn test_query_string_plus_as_space() {
        let decoded = super::percent_decode("hello+world");
        assert_eq!(decoded, "hello world");
    }

    #[test]
    fn test_roundtrip_windows_path() {
        // Windows absolute paths use drive letters and backslashes.
        // PathBuf stores them as-is; the URI round-trip must preserve the exact string.
        let path = r"C:\Users\alice\Documents\notes.md";
        let uri = HookUri {
            uri_type: UriType::File(std::path::PathBuf::from(path)),
            fragment: None,
        };
        let serialized = uri.to_string();
        assert!(serialized.starts_with("hook://file/"));
        let parsed = HookUri::parse(&serialized).unwrap();
        match parsed.uri_type {
            UriType::File(p) => assert_eq!(p.to_string_lossy(), path),
            _ => panic!("Expected File URI"),
        }
    }

    #[test]
    fn test_roundtrip_windows_unc_path() {
        // UNC paths (network shares) are common on Windows.
        let path = r"\\server\share\project\file.md";
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
    fn test_roundtrip_windows_path_with_spaces() {
        let path = r"C:\Users\alice\My Documents\résumé 2024.md";
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
    fn test_roundtrip_windows_path_with_fragment() {
        let path = r"C:\notes\meeting.md";
        let uri = HookUri {
            uri_type: UriType::File(std::path::PathBuf::from(path)),
            fragment: Some("para-abc123".to_string()),
        };
        let serialized = uri.to_string();
        assert!(serialized.contains("#para-abc123"));
        let parsed = HookUri::parse(&serialized).unwrap();
        assert_eq!(parsed.fragment, Some("para-abc123".to_string()));
        match parsed.uri_type {
            UriType::File(p) => assert_eq!(p.to_string_lossy(), path),
            _ => panic!("Expected File URI"),
        }
    }
}
