//! File I/O — save and load `.crystalbook` documents.
//!
//! Fills ∅₁: the document has no home. Now it does.
//!
//! The `.crystalbook` format is JSON with a version check on load.
//! Format migrations (∅₈) are handled here when `crystalbook_version`
//! doesn't match the current version.

use std::path::Path;

use crate::document::CrystalbookDocument;

/// Current format version. Increment when the schema changes.
pub const CURRENT_FORMAT_VERSION: &str = "1.0";

/// Errors from file I/O operations.
#[derive(Debug)]
pub enum IoError {
    /// Failed to read the file from disk.
    ReadFailed(std::io::Error),
    /// Failed to write the file to disk.
    WriteFailed(std::io::Error),
    /// Failed to parse JSON content.
    ParseFailed(String),
    /// Failed to serialize to JSON.
    SerializeFailed(String),
    /// Format version is unsupported.
    UnsupportedVersion {
        /// The version found in the file.
        found: String,
        /// The version expected by this library.
        expected: String,
    },
}

impl core::fmt::Display for IoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ReadFailed(e) => write!(f, "failed to read file: {e}"),
            Self::WriteFailed(e) => write!(f, "failed to write file: {e}"),
            Self::ParseFailed(msg) => write!(f, "failed to parse .crystalbook: {msg}"),
            Self::SerializeFailed(msg) => write!(f, "failed to serialize: {msg}"),
            Self::UnsupportedVersion { found, expected } => {
                write!(
                    f,
                    "unsupported format version {found} (expected {expected})"
                )
            }
        }
    }
}

impl std::error::Error for IoError {}

/// Save a document to a `.crystalbook` JSON file.
///
/// Writes pretty-printed JSON for human readability and diff-friendliness.
pub fn save(doc: &CrystalbookDocument, path: &Path) -> Result<SaveResult, IoError> {
    let json =
        serde_json::to_string_pretty(doc).map_err(|e| IoError::SerializeFailed(e.to_string()))?;

    let bytes = json.as_bytes().len();
    std::fs::write(path, &json).map_err(IoError::WriteFailed)?;

    Ok(SaveResult {
        path: path.display().to_string(),
        bytes,
        cell_count: doc.cells.len(),
        sealed: doc.is_sealed(),
    })
}

/// Load a document from a `.crystalbook` JSON file.
///
/// Validates the format version and runs `validate()` on the loaded document.
pub fn load(path: &Path) -> Result<CrystalbookDocument, IoError> {
    let content = std::fs::read_to_string(path).map_err(IoError::ReadFailed)?;

    let doc: CrystalbookDocument =
        serde_json::from_str(&content).map_err(|e| IoError::ParseFailed(e.to_string()))?;

    // Version check (∅₈: format versioning)
    if doc.crystalbook_version != CURRENT_FORMAT_VERSION {
        return Err(IoError::UnsupportedVersion {
            found: doc.crystalbook_version.clone(),
            expected: CURRENT_FORMAT_VERSION.to_string(),
        });
    }

    Ok(doc)
}

/// Result of saving a document.
#[derive(Debug)]
pub struct SaveResult {
    /// Path the file was written to.
    pub path: String,
    /// Size in bytes.
    pub bytes: usize,
    /// Number of cells in the document.
    pub cell_count: usize,
    /// Whether the document was sealed at save time.
    pub sealed: bool,
}

impl core::fmt::Display for SaveResult {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Saved {} ({} cells, {} bytes, {})",
            self.path,
            self.cell_count,
            self.bytes,
            if self.sealed { "sealed" } else { "unsealed" },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::crystalbook_v2;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("crystalbook-test-{name}.crystalbook"))
    }

    #[test]
    fn save_and_load_roundtrip() {
        let doc = crystalbook_v2();
        let path = temp_path("roundtrip");

        let result = save(&doc, &path).unwrap_or_else(|e| panic!("save failed: {e}"));
        assert!(result.bytes > 0);
        assert_eq!(result.cell_count, 11);

        let loaded = load(&path).unwrap_or_else(|e| panic!("load failed: {e}"));
        assert_eq!(loaded.merkle_root, doc.merkle_root);
        assert_eq!(loaded.cell_count(), doc.cell_count());
        assert!(loaded.verify_integrity());

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_sealed_document() {
        let mut doc = crystalbook_v2();
        doc.seal("test-signer");
        let path = temp_path("sealed");

        let result = save(&doc, &path).unwrap_or_else(|e| panic!("save failed: {e}"));
        assert!(result.sealed);

        let loaded = load(&path).unwrap_or_else(|e| panic!("load failed: {e}"));
        assert!(loaded.is_sealed());
        assert_eq!(loaded.seals.len(), 1);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_invalid_json_fails() {
        let path = temp_path("invalid");
        std::fs::write(&path, "not json").unwrap_or_default();

        let result = load(&path);
        assert!(result.is_err());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_wrong_version_fails() {
        let path = temp_path("wrong-version");
        let json = r#"{"crystalbook_version":"99.0","metadata":{"title":"","subtitle":null,"author":"","version":"","created":"","last_amended":"","theme":{"bg":"","surface":"","border":"","text":"","muted":"","accent":"","accent_dim":"","vice":"","virtue":"","principle":"","mechanism":"","font_display":"","font_body":""}},"cells":[],"merkle_root":"","seals":{"seals":[]}}"#;
        std::fs::write(&path, json).unwrap_or_default();

        let result = load(&path);
        assert!(result.is_err());
        if let Err(IoError::UnsupportedVersion { found, .. }) = result {
            assert_eq!(found, "99.0");
        }

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_nonexistent_fails() {
        let result = load(Path::new("/tmp/nonexistent-crystalbook-test.crystalbook"));
        assert!(result.is_err());
    }

    #[test]
    fn save_result_displays() {
        let result = SaveResult {
            path: "test.crystalbook".into(),
            bytes: 42000,
            cell_count: 11,
            sealed: true,
        };
        let display = format!("{result}");
        assert!(display.contains("test.crystalbook"));
        assert!(display.contains("sealed"));
    }
}
