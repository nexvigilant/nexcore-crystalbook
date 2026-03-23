//! Cell — the atomic unit of a Crystalbook document.
//!
//! Every cell has content, a type, and a content-addressed hash.
//! The hash is computed from the source bytes and is the cell's identity
//! in the Merkle tree.
//!
//! ## Integrity Contract
//!
//! `Cell::law()` and `Cell::new()` are infallible constructors that always
//! produce a valid, hashable cell. `Cell::try_law()` propagates serialization
//! errors for callers that need strict guarantees.

use nexcore_hash::sha256::Sha256;
use serde::{Deserialize, Serialize};

// ── CellId ──────────────────────────────────────────────

/// Unique cell identifier — deterministic from position and content.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellId(pub String);

impl CellId {
    /// Create a cell ID from a string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl core::fmt::Display for CellId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── CellType ────────────────────────────────────────────

/// The type of content a cell holds.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CellType {
    /// Markdown prose — rendered as styled text.
    Text,
    /// A structured Crystalbook Law (vice, virtue, deviation, correction, principle).
    Law,
    /// Rust source code — compiled and executed.
    RustCode,
    /// Shell commands — executed in a sandboxed PTY.
    ShellCode,
    /// PVDSL script — executed via the PVDSL engine.
    PvdslCode,
    /// Interactive 8-Laws diagnostic widget.
    Diagnostic,
}

// ── LawContent ──────────────────────────────────────────

/// The structured content of a Law cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LawContent {
    /// Roman numeral (I–VIII).
    pub num: String,
    /// Law title (e.g., "The Law of True Measure").
    pub title: String,
    /// Vice name and Latin.
    pub vice: ViceVirtue,
    /// Virtue name and Latin.
    pub virtue: ViceVirtue,
    /// The deviation — how the vice manifests.
    pub deviation: String,
    /// The correction — how the virtue restores.
    pub correction: String,
    /// The homeostatic principle.
    pub principle: String,
    /// The compounding mechanism (optional — not all Laws have this yet).
    pub mechanism: Option<String>,
}

/// A vice or virtue with its Latin name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViceVirtue {
    /// English name.
    pub name: String,
    /// Latin name.
    pub latin: String,
}

// ── CellOutput ──────────────────────────────────────────

/// Output from executing a cell.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CellOutput {
    /// Rendered HTML/text output (for Text, Law, Diagnostic cells).
    Rendered {
        /// The rendered content.
        html: String,
    },
    /// Value output from code execution (PVDSL, Rust).
    Value {
        /// String representation of the return value.
        value: String,
        /// Execution duration in microseconds.
        duration_us: u64,
    },
    /// Terminal output from shell execution.
    Terminal {
        /// Standard output.
        stdout: String,
        /// Standard error.
        stderr: String,
        /// Exit code.
        exit_code: i32,
    },
    /// Error during execution.
    Error {
        /// Error message.
        message: String,
    },
}

// ── CellMetadata ────────────────────────────────────────

/// Cell metadata — timestamps and execution tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMetadata {
    /// When the cell was created (ISO 8601).
    pub created_at: String,
    /// When the cell was last executed (None for never).
    pub last_executed_at: Option<String>,
    /// How many times this cell has been executed.
    pub execution_count: u64,
}

impl Default for CellMetadata {
    fn default() -> Self {
        Self {
            created_at: nexcore_chrono::DateTime::now().to_string(),
            last_executed_at: None,
            execution_count: 0,
        }
    }
}

// ── CellError ───────────────────────────────────────────

/// Errors that can occur during cell construction.
#[derive(Debug)]
pub enum CellError {
    /// Failed to serialize LawContent to JSON source.
    SerializationFailed(String),
}

impl core::fmt::Display for CellError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SerializationFailed(msg) => write!(f, "cell serialization failed: {msg}"),
        }
    }
}

impl std::error::Error for CellError {}

// ── Cell ────────────────────────────────────────────────

/// A single cell in a Crystalbook document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    /// Unique cell identifier.
    pub id: CellId,
    /// What kind of cell this is.
    pub cell_type: CellType,
    /// Raw source content (markdown, code, or JSON for Law cells).
    pub source: String,
    /// Execution output, if any.
    pub output: Option<CellOutput>,
    /// SHA-256 hash of the source bytes (hex-encoded).
    pub content_hash: String,
    /// SHA-256 hash of the output bytes (hex-encoded), if executed.
    pub output_hash: Option<String>,
    /// Cell metadata.
    pub metadata: CellMetadata,
}

impl Cell {
    /// Create a new cell with content-addressed hash.
    #[must_use]
    pub fn new(id: impl Into<String>, cell_type: CellType, source: impl Into<String>) -> Self {
        let source = source.into();
        let content_hash = hash_content(source.as_bytes());
        Self {
            id: CellId::new(id),
            cell_type,
            source,
            output: None,
            content_hash,
            output_hash: None,
            metadata: CellMetadata::default(),
        }
    }

    /// Create a Law cell from structured content (fallible).
    ///
    /// Returns `CellError::SerializationFailed` if the LawContent cannot be
    /// serialized to JSON. This is the honest path — Law I demands it.
    pub fn try_law(id: impl Into<String>, law: &LawContent) -> Result<Self, CellError> {
        let source = serde_json::to_string(law)
            .map_err(|e| CellError::SerializationFailed(e.to_string()))?;
        Ok(Self::new(id, CellType::Law, source))
    }

