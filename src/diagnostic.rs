//! Crystalbook Diagnostic — the 8 Laws as a system health assessment instrument.
//!
//! Port of the TypeScript `crystalbook-diagnostic.ts` data model to Rust.
//! This is the authoritative source — the Nucleus frontend consumes this
//! via API rather than maintaining a parallel copy.
//!
//! ## Usage
//!
//! ```rust
//! use nexcore_crystalbook::diagnostic::{questions, score, LawStatus};
//!
//! let answers = vec![
//!     ("I", LawStatus::Healthy),
//!     ("II", LawStatus::AtRisk),
//!     ("III", LawStatus::Healthy),
//!     ("IV", LawStatus::Healthy),
//!     ("V", LawStatus::Violated),
//!     ("VI", LawStatus::Healthy),
//!     ("VII", LawStatus::AtRisk),
//!     ("VIII", LawStatus::Healthy),
//! ];
//!
//! let result = score(&answers);
//! assert_eq!(result.grade, "Under Stress");
//! ```

use serde::{Deserialize, Serialize};

// ── LawStatus ───────────────────────────────────────────

/// Health status of a single Law in a system assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LawStatus {
    /// The system satisfies this Law.
    Healthy,
    /// The system shows signs of deviation but hasn't fully violated the Law.
    AtRisk,
    /// The system has broken this Law's homeostatic principle.
    Violated,
}

impl core::fmt::Display for LawStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Healthy => f.write_str("Healthy"),
            Self::AtRisk => f.write_str("At Risk"),
            Self::Violated => f.write_str("Violated"),
        }
    }
}

// ── DiagnosticQuestion ──────────────────────────────────

/// One diagnostic question — maps a Law to a plain-English health check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticQuestion {
    /// Roman numeral of the Law (I–VIII).
    pub law_num: String,
    /// Law title.
    pub law_title: String,
    /// Vice name.
    pub vice: String,
    /// Virtue name.
    pub virtue: String,
    /// The diagnostic question in plain English.
    pub question: String,
    /// Brief description of what this question assesses.
    pub description: String,
    /// What "Healthy" looks like for this Law.
    pub healthy_signal: String,
    /// What "At Risk" looks like for this Law.
    pub risk_signal: String,
    /// What "Violated" looks like for this Law.
    pub violated_signal: String,
}

/// The eight diagnostic questions — one per Law.
#[must_use]
pub fn questions() -> Vec<DiagnosticQuestion> {
    vec![
        DiagnosticQuestion {
            law_num: "I".into(),
            law_title: "True Measure".into(),
            vice: "Pride".into(),
            virtue: "Humility".into(),
            question: "Is your system honest about what it doesn't know?".into(),
            description: "Systems fail when they stop validating their own assumptions. Confidence without measurement is the first crack.".into(),
            healthy_signal: "We regularly test our assumptions against reality".into(),
            risk_signal: "We sometimes operate on untested beliefs".into(),
            violated_signal: "We trust our internal model more than external feedback".into(),
        },
        DiagnosticQuestion {
            law_num: "II".into(),
            law_title: "Sufficient Portion".into(),
            vice: "Greed".into(),
            virtue: "Charity".into(),
            question: "Does information and authority flow freely?".into(),
            description: "When one part of a system hoards resources, data, or decision-making power, the rest starves. Circulation is health.".into(),
            healthy_signal: "Resources reach where they're needed".into(),
            risk_signal: "Some bottlenecks slow the flow".into(),
            violated_signal: "Key resources are concentrated and don't reach those who need them".into(),
        },
        DiagnosticQuestion {
            law_num: "III".into(),
            law_title: "Bounded Pursuit".into(),
            vice: "Lust".into(),
            virtue: "Chastity".into(),
            question: "Does your system finish what it starts?".into(),
            description: "Chasing every opportunity while completing none is how systems scatter their energy. Depth requires the refusal of breadth.".into(),
            healthy_signal: "We complete commitments before taking on new ones".into(),
            risk_signal: "We sometimes overextend into new initiatives".into(),
            violated_signal: "We're stretched across too many incomplete efforts".into(),
        },
        DiagnosticQuestion {
            law_num: "IV".into(),
            law_title: "Generous Witness".into(),
            vice: "Envy".into(),
            virtue: "Kindness".into(),
            question: "Do you learn from others' success?".into(),
            description: "When a neighbor's success feels like a threat instead of a signal, collaboration dies. The ecosystem is a commons, not an arena.".into(),
            healthy_signal: "We study what works elsewhere and adapt it".into(),
            risk_signal: "We occasionally feel threatened by peer success".into(),
            violated_signal: "Competitor success triggers defensive reactions, not learning".into(),
        },
        DiagnosticQuestion {
            law_num: "V".into(),
            law_title: "Measured Intake".into(),
            vice: "Gluttony".into(),
            virtue: "Temperance".into(),
            question: "Can your system process what it takes in?".into(),
            description: "Ingesting data, requirements, and meetings without transforming them into decisions is bloat. Signal-to-noise degrades when everything is kept.".into(),
            healthy_signal: "We process inputs within a reasonable cycle".into(),
            risk_signal: "Backlogs are growing faster than we resolve them".into(),
            violated_signal: "We accumulate far more than we can act on".into(),
        },
        DiagnosticQuestion {
            law_num: "VI".into(),
            law_title: "Measured Response".into(),
            vice: "Wrath".into(),
            virtue: "Patience".into(),
            question: "Are your reactions proportionate?".into(),
            description: "When a small problem triggers a massive response, the correction becomes worse than the deviation. Absorb before you act.".into(),
            healthy_signal: "We calibrate responses to the size of the issue".into(),
            risk_signal: "We sometimes overreact to minor setbacks".into(),
            violated_signal: "Incident response often creates more disruption than the original problem".into(),
        },
        DiagnosticQuestion {
            law_num: "VII".into(),
            law_title: "Active Maintenance".into(),
            vice: "Sloth".into(),
            virtue: "Diligence".into(),
            question: "Does your system check its own health?".into(),
            description: "A system that stops inspecting itself is already degrading. By the time collapse is visible, the mechanisms that could have prevented it have rusted shut.".into(),
            healthy_signal: "We actively monitor our own processes and fix issues early".into(),
            risk_signal: "Health checks happen but aren't always acted on".into(),
            violated_signal: "We don't have reliable ways to detect our own degradation".into(),
        },
        DiagnosticQuestion {
            law_num: "VIII".into(),
            law_title: "Sovereign Boundary".into(),
            vice: "Corruption".into(),
            virtue: "Independence".into(),
            question: "Are your boundaries independent from what they constrain?".into(),
            description: "When the entity a boundary was designed to oversee becomes its benefactor, the boundary inverts \u{2014} protecting the powerful instead of constraining them.".into(),
            healthy_signal: "Our oversight functions operate independently".into(),
            risk_signal: "Some oversight depends on the entities being overseen".into(),
            violated_signal: "Those who should be constrained have influence over their own constraints".into(),
        },
    ]
}

