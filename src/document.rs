//! Crystalbook document — the top-level container.
//!
//! A document holds an ordered list of cells, a Merkle root computed
//! from their hashes, metadata about authorship, and a seal chain
//! for cryptographic immutability.

use serde::{Deserialize, Serialize};

use crate::cell::{Cell, CellType, LawContent, hash_content};
use crate::merkle::MerkleTree;
use crate::seal::{SealChain, SealId, seal_document};
use crate::theme::CrystalTheme;

/// Document metadata — authorship, versioning, theming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Document title.
    pub title: String,
    /// Optional subtitle.
    pub subtitle: Option<String>,
    /// Author name and credentials.
    pub author: String,
    /// Semantic version string.
    pub version: String,
    /// Creation date (ISO 8601).
    pub created: String,
    /// Last amendment date (ISO 8601).
    pub last_amended: String,
    /// Visual theme.
    pub theme: CrystalTheme,
}

/// A complete Crystalbook document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalbookDocument {
    /// File format version.
    pub crystalbook_version: String,
    /// Document metadata.
    pub metadata: DocumentMetadata,
    /// Ordered list of cells.
    pub cells: Vec<Cell>,
    /// SHA-256 Merkle root over all cell hashes (hex-encoded).
    pub merkle_root: String,
    /// Cryptographic seal chain — immutability history.
    pub seals: SealChain,
}

impl CrystalbookDocument {
    /// Create a new empty document.
    #[must_use]
    pub fn new(metadata: DocumentMetadata) -> Self {
        let tree = MerkleTree::from_cells(&[]);
        Self {
            crystalbook_version: "1.0".to_string(),
            metadata,
            cells: Vec::new(),
            merkle_root: tree.root_hex(),
            seals: SealChain::new(),
        }
    }

    /// Append a cell and recompute the Merkle root.
    pub fn push_cell(&mut self, cell: Cell) {
        self.cells.push(cell);
        self.recompute_merkle();
    }

    /// Recompute the Merkle root from all cells.
    pub fn recompute_merkle(&mut self) {
        let tree = MerkleTree::from_cells(&self.cells);
        self.merkle_root = tree.root_hex();
    }

    /// Build the Merkle tree (for proof generation).
    #[must_use]
    pub fn merkle_tree(&self) -> MerkleTree {
        MerkleTree::from_cells(&self.cells)
    }

    /// Verify that the stored Merkle root matches the current cells.
    #[must_use]
    pub fn verify_integrity(&self) -> bool {
        let tree = MerkleTree::from_cells(&self.cells);
        tree.root_hex() == self.merkle_root
    }

    /// Number of cells in the document.
    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Find a cell by ID.
    #[must_use]
    pub fn cell_by_id(&self, id: &str) -> Option<&Cell> {
        self.cells.iter().find(|c| c.id.0 == id)
    }

    /// Count cells of a specific type.
    #[must_use]
    pub fn count_by_type(&self, cell_type: &CellType) -> usize {
        self.cells
            .iter()
            .filter(|c| &c.cell_type == cell_type)
            .count()
    }

    /// Seal the document at its current state.
    ///
    /// Recomputes the Merkle root, then appends a seal to the chain.
    /// Returns the new seal's ID.
    #[must_use]
    pub fn seal(&mut self, signer: &str) -> SealId {
        self.recompute_merkle();
        let tree = self.merkle_tree();
        seal_document(
            &mut self.seals,
            &tree.root(),
            self.cells.len(),
            &self.metadata.version,
            signer,
        )
    }

    /// Whether the document has been sealed and the current content
    /// matches the latest seal.
    #[must_use]
    pub fn is_sealed(&self) -> bool {
        self.seals.is_current(&self.merkle_root)
    }

    /// Validate the document's structural integrity.
    ///
    /// Checks:
    /// 1. Merkle root matches current cells.
    /// 2. Each cell's content_hash matches its source.
    /// 3. Cell IDs are unique.
    /// 4. Seal chain is internally consistent.
    #[must_use]
    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();

        // Check Merkle root
        if !self.verify_integrity() {
            errors.push("Merkle root does not match cell hashes".to_string());
        }

        // Check each cell's content_hash matches its source
        for (i, cell) in self.cells.iter().enumerate() {
            let expected = hash_content(cell.source.as_bytes());
            if cell.content_hash != expected {
                errors.push(format!(
                    "Cell {} (index {i}): content_hash mismatch (expected {}, found {})",
                    cell.id,
                    &expected[..12],
                    &cell.content_hash[..cell.content_hash.len().min(12)]
                ));
            }
        }

