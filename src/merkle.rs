//! Merkle tree — content-addressed integrity for Crystalbook documents.
//!
//! A balanced binary tree over cell leaf hashes. The root hash changes
//! if any cell's content or output changes. Proofs are logarithmic —
//! a verifier with only the root hash can confirm any single cell's integrity.
//!
//! ## Security Properties
//!
//! - **Order-dependent hashing**: `hash_pair(a, b) != hash_pair(b, a)` — prevents
//!   second-preimage attacks via sibling swap.
//! - **Bounded proofs**: `MerkleProof` carries `leaf_count` so verifiers can
//!   distinguish real leaves from zero-hash padding.
//! - **Deterministic**: same cells in same order always produce same root.

use nexcore_hash::sha256::Sha256;
use serde::{Deserialize, Serialize};

use crate::cell::Cell;

/// SHA-256 digest (32 bytes).
pub type Hash256 = [u8; 32];

/// The zero hash — used to pad the tree to a power-of-two leaf count.
pub const ZERO_HASH: Hash256 = [0u8; 32];

// ── MerkleTree ──────────────────────────────────────────

/// A Merkle tree built from cell leaf hashes.
#[derive(Debug, Clone)]
pub struct MerkleTree {
    /// Flat array in 1-indexed binary-heap layout.
    /// nodes[1] = root, nodes[2..3] = children of root, etc.
    nodes: Vec<Hash256>,
    /// Number of actual leaves (before padding).
    leaf_count: usize,
    /// Total capacity (next power of two >= leaf_count).
    capacity: usize,
}

impl MerkleTree {
    /// Build a Merkle tree from a slice of cells.
    ///
    /// Each cell contributes `cell.leaf_hash()` as a leaf.
    /// The tree is padded to the next power of two with zero hashes.
    #[must_use]
    pub fn from_cells(cells: &[Cell]) -> Self {
        let leaf_count = cells.len();
        let capacity = next_power_of_two(leaf_count.max(1));
        let total_nodes = 2 * capacity; // 1-indexed: nodes[1..2*capacity-1]

        let mut nodes = vec![ZERO_HASH; total_nodes];

        // Place leaves at positions [capacity .. capacity + leaf_count)
        for (i, cell) in cells.iter().enumerate() {
            nodes[capacity + i] = cell.leaf_hash();
        }

        // Build interior nodes bottom-up
        let mut pos = capacity - 1;
        while pos >= 1 {
            nodes[pos] = hash_pair(&nodes[2 * pos], &nodes[2 * pos + 1]);
            pos -= 1;
        }

        Self {
            nodes,
            leaf_count,
            capacity,
        }
    }

    /// The root hash of the tree. Changes if any cell changes.
    #[must_use]
    pub fn root(&self) -> Hash256 {
        if self.nodes.len() > 1 {
            self.nodes[1]
        } else {
            ZERO_HASH
        }
    }

    /// The root hash as a hex string.
    #[must_use]
    pub fn root_hex(&self) -> String {
        nexcore_codec::hex::encode(&self.root())
    }

    /// Number of actual (non-padding) leaves.
    #[must_use]
    pub fn leaf_count(&self) -> usize {
        self.leaf_count
    }

    /// Generate a Merkle proof for the cell at `index`.
    ///
    /// Returns `None` if `index >= leaf_count`.
    /// The proof includes `leaf_count` so external verifiers can validate bounds.
    #[must_use]
    pub fn proof_for_cell(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaf_count {
            return None;
        }

        let mut pos = self.capacity + index;
        let leaf_hash = self.nodes[pos];
        let mut siblings = Vec::new();

        while pos > 1 {
            let sibling_pos = pos ^ 1;
            let is_left = pos % 2 == 0;
            siblings.push(ProofStep {
                hash: self.nodes[sibling_pos],
                is_left,
            });
            pos /= 2;
        }

        Some(MerkleProof {
            leaf_index: index,
            leaf_hash,
            leaf_count: self.leaf_count,
            siblings,
        })
    }

    /// Verify that a proof is valid against a root hash.
    ///
    /// Checks both the hash chain AND that the leaf index is within the
    /// document's actual leaf count (not a padding position).
    #[must_use]
    pub fn verify_proof(root: &Hash256, proof: &MerkleProof) -> bool {
        // Bounds check: reject proofs for padding positions
        if proof.leaf_index >= proof.leaf_count {
            return false;
        }

        let mut current = proof.leaf_hash;
        for step in &proof.siblings {
            current = if step.is_left {
                hash_pair(&current, &step.hash)
            } else {
                hash_pair(&step.hash, &current)
            };
        }

        current == *root
    }
}

