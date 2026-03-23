//! Document seals — cryptographic immutability for Crystalbook documents.
//!
//! A seal captures the Merkle root at a point in time, creating an
//! immutable snapshot. Seals form a hash-linked chain: each seal references
//! the previous, making the history tamper-evident.
//!
//! ## Design
//!
//! The seal chain is the Crystalbook's equivalent of a blockchain — but
//! for a single document owned by a single author. No consensus needed,
//! no mining, no tokens. Just: "at this moment, the document had this hash,
//! and I vouch for it."
//!
//! ## Integrity Properties
//!
//! - **Forward-linked**: each seal hashes the previous seal's ID, creating
//!   an append-only chain. Removing a seal from the middle breaks the chain.
//! - **Tamper-evident**: modifying a sealed document changes its Merkle root,
//!   which no longer matches any seal in the chain.
//! - **Auditable**: the chain records when and by whom each version was sealed.

use serde::{Deserialize, Serialize};

use crate::cell::hash_content;
use crate::merkle::Hash256;

// ── SealId ──────────────────────────────────────────────

/// Unique identifier for a document seal — derived from its content hash.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SealId(pub String);

impl SealId {
    /// The null seal ID — used as the "previous" for the first seal in a chain.
    #[must_use]
    pub fn genesis() -> Self {
        Self("genesis".to_string())
    }
}

impl core::fmt::Display for SealId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── DocumentSeal ────────────────────────────────────────

/// A cryptographic seal capturing a document's state at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSeal {
    /// Unique seal identifier (SHA-256 of seal contents).
    pub seal_id: SealId,
    /// Merkle root of the document at seal time (hex-encoded).
    pub merkle_root: String,
    /// When the seal was created (ISO 8601).
    pub sealed_at: String,
    /// Who sealed the document (author name or system ID).
    pub signer: String,
    /// Number of cells in the document at seal time.
    pub cell_count: usize,
    /// Document version at seal time.
    pub document_version: String,
    /// ID of the previous seal in the chain (genesis for first seal).
    pub previous_seal: SealId,
}

impl DocumentSeal {
    /// Compute the seal ID from its content. The ID is deterministic:
    /// same content always produces the same ID.
    #[must_use]
    pub fn compute_id(
        merkle_root: &str,
        sealed_at: &str,
        signer: &str,
        previous: &SealId,
    ) -> SealId {
        let material = format!("{merkle_root}|{sealed_at}|{signer}|{}", previous.0);
        SealId(hash_content(material.as_bytes()))
    }

    /// Short display of the seal ID (first 12 hex chars).
    #[must_use]
    pub fn short_id(&self) -> &str {
        if self.seal_id.0.len() >= 12 {
            &self.seal_id.0[..12]
        } else {
            &self.seal_id.0
        }
    }
}

// ── SealChain ───────────────────────────────────────────

/// An append-only chain of document seals.
///
/// Each seal links to the previous via `previous_seal`, forming a
/// tamper-evident history. The chain validates that:
/// 1. Each seal's ID matches its content hash.
/// 2. Each seal's `previous_seal` matches the prior seal's ID.
/// 3. The first seal links to `genesis`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SealChain {
    /// Ordered seals — oldest first, newest last.
    seals: Vec<DocumentSeal>,
}

impl SealChain {
    /// Create a new empty seal chain.
    #[must_use]
    pub fn new() -> Self {
        Self { seals: Vec::new() }
    }

    /// Seal a document at its current state.
    ///
    /// Appends a new seal to the chain, linking it to the previous seal.
    /// Returns the new seal's ID.
    #[must_use]
    pub fn seal(
        &mut self,
        merkle_root: String,
        cell_count: usize,
        document_version: String,
        signer: impl Into<String>,
    ) -> SealId {
        let signer = signer.into();
        let sealed_at = nexcore_chrono::DateTime::now().to_string();
        let previous = self.latest_id();

        let seal_id = DocumentSeal::compute_id(&merkle_root, &sealed_at, &signer, &previous);

        let seal = DocumentSeal {
            seal_id: seal_id.clone(),
            merkle_root,
            sealed_at,
            signer,
            cell_count,
            document_version,
            previous_seal: previous,
        };

        self.seals.push(seal);
        seal_id
    }