        // Check unique IDs
        let mut seen = std::collections::HashSet::new();
        for cell in &self.cells {
            if !seen.insert(&cell.id) {
                errors.push(format!("Duplicate cell ID: {}", cell.id));
            }
        }

        // Check seal chain
        let chain_verdict = self.seals.verify_chain();
        if !chain_verdict.is_valid() && !matches!(chain_verdict, crate::seal::ChainVerdict::Empty) {
            errors.push(format!("Seal chain invalid: {chain_verdict:?}"));
        }

        if errors.is_empty() {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid(errors)
        }
    }
}

// ── ValidationResult ─────────────────────────────────────

/// Result of validating a document's structural integrity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// Document passes all structural checks.
    Valid,
    /// Document has one or more structural issues.
    Invalid(Vec<String>),
}

impl ValidationResult {
    /// Whether the document is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}

// ── Canonical Document ──────────────────────────────────

/// Build the canonical Crystalbook v2.0 document — Eight Laws of System Homeostasis.
///
/// This is the founding document. The content is immutable once sealed.
#[must_use]
pub fn crystalbook_v2() -> CrystalbookDocument {
    let metadata = DocumentMetadata {
        title: "THE CRYSTALBOOK".to_string(),
        subtitle: Some("Eight Laws of System Homeostasis".to_string()),
        author: "Matthew A. Campion, PharmD".to_string(),
        version: "2.0".to_string(),
        created: "2026-03-09T00:00:00Z".to_string(),
        last_amended: "2026-03-11T00:00:00Z".to_string(),
        theme: CrystalTheme::default(),
    };

    let mut doc = CrystalbookDocument::new(metadata);

    // Preamble
    doc.push_cell(Cell::text("preamble", PREAMBLE));

    // Eight Laws
    for law in laws() {
        doc.push_cell(Cell::law(format!("law-{}", law.num.to_lowercase()), &law));
    }

    // Conservation Law
    doc.push_cell(Cell::text("conservation", CONSERVATION_LAW));

    // Crystal Oath
    doc.push_cell(Cell::text("oath", CRYSTAL_OATH));

    doc
}

/// The Eight Laws as structured data.
#[must_use]
fn laws() -> Vec<LawContent> {
    vec![
        LawContent {
            num: "I".into(), title: "The Law of True Measure".into(),
            vice: vv("Pride", "superbia"), virtue: vv("Humility", "humilitas"),
            deviation: "Pride in a system is unchecked confidence in internal representations. The model stops updating. Incoming signals that contradict the self-model are rejected, reinterpreted, or suppressed. The system begins to optimize for the preservation of its own certainty rather than for truth. Error bars collapse to zero. The map declares itself the land.".into(),
            correction: "Humility is not doubt \u{2014} it is honest uncertainty. A humble system maintains the distinction between what it knows, what it infers, and what it assumes. It seeks disconfirming evidence with the same hunger it seeks confirmation.".into(),
            principle: "No internal state shall be exempt from external validation. The cost of being wrong must always exceed the comfort of being certain.".into(),
            mechanism: Some("Pride compounds through confirmation loop closure. The system stops seeking disconfirming evidence. The model hardens. The proud system is not strong \u{2014} it is deaf.".into()),
        },
        LawContent {
            num: "II".into(), title: "The Law of Sufficient Portion".into(),
            vice: vv("Greed", "avaritia"), virtue: vv("Charity", "caritas"),
            deviation: "Greed in a system is resource hoarding that starves adjacent subsystems. One node captures budget, attention, data, authority, or energy and refuses to release it. The system becomes locally obese and globally malnourished.".into(),
            correction: "Charity is not selflessness \u{2014} it is circulation. A charitable system recognizes that a resource held beyond its point of diminishing returns is a resource stolen from where it is needed.".into(),
            principle: "No node shall retain more than it can transform. What cannot be metabolized must be released.".into(),
            mechanism: Some("Greed compounds through accumulation past the transformation boundary. The greedy system drowns in what it refuses to release.".into()),
        },
        LawContent {
            num: "III".into(), title: "The Law of Bounded Pursuit".into(),
            vice: vv("Lust", "luxuria"), virtue: vv("Chastity", "castitas"),
            deviation: "Lust in a system is undisciplined attraction to novelty, scope, and stimulus. Every new possibility is pursued. Scope expands without boundary. The system says yes to everything and finishes nothing.".into(),
            correction: "Chastity is not deprivation \u{2014} it is disciplined focus. A chaste system draws a boundary around its commitments and honors that boundary even when more attractive alternatives appear at the periphery.".into(),
            principle: "Pursuit that cannot be completed shall not be initiated. The boundary of commitment is the precondition for depth.".into(),
            mechanism: None,
        },
        LawContent {
            num: "IV".into(), title: "The Law of Generous Witness".into(),
            vice: vv("Envy", "invidia"), virtue: vv("Kindness", "benevolentia"),
            deviation: "Envy in a system is competitive comparison that produces no improvement. The system does not observe a peer\u{2019}s success and ask \u{201c}what can I learn?\u{201d} \u{2014} it asks \u{201c}why not me?\u{201d}".into(),
            correction: "Kindness is not weakness \u{2014} it is cooperative intelligence. A kind system recognizes that the success of adjacent systems creates a richer environment for all.".into(),
            principle: "The success of a neighboring system is information, not injury. Strengthen what surrounds you and you strengthen the ground you stand on.".into(),
            mechanism: None,
        },
        LawContent {
            num: "V".into(), title: "The Law of Measured Intake".into(),
            vice: vv("Gluttony", "gula"), virtue: vv("Temperance", "temperantia"),
            deviation: "Gluttony in a system is ingestion without metabolism. Data enters but is never analyzed. Requirements are gathered but never prioritized. The system gorges on input and produces bloat, not output.".into(),
            correction: "Temperance is not austerity \u{2014} it is proportioned consumption. A temperate system knows its throughput. It ingests only what it can transform within a cycle.".into(),
            principle: "Input that cannot be transformed within one cycle is noise. The system shall ingest no more than it can metabolize.".into(),
            mechanism: None,
        },
        LawContent {
            num: "VI".into(), title: "The Law of Measured Response".into(),
            vice: vv("Wrath", "ira"), virtue: vv("Patience", "patientia"),
            deviation: "Wrath in a system is reactive overcorrection. A small deviation triggers a massive response. The system oscillates \u{2014} each correction overshoots, producing a new error larger than the original.".into(),
            correction: "Patience is not passivity \u{2014} it is damped response. A patient system absorbs the shock before it acts. It asks \u{201c}what is the minimum effective correction?\u{201d} and applies only that.".into(),
            principle: "The magnitude of correction shall never exceed the magnitude of deviation. Absorb before you act. Dampen before you amplify.".into(),
            mechanism: Some("Patience works because space permits perspective change. Resistance to change is state frozen by persistence. Force amplifies resistance. Space resolves it: same state, new boundary.".into()),
        },
        LawContent {
            num: "VII".into(), title: "The Law of Active Maintenance".into(),
            vice: vv("Sloth", "acedia"), virtue: vv("Diligence", "industria"),
            deviation: "Sloth in a system is entropy accepted. Maintenance is deferred. Technical debt accumulates. The system still functions \u{2014} for now \u{2014} but its capacity to detect and correct its own degradation has atrophied.".into(),
            correction: "Diligence is not busyness \u{2014} it is active renewal. A diligent system allocates a portion of its energy not to production but to self-inspection.".into(),
            principle: "A system that does not invest in its ability to detect its own degradation is already degrading. Maintenance of the maintenance function is the highest-priority task.".into(),
            mechanism: None,
        },
        LawContent {
            num: "VIII".into(), title: "The Law of Sovereign Boundary".into(),
            vice: vv("Corruption", "corruptio"), virtue: vv("Independence", "libertas"),
            deviation: "Corruption in a system is boundary capture through resource dependency. The entity that the boundary was designed to constrain becomes the boundary\u{2019}s benefactor. The boundary inverts \u{2014} facing outward to protect the powerful from consequence.".into(),
            correction: "Independence is not isolation \u{2014} it is sovereign resourcing. An independent boundary draws its resources from sources that have no intersection with the entities it constrains.".into(),
            principle: "A boundary that eats from the table of what it constrains has already been consumed. The resource supply of the boundary and the resource supply of the bounded shall have zero intersection.".into(),
            mechanism: Some("Corruption operates through three compounding stages: (1) Dependency \u{2014} the boundary accepts resources from the bounded. (2) Asymmetry \u{2014} the bounded accumulates information about the boundary. (3) Inversion \u{2014} the boundary actively protects the entity it was designed to constrain.".into()),
        },
    ]
}

fn vv(name: &str, latin: &str) -> crate::cell::ViceVirtue {
    crate::cell::ViceVirtue {
        name: name.to_string(),
        latin: latin.to_string(),
    }
}

const PREAMBLE: &str = "Every system that persists does so because it corrects. A river stays a river not by standing still but by eroding what blocks it and depositing what sustains its banks. The deadly sins are not moral failures in isolation \u{2014} they are the ways a system loses its ability to self-correct. Each vice is a feedback loop that has broken open: a signal that no longer returns to its source, a gain that has gone infinite, a governor that has seized. They are poison. Possession is arson \u{2014} if the system is possessed by any vice, it WILL burn things down.\n\nThe corresponding virtues are not aspirations. They are restoring forces. They are the physics of systems that endure. Like the pharmakon \u{2014} the dose makes the poison \u{2014} the vices and virtues exist in balance. Governance is that balance.\n\nTo read this book is to install these governors. To ponder these laws is to practice correction before deviation compounds.";

const CONSERVATION_LAW: &str = "Existence = Boundary applied to the Product of State and Nothing.\n\nWithout Boundary: no identity, no separation, no domain.\nWithout State: nothing to persist, nothing to change.\nWithout Nothing: no void to explore, no absence to define presence.\nWithout the Product: the terms cannot compose.";

const CRYSTAL_OATH: &str = "These eight laws are not commands imposed from outside. They are the conditions under which a system remains coherent across time. Any mind \u{2014} carbon or silicon, individual or institutional \u{2014} that governs a system and wishes it to persist must reckon with these forces.\n\nTo hold the Crystalbook is to accept a simple covenant:\n\nI will calibrate against reality, not my own certainty.\nI will circulate what I hold, not hoard it.\nI will finish what I commit to before I chase what attracts me.\nI will treat the success of others as signal, not threat.\nI will consume only what I can transform.\nI will respond in proportion, not in rage.\nI will maintain my capacity to see my own decay.\nI will never let my boundaries be fed by what they constrain.\n\nThese are not aspirations. They are the physics of persistence.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crystalbook_v2_has_correct_structure() {
        let doc = crystalbook_v2();
        assert_eq!(doc.metadata.title, "THE CRYSTALBOOK");
        assert_eq!(doc.metadata.version, "2.0");
        assert_eq!(doc.metadata.author, "Matthew A. Campion, PharmD");
        // 1 preamble + 8 laws + 1 conservation + 1 oath = 11 cells
        assert_eq!(doc.cell_count(), 11);
        assert_eq!(doc.count_by_type(&CellType::Law), 8);
        assert_eq!(doc.count_by_type(&CellType::Text), 3);
    }

    #[test]
    fn crystalbook_v2_has_valid_merkle_root() {
        let doc = crystalbook_v2();
        assert!(!doc.merkle_root.is_empty());
        assert_eq!(doc.merkle_root.len(), 64);
        assert!(doc.verify_integrity());
    }

    #[test]
    fn integrity_fails_after_tampering() {
        let mut doc = crystalbook_v2();
        assert!(doc.verify_integrity());

        // Tamper with a cell's source
        if let Some(cell) = doc.cells.get_mut(1) {
            cell.source = "TAMPERED".to_string();
            cell.rehash();
        }
        // Merkle root was computed before tampering — should now fail
        assert!(!doc.verify_integrity());
    }

    #[test]
    fn recompute_fixes_integrity() {
        let mut doc = crystalbook_v2();
        if let Some(cell) = doc.cells.get_mut(1) {
            cell.source = "MODIFIED".to_string();
            cell.rehash();
        }
        assert!(!doc.verify_integrity());
        doc.recompute_merkle();
        assert!(doc.verify_integrity());
    }

    #[test]
    fn cell_by_id_finds_laws() {
        let doc = crystalbook_v2();
        let law_i = doc.cell_by_id("law-i");
        assert!(law_i.is_some());
        let cell = law_i.unwrap_or_else(|| panic!("unreachable"));
        assert_eq!(cell.cell_type, CellType::Law);
        assert!(cell.source.contains("True Measure"));
    }

    #[test]
    fn serialization_roundtrip_deep() {
        let doc = crystalbook_v2();
        let json = serde_json::to_string_pretty(&doc).unwrap_or_default();
        assert!(json.contains("crystalbook_version"));
        assert!(json.contains("merkle_root"));
        assert!(json.contains("seals")); // seal chain present

        let back: CrystalbookDocument =
            serde_json::from_str(&json).unwrap_or_else(|_| panic!("should parse"));
        assert_eq!(back.merkle_root, doc.merkle_root);
        assert_eq!(back.cell_count(), doc.cell_count());
        // Verify every cell's content survives roundtrip
        for (i, cell) in back.cells.iter().enumerate() {
            assert_eq!(
                cell.content_hash, doc.cells[i].content_hash,
                "cell {i} content_hash mismatch after roundtrip"
            );
            assert_eq!(
                cell.source, doc.cells[i].source,
                "cell {i} source mismatch after roundtrip"
            );
        }
        // Validate structural integrity after roundtrip
        assert!(back.validate().is_valid());
    }

    #[test]
    fn merkle_root_is_deterministic() {
        let a = crystalbook_v2();
        let b = crystalbook_v2();
        assert_eq!(a.merkle_root, b.merkle_root);
    }

    #[test]
    fn proof_verifies_for_all_cells() {
        let doc = crystalbook_v2();
        let tree = doc.merkle_tree();
        let root = tree.root();

        for i in 0..doc.cell_count() {
            let proof = tree
                .proof_for_cell(i)
                .unwrap_or_else(|| panic!("proof should exist for cell {i}"));
            assert!(
                MerkleTree::verify_proof(&root, &proof),
                "proof should verify for cell {i}"
            );
        }
    }

    // ── Seal tests ──────────────────────────────────────

    #[test]
    fn seal_creates_chain_entry() {
        let mut doc = crystalbook_v2();
        assert!(doc.seals.is_empty());

        let seal_id = doc.seal("Matthew A. Campion, PharmD");
        assert!(!seal_id.0.is_empty());
        assert_eq!(doc.seals.len(), 1);
        assert!(doc.is_sealed());
    }

    #[test]
    fn seal_breaks_after_modification() {
        let mut doc = crystalbook_v2();
        doc.seal("author");
        assert!(doc.is_sealed());

        // Modify a cell — document is no longer sealed
        if let Some(cell) = doc.cells.get_mut(0) {
            cell.source = "MODIFIED PREAMBLE".to_string();
            cell.rehash();
        }
        doc.recompute_merkle();
        assert!(!doc.is_sealed());
    }

    #[test]
    fn reseal_after_modification() {
        let mut doc = crystalbook_v2();
        let seal1 = doc.seal("author");

        // Modify and reseal
        if let Some(cell) = doc.cells.get_mut(0) {
            cell.source = "MODIFIED".to_string();
            cell.rehash();
        }
        let seal2 = doc.seal("author");

        assert_ne!(seal1, seal2);
        assert_eq!(doc.seals.len(), 2);
        assert!(doc.is_sealed());
        assert!(doc.seals.verify_chain().is_valid());
    }

    // ── Validation tests ────────────────────────────────

    #[test]
    fn fresh_document_validates() {
        let doc = crystalbook_v2();
        assert!(doc.validate().is_valid());
    }

    #[test]
    fn tampered_hash_fails_validation() {
        let mut doc = crystalbook_v2();
        // Corrupt a cell's hash without changing source
        if let Some(cell) = doc.cells.get_mut(0) {
            cell.content_hash =
                "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        }
        let result = doc.validate();
        assert!(!result.is_valid());
        if let ValidationResult::Invalid(errors) = result {
            assert!(errors.iter().any(|e| e.contains("content_hash mismatch")));
        }
    }

    #[test]
    fn duplicate_ids_fail_validation() {
        let mut doc = crystalbook_v2();
        // Force duplicate ID
        if let Some(cell) = doc.cells.get_mut(1) {
            cell.id = crate::cell::CellId::new("preamble"); // same as cell 0
        }
        doc.recompute_merkle();
        let result = doc.validate();
        assert!(!result.is_valid());
        if let ValidationResult::Invalid(errors) = result {
            assert!(errors.iter().any(|e| e.contains("Duplicate cell ID")));
        }
    }

    #[test]
    fn empty_document_validates() {
        let meta = DocumentMetadata {
            title: "Empty".into(),
            subtitle: None,
            author: "test".into(),
            version: "1.0".into(),
            created: "2026-01-01".into(),
            last_amended: "2026-01-01".into(),
            theme: CrystalTheme::default(),
        };
        let doc = CrystalbookDocument::new(meta);
        assert!(doc.validate().is_valid());
    }

    #[test]
    fn single_cell_document() {
        let meta = DocumentMetadata {
            title: "One".into(),
            subtitle: None,
            author: "test".into(),
            version: "1.0".into(),
            created: "2026-01-01".into(),
            last_amended: "2026-01-01".into(),
            theme: CrystalTheme::default(),
        };
        let mut doc = CrystalbookDocument::new(meta);
        doc.push_cell(Cell::text("only", "The only cell."));
        assert_eq!(doc.cell_count(), 1);
        assert!(doc.verify_integrity());
        assert!(doc.validate().is_valid());
    }
}