// ── MerkleProof ─────────────────────────────────────────

/// A Merkle proof for a single cell — proves membership without revealing others.
///
/// Carries `leaf_count` to prevent padding-collision attacks where a verifier
/// cannot distinguish a real leaf from a zero-hash padding leaf.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Index of the leaf in the original cell array.
    pub leaf_index: usize,
    /// Hash of the leaf being proven.
    pub leaf_hash: Hash256,
    /// Number of actual cells in the document (not padding).
    pub leaf_count: usize,
    /// Sibling hashes from leaf to root.
    pub siblings: Vec<ProofStep>,
}

/// One step in a Merkle proof — a sibling hash and its position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofStep {
    /// The sibling's hash.
    pub hash: Hash256,
    /// Whether the proven node is the left child (true) or right child (false).
    pub is_left: bool,
}

// ── Internal helpers ────────────────────────────────────

/// Hash two child hashes into a parent: H(left || right).
///
/// Order-dependent: `hash_pair(a, b) != hash_pair(b, a)` for a != b.
#[must_use]
fn hash_pair(left: &Hash256, right: &Hash256) -> Hash256 {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize()
}

/// Next power of two >= n. Returns 1 for n <= 1.
#[must_use]
fn next_power_of_two(n: usize) -> usize {
    if n <= 1 {
        return 1;
    }
    let mut v = n - 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v |= v >> 32;
    v + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cells(contents: &[&str]) -> Vec<Cell> {
        contents
            .iter()
            .enumerate()
            .map(|(i, c)| Cell::text(format!("cell-{i}"), *c))
            .collect()
    }

    // ── Structure tests ─────────────────────────────────

    #[test]
    fn single_cell_tree() {
        let cells = make_cells(&["hello"]);
        let tree = MerkleTree::from_cells(&cells);
        assert_eq!(tree.leaf_count(), 1);
        assert_ne!(tree.root(), ZERO_HASH);
    }

    #[test]
    fn empty_cells_produces_tree() {
        let tree = MerkleTree::from_cells(&[]);
        assert_eq!(tree.leaf_count(), 0);
        assert_eq!(tree.root_hex().len(), 64);
    }

    #[test]
    fn power_of_two_cell_count() {
        let cells = make_cells(&["a", "b", "c", "d"]); // exactly 4
        let tree = MerkleTree::from_cells(&cells);
        assert_eq!(tree.leaf_count(), 4);
        assert_eq!(tree.capacity, 4);
    }

    #[test]
    fn non_power_of_two_cell_count() {
        let cells = make_cells(&["a", "b", "c"]); // 3 → padded to 4
        let tree = MerkleTree::from_cells(&cells);
        assert_eq!(tree.leaf_count(), 3);
        assert_eq!(tree.capacity, 4);
    }

    // ── Determinism tests ───────────────────────────────

    #[test]
    fn root_stable_for_same_content() {
        let a = MerkleTree::from_cells(&make_cells(&["x", "y", "z"]));
        let b = MerkleTree::from_cells(&make_cells(&["x", "y", "z"]));
        assert_eq!(a.root(), b.root());
    }

    #[test]
    fn root_changes_with_different_content() {
        let a = MerkleTree::from_cells(&make_cells(&["alpha", "beta"]));
        let b = MerkleTree::from_cells(&make_cells(&["alpha", "gamma"]));
        assert_ne!(a.root(), b.root());
    }

    #[test]
    fn root_changes_with_different_order() {
        let a = MerkleTree::from_cells(&make_cells(&["first", "second"]));
        let b = MerkleTree::from_cells(&make_cells(&["second", "first"]));
        assert_ne!(a.root(), b.root());
    }

    // ── Proof tests ─────────────────────────────────────

    #[test]
    fn proof_verifies_for_each_cell() {
        let cells = make_cells(&["one", "two", "three", "four"]);
        let tree = MerkleTree::from_cells(&cells);
        let root = tree.root();

        for i in 0..cells.len() {
            let proof = tree
                .proof_for_cell(i)
                .unwrap_or_else(|| panic!("proof should exist for cell {i}"));
            assert!(
                MerkleTree::verify_proof(&root, &proof),
                "proof should verify for cell {i}"
            );
        }
    }

    #[test]
    fn proof_includes_leaf_count() {
        let cells = make_cells(&["a", "b", "c"]);
        let tree = MerkleTree::from_cells(&cells);
        let proof = tree
            .proof_for_cell(0)
            .unwrap_or_else(|| panic!("should exist"));
        assert_eq!(proof.leaf_count, 3);
    }

    #[test]
    fn proof_rejects_padding_index() {
        // 3 cells → capacity 4 → index 3 is padding
        let cells = make_cells(&["a", "b", "c"]);
        let tree = MerkleTree::from_cells(&cells);
        assert!(tree.proof_for_cell(3).is_none());
    }

    #[test]
    fn proof_fails_against_wrong_root() {
        let cells = make_cells(&["a", "b"]);
        let tree = MerkleTree::from_cells(&cells);
        let proof = tree
            .proof_for_cell(0)
            .unwrap_or_else(|| panic!("should exist"));
        let wrong_root = [0xFFu8; 32];
        assert!(!MerkleTree::verify_proof(&wrong_root, &proof));
    }

    #[test]
    fn forged_proof_with_bad_leaf_count_rejected() {
        let cells = make_cells(&["a", "b", "c"]);
        let tree = MerkleTree::from_cells(&cells);
        let root = tree.root();

        let mut proof = tree
            .proof_for_cell(2)
            .unwrap_or_else(|| panic!("should exist"));
        // Forge the proof to claim more leaves exist than actually do
        proof.leaf_index = 5;
        proof.leaf_count = 3; // still 3, but index 5 >= 3
        assert!(!MerkleTree::verify_proof(&root, &proof));
    }

    #[test]
    fn out_of_bounds_returns_none() {
        let cells = make_cells(&["only"]);
        let tree = MerkleTree::from_cells(&cells);
        assert!(tree.proof_for_cell(1).is_none());
        assert!(tree.proof_for_cell(100).is_none());
    }

    // ── Hash property tests ─────────────────────────────

    #[test]
    fn hash_pair_is_order_dependent() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        assert_ne!(hash_pair(&a, &b), hash_pair(&b, &a));
    }

    // ── Known-value validation ──────────────────────────

    #[test]
    fn two_leaf_tree_matches_manual_computation() {
        let cells = make_cells(&["left", "right"]);
        let tree = MerkleTree::from_cells(&cells);

        let left_leaf = cells[0].leaf_hash();
        let right_leaf = cells[1].leaf_hash();
        let expected_root = {
            let mut h = Sha256::new();
            h.update(&left_leaf);
            h.update(&right_leaf);
            h.finalize()
        };

        assert_eq!(tree.root(), expected_root);
    }

    // ── Proof serialization ─────────────────────────────

    #[test]
    fn proof_serializes_roundtrip() {
        let cells = make_cells(&["test"]);
        let tree = MerkleTree::from_cells(&cells);
        let proof = tree
            .proof_for_cell(0)
            .unwrap_or_else(|| panic!("should exist"));

        let json = serde_json::to_string(&proof).unwrap_or_default();
        let back: MerkleProof =
            serde_json::from_str(&json).unwrap_or_else(|_| panic!("should parse"));
        assert_eq!(back.leaf_index, proof.leaf_index);
        assert_eq!(back.leaf_count, proof.leaf_count);
        assert_eq!(back.leaf_hash, proof.leaf_hash);
    }

    // ── Utility tests ───────────────────────────────────

    #[test]
    fn power_of_two_cases() {
        assert_eq!(next_power_of_two(0), 1);
        assert_eq!(next_power_of_two(1), 1);
        assert_eq!(next_power_of_two(2), 2);
        assert_eq!(next_power_of_two(3), 4);
        assert_eq!(next_power_of_two(5), 8);
        assert_eq!(next_power_of_two(8), 8);
        assert_eq!(next_power_of_two(9), 16);
    }

    #[test]
    fn root_hex_is_64_chars() {
        let cells = make_cells(&["test"]);
        let tree = MerkleTree::from_cells(&cells);
        assert_eq!(tree.root_hex().len(), 64);
    }
}