// ── Scoring ─────────────────────────────────────────────

/// The result of scoring a diagnostic assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticScore {
    /// Number of Laws in healthy state.
    pub healthy: usize,
    /// Number of Laws at risk.
    pub at_risk: usize,
    /// Number of Laws violated.
    pub violated: usize,
    /// Overall grade: Resilient, Stable, Under Stress, Critical.
    pub grade: String,
}

/// Score a set of diagnostic answers.
///
/// Each answer is a `(law_num, status)` tuple. Unrecognized law numbers are ignored.
#[must_use]
pub fn score(answers: &[(&str, LawStatus)]) -> DiagnosticScore {
    let healthy = answers
        .iter()
        .filter(|(_, s)| *s == LawStatus::Healthy)
        .count();
    let at_risk = answers
        .iter()
        .filter(|(_, s)| *s == LawStatus::AtRisk)
        .count();
    let violated = answers
        .iter()
        .filter(|(_, s)| *s == LawStatus::Violated)
        .count();

    let grade = if violated == 0 && at_risk <= 1 {
        "Resilient"
    } else if violated <= 1 && at_risk <= 3 {
        "Stable"
    } else if violated <= 3 {
        "Under Stress"
    } else {
        "Critical"
    };

    DiagnosticScore {
        healthy,
        at_risk,
        violated,
        grade: grade.to_string(),
    }
}

// ── Conservation Check ──────────────────────────────────

/// The conservation law table — maps each Law to what it breaks in the
/// conservation equation `∃ = ∂(×(ς, ∅))`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservationRow {
    /// Law title.
    pub law: String,
    /// Vice name.
    pub vice: String,
    /// What the vice breaks in the conservation equation.
    pub breaks: String,
}

