//! Purple number generation and management.
//!
//! Purple numbers are stable, human-readable IDs for paragraphs in documents.
//! They survive document reordering and minor edits through content-hash-based generation.

use crate::error::Result;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// A purple number ID: short, base-58 encoded hash of paragraph content.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PurpleId(String);

impl PurpleId {
    /// Create a new purple ID from raw hash (typically 6 chars, base-58)
    pub fn new(id: String) -> Self {
        PurpleId(id)
    }

    /// Return the ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PurpleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Generates stable purple numbers for paragraphs.
pub struct PurpleNumberGenerator {
    collision_map: HashMap<String, usize>,
}

impl PurpleNumberGenerator {
    /// Create a new purple number generator
    pub fn new() -> Self {
        PurpleNumberGenerator {
            collision_map: HashMap::new(),
        }
    }

    /// Generate a purple ID for paragraph text.
    /// Uses SHA-256(text) → base-58 → first 6 chars
    pub fn generate(&mut self, paragraph_text: &str) -> Result<PurpleId> {
        let mut hasher = Sha256::new();
        hasher.update(paragraph_text);
        let hash = hasher.finalize();
        let base58 = bs58::encode(&hash[..]).into_string();

        // Take first 6 characters
        let short_id = base58.chars().take(6).collect::<String>();

        // Detect collisions within document: extend to 8 chars if needed
        let count = self.collision_map.entry(short_id.clone()).or_insert(0);
        *count += 1;
        let final_id = if *count > 1 {
            base58.chars().take(8).collect::<String>()
        } else {
            short_id
        };

        Ok(PurpleId::new(final_id))
    }
}

impl Default for PurpleNumberGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Split markdown text into paragraphs
pub fn split_paragraphs(text: &str) -> Vec<String> {
    text.split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purple_id_generation() {
        let mut gen = PurpleNumberGenerator::new();
        let id = gen.generate("Hello, world!").unwrap();
        assert_eq!(id.as_str().len(), 6);
    }

    #[test]
    fn test_paragraph_splitting() {
        let text = "First paragraph\n\nSecond paragraph\n\nThird";
        let paras = split_paragraphs(text);
        assert_eq!(paras.len(), 3);
        assert_eq!(paras[0], "First paragraph");
    }
}