    /// The latest seal's ID, or genesis if the chain is empty.
    #[must_use]
    pub fn latest_id(&self) -> SealId {
        self.seals
            .last()
            .map(|s| s.seal_id.clone())
            .unwrap_or_else(SealId::genesis)
    }

    /// The latest seal, if any.
    #[must_use]
    pub fn latest(&self) -> Option<&DocumentSeal> {
        self.seals.last()
    }

    /// All seals in chronological order.
    #[must_use]
    pub fn all(&self) -> &[DocumentSeal] {
        &self.seals
    }

    /// Number of seals in the chain.
    #[must_use]
    pub fn len(&self) -> usize {
        self.seals.len()
    }

    /// Whether the chain has no seals.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.seals.is_empty()
    }

    /// Verify the integrity of the seal chain.
    ///
    /// Checks that:
    /// 1. Each seal's ID matches its content hash (not forged).
    /// 2. Each seal links to the correct previous seal.
    /// 3. The first seal links to genesis.
    #[must_use]
    pub fn verify_chain(&self) -> ChainVerdict {
        if self.seals.is_empty() {
            return ChainVerdict::Empty;
        }

        let mut expected_previous = SealId::genesis();

        for (i, seal) in self.seals.iter().enumerate() {
            // Check previous link
            if seal.previous_seal != expected_previous {
                return ChainVerdict::BrokenLink {
                    seal_index: i,
                    expected: expected_previous,
                    found: seal.previous_seal.clone(),
                };
            }

            // Check seal ID integrity (recompute from content)
            let recomputed = DocumentSeal::compute_id(
                &seal.merkle_root,
                &seal.sealed_at,
                &seal.signer,
                &seal.previous_seal,
            );
            if seal.seal_id != recomputed {
                return ChainVerdict::ForgedSeal {
                    seal_index: i,
                    claimed: seal.seal_id.clone(),
                    recomputed,
                };
            }

            expected_previous = seal.seal_id.clone();
        }

        ChainVerdict::Valid
    }

    /// Check whether a given Merkle root matches the latest seal.
    ///
    /// Returns `true` if the document hasn't changed since the last seal.
    #[must_use]
    pub fn is_current(&self, merkle_root: &str) -> bool {
        self.latest()
            .map(|s| s.merkle_root == merkle_root)
            .unwrap_or(false)
    }
}

// ── ChainVerdict ────────────────────────────────────────

/// Result of verifying a seal chain's integrity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainVerdict {
    /// Chain is valid — all links and IDs check out.
    Valid,
    /// Chain is empty — no seals to verify.
    Empty,
    /// A link in the chain is broken (previous_seal doesn't match).
    BrokenLink {
        /// Index of the broken seal.
        seal_index: usize,
        /// Expected previous seal ID.
        expected: SealId,
        /// Actual previous seal ID found.
        found: SealId,
    },
    /// A seal's ID doesn't match its content hash (tampered).
    ForgedSeal {
        /// Index of the forged seal.
        seal_index: usize,
        /// The claimed seal ID.
        claimed: SealId,
        /// The recomputed seal ID.
        recomputed: SealId,
    },
}