    /// Create a Law cell from structured content (infallible).
    ///
    /// Serializes the LawContent to JSON. If serialization fails (which should
    /// never happen for well-formed LawContent), the source is set to the debug
    /// representation so the failure is visible, not silent.
    #[must_use]
    pub fn law(id: impl Into<String>, law: &LawContent) -> Self {
        match Self::try_law(id, law) {
            Ok(cell) => cell,
            Err(_) => {
                // Make the failure visible — not silent empty string.
                // Debug repr preserves all data for forensic inspection.
                let fallback = format!("{law:?}");
                Self::new("error-cell", CellType::Law, fallback)
            }
        }
    }

    /// Create a text (markdown) cell.
    #[must_use]
    pub fn text(id: impl Into<String>, markdown: impl Into<String>) -> Self {
        Self::new(id, CellType::Text, markdown)
    }

    /// Recompute the content hash from the current source.
    pub fn rehash(&mut self) {
        self.content_hash = hash_content(self.source.as_bytes());
    }

    /// Compute and set the output hash from the current output.
    pub fn hash_output(&mut self) {
        self.output_hash = self.output.as_ref().map(|o| {
            let bytes = serde_json::to_vec(o).unwrap_or_default();
            hash_content(&bytes)
        });
    }

    /// The combined leaf hash for the Merkle tree: H(content_hash || output_hash).
    #[must_use]
    pub fn leaf_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.content_hash.as_bytes());
        if let Some(ref oh) = self.output_hash {
            hasher.update(oh.as_bytes());
        }
        hasher.finalize()
    }

    /// Parse the source as a `LawContent` (for Law cells).
    /// Returns `None` if the cell is not a Law or the source is malformed.
    #[must_use]
    pub fn as_law(&self) -> Option<LawContent> {
        if self.cell_type != CellType::Law {
            return None;
        }
        serde_json::from_str(&self.source).ok()
    }
}

/// Hash arbitrary bytes and return the hex-encoded SHA-256 digest.
#[must_use]
pub fn hash_content(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    nexcore_codec::hex::encode(&digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_cell_computes_hash() {
        let cell = Cell::text("cell-1", "# Hello World");
        assert!(!cell.content_hash.is_empty());
        assert_eq!(cell.content_hash.len(), 64); // 32 bytes hex = 64 chars
    }

    #[test]
    fn hash_content_known_value() {
        // Known-value check: SHA-256("hello world") per NIST
        let hash = hash_content(b"hello world");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn same_content_produces_same_hash() {
        let a = Cell::text("a", "identical content");
        let b = Cell::text("b", "identical content");
        assert_eq!(a.content_hash, b.content_hash);
    }

    #[test]
    fn different_content_produces_different_hash() {
        let a = Cell::text("a", "content A");
        let b = Cell::text("b", "content B");
        assert_ne!(a.content_hash, b.content_hash);
    }

    #[test]
    fn try_law_succeeds_for_valid_content() {
        let law = test_law();
        let result = Cell::try_law("law-test", &law);
        assert!(result.is_ok());
        let cell = result.unwrap_or_else(|_| panic!("should succeed"));
        assert_eq!(cell.cell_type, CellType::Law);
        assert!(cell.source.contains("superbia"));
    }

    #[test]
    fn law_infallible_produces_valid_cell() {
        let law = test_law();
        let cell = Cell::law("law-i", &law);
        assert_eq!(cell.cell_type, CellType::Law);
        assert!(cell.source.contains("True Measure"));
    }

    #[test]
    fn as_law_roundtrips() {
        let law = test_law();
        let cell = Cell::law("law-i", &law);
        let parsed = cell.as_law();
        assert!(parsed.is_some());
        let back = parsed.unwrap_or_else(|| panic!("should parse"));
        assert_eq!(back.num, "I");
        assert_eq!(back.vice.latin, "superbia");
    }

    #[test]
    fn as_law_returns_none_for_text_cell() {
        let cell = Cell::text("not-a-law", "just prose");
        assert!(cell.as_law().is_none());
    }

    #[test]
    fn rehash_updates_after_source_change() {
        let mut cell = Cell::text("c", "original");
        let original_hash = cell.content_hash.clone();
        cell.source = "modified".to_string();
        cell.rehash();
        assert_ne!(cell.content_hash, original_hash);
    }

    #[test]
    fn leaf_hash_differs_with_output() {
        let mut cell = Cell::text("c", "test");
        let hash_without_output = cell.leaf_hash();
        cell.output = Some(CellOutput::Rendered {
            html: "<p>test</p>".to_string(),
        });
        cell.hash_output();
        let hash_with_output = cell.leaf_hash();
        assert_ne!(hash_without_output, hash_with_output);
    }

    #[test]
    fn cell_serializes_roundtrip() {
        let cell = Cell::text("rt", "roundtrip test");
        let json = serde_json::to_string(&cell).unwrap_or_default();
        let back: Cell = serde_json::from_str(&json).unwrap_or_else(|_| panic!("should parse"));
        assert_eq!(back.content_hash, cell.content_hash);
        assert_eq!(back.id, cell.id);
        assert_eq!(back.cell_type, cell.cell_type);
        assert_eq!(back.source, cell.source);
    }

    #[test]
    fn cell_error_displays() {
        let err = CellError::SerializationFailed("bad json".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("bad json"));
    }

    fn test_law() -> LawContent {
        LawContent {
            num: "I".to_string(),
            title: "The Law of True Measure".to_string(),
            vice: ViceVirtue {
                name: "Pride".to_string(),
                latin: "superbia".to_string(),
            },
            virtue: ViceVirtue {
                name: "Humility".to_string(),
                latin: "humilitas".to_string(),
            },
            deviation: "Unchecked confidence.".to_string(),
            correction: "Honest uncertainty.".to_string(),
            principle: "No internal state shall be exempt from external validation.".to_string(),
            mechanism: Some("Confirmation loop closure.".to_string()),
        }
    }
}