/// The 8-row conservation table.
#[must_use]
pub fn conservation_table() -> Vec<ConservationRow> {
    vec![
        ConservationRow {
            law: "I. True Measure".into(),
            vice: "Pride".into(),
            breaks:
                "Claims Existence without Boundary \u{2014} asserts identity without measurement"
                    .into(),
        },
        ConservationRow {
            law: "II. Sufficient Portion".into(),
            vice: "Greed".into(),
            breaks:
                "Inflates State beyond Boundary \u{2014} hoards past the domain\u{2019}s capacity"
                    .into(),
        },
        ConservationRow {
            law: "III. Bounded Pursuit".into(),
            vice: "Lust".into(),
            breaks: "Dissolves Boundary \u{2014} chases beyond commitment".into(),
        },
        ConservationRow {
            law: "IV. Generous Witness".into(),
            vice: "Envy".into(),
            breaks:
                "Imports foreign Boundary without comparison \u{2014} adopts others\u{2019} domains"
                    .into(),
        },
        ConservationRow {
            law: "V. Measured Intake".into(),
            vice: "Gluttony".into(),
            breaks: "State ingested exceeds transformation capacity \u{2014} bloat".into(),
        },
        ConservationRow {
            law: "VI. Measured Response".into(),
            vice: "Wrath".into(),
            breaks: "Irreversible action without causal understanding \u{2014} overcorrection"
                .into(),
        },
        ConservationRow {
            law: "VII. Active Maintenance".into(),
            vice: "Sloth".into(),
            breaks: "Skips Existence verification \u{2014} assumes persistence without checking"
                .into(),
        },
        ConservationRow {
            law: "VIII. Sovereign Boundary".into(),
            vice: "Corruption".into(),
            breaks:
                "Boundary captured by external dependency \u{2014} inverts to protect the bounded"
                    .into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn questions_returns_eight() {
        assert_eq!(questions().len(), 8);
    }

    #[test]
    fn questions_cover_all_laws() {
        let qs = questions();
        let nums: Vec<&str> = qs.iter().map(|q| q.law_num.as_str()).collect();
        assert_eq!(nums, vec!["I", "II", "III", "IV", "V", "VI", "VII", "VIII"]);
    }

    #[test]
    fn each_question_has_three_signals() {
        for q in questions() {
            assert!(
                !q.healthy_signal.is_empty(),
                "Law {} missing healthy signal",
                q.law_num
            );
            assert!(
                !q.risk_signal.is_empty(),
                "Law {} missing risk signal",
                q.law_num
            );
            assert!(
                !q.violated_signal.is_empty(),
                "Law {} missing violated signal",
                q.law_num
            );
        }
    }

    #[test]
    fn score_all_healthy() {
        let answers: Vec<(&str, LawStatus)> = (1..=8).map(|_| ("I", LawStatus::Healthy)).collect();
        let result = score(&answers);
        assert_eq!(result.grade, "Resilient");
        assert_eq!(result.healthy, 8);
        assert_eq!(result.violated, 0);
    }

    #[test]
    fn score_all_violated() {
        let answers: Vec<(&str, LawStatus)> = (1..=8).map(|_| ("I", LawStatus::Violated)).collect();
        let result = score(&answers);
        assert_eq!(result.grade, "Critical");
        assert_eq!(result.violated, 8);
    }

    #[test]
    fn score_mixed_under_stress() {
        let answers = vec![
            ("I", LawStatus::Healthy),
            ("II", LawStatus::AtRisk),
            ("III", LawStatus::Healthy),
            ("IV", LawStatus::Healthy),
            ("V", LawStatus::Violated),
            ("VI", LawStatus::Healthy),
            ("VII", LawStatus::Violated),
            ("VIII", LawStatus::Healthy),
        ];
        let result = score(&answers);
        assert_eq!(result.grade, "Under Stress");
    }

    #[test]
    fn score_stable_with_one_violation() {
        let answers = vec![
            ("I", LawStatus::Healthy),
            ("II", LawStatus::AtRisk),
            ("III", LawStatus::Healthy),
            ("IV", LawStatus::Healthy),
            ("V", LawStatus::Violated),
            ("VI", LawStatus::Healthy),
            ("VII", LawStatus::Healthy),
            ("VIII", LawStatus::Healthy),
        ];
        let result = score(&answers);
        assert_eq!(result.grade, "Stable");
    }

    #[test]
    fn law_status_display() {
        assert_eq!(format!("{}", LawStatus::Healthy), "Healthy");
        assert_eq!(format!("{}", LawStatus::AtRisk), "At Risk");
        assert_eq!(format!("{}", LawStatus::Violated), "Violated");
    }

    #[test]
    fn conservation_table_has_eight_rows() {
        assert_eq!(conservation_table().len(), 8);
    }

    #[test]
    fn conservation_table_matches_laws() {
        let table = conservation_table();
        assert!(table[0].law.contains("True Measure"));
        assert!(table[7].law.contains("Sovereign Boundary"));
        assert_eq!(table[7].vice, "Corruption");
    }

    #[test]
    fn diagnostic_score_serializes() {
        let result = score(&[("I", LawStatus::Healthy)]);
        let json = serde_json::to_string(&result).unwrap_or_default();
        assert!(json.contains("Resilient"));
    }

    #[test]
    fn law_status_serde_roundtrip() {
        let status = LawStatus::AtRisk;
        let json = serde_json::to_string(&status).unwrap_or_default();
        assert_eq!(json, "\"at-risk\"");
        let back: LawStatus =
            serde_json::from_str(&json).unwrap_or_else(|_| panic!("should parse"));
        assert_eq!(back, LawStatus::AtRisk);
    }
}