impl ChainVerdict {
    /// Whether the chain is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}

// ── Convenience: seal a document ────────────────────────

/// Seal a Merkle root into a chain, returning the new seal ID.
///
/// This is the primary entry point for sealing a document.
#[must_use]
pub fn seal_document(
    chain: &mut SealChain,
    merkle_root: &Hash256,
    cell_count: usize,
    document_version: &str,
    signer: &str,
) -> SealId {
    let root_hex = nexcore_codec::hex::encode(merkle_root);
    chain.seal(root_hex, cell_count, document_version.to_string(), signer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_chain_starts_empty() {
        let chain = SealChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
        assert!(chain.latest().is_none());
        assert_eq!(chain.latest_id(), SealId::genesis());
    }

    #[test]
    fn single_seal() {
        let mut chain = SealChain::new();
        let id = chain.seal("abc123".into(), 11, "2.0".into(), "Matthew Campion");
        assert_eq!(chain.len(), 1);
        assert!(!id.0.is_empty());
        assert_eq!(chain.latest_id(), id);
    }

    #[test]
    fn chain_links_correctly() {
        let mut chain = SealChain::new();
        let id1 = chain.seal("root1".into(), 11, "2.0".into(), "author");
        let _id2 = chain.seal("root2".into(), 12, "2.1".into(), "author");

        let seals = chain.all();
        assert_eq!(seals[0].previous_seal, SealId::genesis());
        assert_eq!(seals[1].previous_seal, id1);
    }

    #[test]
    fn verify_valid_chain() {
        let mut chain = SealChain::new();
        chain.seal("root1".into(), 11, "2.0".into(), "author");
        chain.seal("root2".into(), 12, "2.1".into(), "author");
        chain.seal("root3".into(), 13, "2.2".into(), "author");

        assert_eq!(chain.verify_chain(), ChainVerdict::Valid);
    }

    #[test]
    fn verify_empty_chain() {
        let chain = SealChain::new();
        assert_eq!(chain.verify_chain(), ChainVerdict::Empty);
    }

    #[test]
    fn verify_detects_broken_link() {
        let mut chain = SealChain::new();
        chain.seal("root1".into(), 11, "2.0".into(), "author");
        chain.seal("root2".into(), 12, "2.1".into(), "author");

        // Forge: change the second seal's previous link
        chain.seals[1].previous_seal = SealId("forged".to_string());

        match chain.verify_chain() {
            ChainVerdict::BrokenLink { seal_index, .. } => assert_eq!(seal_index, 1),
            other => panic!("expected BrokenLink, got {other:?}"),
        }
    }

    #[test]
    fn verify_detects_forged_seal() {
        let mut chain = SealChain::new();
        chain.seal("root1".into(), 11, "2.0".into(), "author");

        // Forge: change the seal ID without changing content
        chain.seals[0].seal_id = SealId("forged-id".to_string());

        match chain.verify_chain() {
            ChainVerdict::ForgedSeal { seal_index, .. } => assert_eq!(seal_index, 0),
            other => panic!("expected ForgedSeal, got {other:?}"),
        }
    }

    #[test]
    fn is_current_tracks_latest_root() {
        let mut chain = SealChain::new();
        chain.seal("root-v1".into(), 11, "2.0".into(), "author");
        assert!(chain.is_current("root-v1"));
        assert!(!chain.is_current("root-v2"));

        chain.seal("root-v2".into(), 12, "2.1".into(), "author");
        assert!(!chain.is_current("root-v1"));
        assert!(chain.is_current("root-v2"));
    }

    #[test]
    fn seal_id_is_deterministic() {
        let id1 = DocumentSeal::compute_id("root", "2026-03-21", "author", &SealId::genesis());
        let id2 = DocumentSeal::compute_id("root", "2026-03-21", "author", &SealId::genesis());
        assert_eq!(id1, id2);
    }

    #[test]
    fn seal_id_changes_with_different_signer() {
        let id1 = DocumentSeal::compute_id("root", "2026-03-21", "alice", &SealId::genesis());
        let id2 = DocumentSeal::compute_id("root", "2026-03-21", "bob", &SealId::genesis());
        assert_ne!(id1, id2);
    }

    #[test]
    fn short_id_is_12_chars() {
        let mut chain = SealChain::new();
        chain.seal("root".into(), 11, "2.0".into(), "author");
        let seal = chain.latest().unwrap_or_else(|| panic!("should have seal"));
        assert_eq!(seal.short_id().len(), 12);
    }

    #[test]
    fn seal_document_convenience() {
        let mut chain = SealChain::new();
        let root = [0xABu8; 32];
        let id = seal_document(&mut chain, &root, 11, "2.0", "author");
        assert!(!id.0.is_empty());
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn chain_serializes_roundtrip() {
        let mut chain = SealChain::new();
        chain.seal("root1".into(), 11, "2.0".into(), "author");
        chain.seal("root2".into(), 12, "2.1".into(), "author");

        let json = serde_json::to_string(&chain).unwrap_or_default();
        let back: SealChain =
            serde_json::from_str(&json).unwrap_or_else(|_| panic!("should parse"));
        assert_eq!(back.len(), 2);
        assert_eq!(back.verify_chain(), ChainVerdict::Valid);
    }
}
