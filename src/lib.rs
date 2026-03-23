//! # The Crystalbook
//!
//! Immutable, content-addressed scientific documents with Merkle integrity
//! and cryptographic seal chains.
//!
//! ## What This Is
//!
//! The Crystalbook is NexVigilant's founding document — Eight Laws of System
//! Homeostasis by Matthew A. Campion, PharmD. This crate provides the
//! document model, content-addressing, integrity verification, cryptographic
//! sealing, and diagnostic assessment that make the Crystalbook scientifically
//! immutable and transparently verifiable.
//!
//! ## Architecture
//!
//! ```text
//! .crystalbook file (JSON)
//!      │
//!      ▼
//! CrystalbookDocument
//!      ├── cells: Vec<Cell>           ← content-addressed (SHA-256)
//!      ├── merkle_root: String        ← Merkle tree root over cell hashes
//!      ├── seals: SealChain           ← hash-linked immutability chain
//!      └── metadata: DocumentMetadata ← author, version, theme
//! ```
//!
//! Every cell's source is hashed. The hashes form a Merkle tree. The root
//! hash seals the entire document. Seals form a hash-linked chain — each
//! references the previous, making the history tamper-evident.
//!
//! ## Modules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`cell`] | The atom — content-addressed cells with typed content |
//! | [`merkle`] | The lattice — balanced binary Merkle tree with proofs |
//! | [`seal`] | The chain — cryptographic immutability via hash-linked seals |
//! | [`document`] | The container — ties cells + Merkle + seals together |
//! | [`diagnostic`] | The instrument — 8 Laws system health assessment |
//! | [`theme`] | The identity — crystal dark visual constants |
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_crystalbook::document::crystalbook_v2;
//! use nexcore_crystalbook::merkle::MerkleTree;
//!
//! // Build the canonical Crystalbook
//! let mut doc = crystalbook_v2();
//! assert_eq!(doc.cell_count(), 11);
//! assert!(doc.verify_integrity());
//! assert!(doc.validate().is_valid());
//!
//! // Seal it
//! let seal_id = doc.seal("Matthew A. Campion, PharmD");
//! assert!(doc.is_sealed());
//!
//! // Prove a cell's membership
//! let tree = doc.merkle_tree();
//! let proof = tree.proof_for_cell(1).unwrap(); // Law I
//! assert!(MerkleTree::verify_proof(&tree.root(), &proof));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod cell;
pub mod diagnostic;
pub mod document;
pub mod execute;
pub mod io;
pub mod merkle;
pub mod render;
pub mod seal;
pub mod theme;
