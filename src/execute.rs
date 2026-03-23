//! Cell execution engine — fills ∅₃.
//!
//! Executes cells based on their type. Text and Law cells are "executed"
//! by rendering to HTML. Code cells (Rust, Shell, PVDSL) are executed
//! by their respective backends when available.
//!
//! ## Design
//!
//! The `CellExecutor` trait defines the interface. Implementations:
//! - `StaticExecutor` — renders Text, Law, and Diagnostic cells to HTML (no side effects)
//! - Future: `ShellExecutor`, `PvdslExecutor`, `RustExecutor`

use crate::cell::{Cell, CellOutput, CellType, LawContent};
use crate::render::render_cell_fragment;

/// Errors from cell execution.
#[derive(Debug)]
pub enum ExecuteError {
    /// Cell type is not supported by this executor.
    UnsupportedType(String),
    /// Execution failed with an error message.
    Failed(String),
}

impl core::fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnsupportedType(t) => write!(f, "unsupported cell type: {t}"),
            Self::Failed(msg) => write!(f, "execution failed: {msg}"),
        }
    }
}

impl std::error::Error for ExecuteError {}

/// A cell executor — transforms cell source into output.
pub trait CellExecutor {
    /// Whether this executor can handle the given cell type.
    fn can_execute(&self, cell_type: &CellType) -> bool;

    /// Execute a cell, producing output.
    fn execute(&self, cell: &Cell) -> Result<CellOutput, ExecuteError>;
}

// ── StaticExecutor ──────────────────────────────────────

/// Renders Text, Law, and Diagnostic cells to HTML.
///
/// Pure and deterministic — no side effects, no I/O, no subprocess.
/// "Execution" of a text cell IS its rendering.
pub struct StaticExecutor;

impl CellExecutor for StaticExecutor {
    fn can_execute(&self, cell_type: &CellType) -> bool {
        matches!(
            cell_type,
            CellType::Text | CellType::Law | CellType::Diagnostic
        )
    }

    fn execute(&self, cell: &Cell) -> Result<CellOutput, ExecuteError> {
        match &cell.cell_type {
            CellType::Text | CellType::Diagnostic => {
                let html = render_cell_fragment(cell);
                Ok(CellOutput::Rendered { html })
            }
            CellType::Law => {
                // Parse law content for structured rendering
                let _law: LawContent = serde_json::from_str(&cell.source)
                    .map_err(|e| ExecuteError::Failed(format!("invalid law content: {e}")))?;
                let html = render_cell_fragment(cell);
                Ok(CellOutput::Rendered { html })
            }
            other => Err(ExecuteError::UnsupportedType(format!("{other:?}"))),
        }
    }
}

// ── Document-level execution ────────────────────────────

/// Execute all executable cells in a document using the given executor.
///
/// Returns the number of cells executed. Cells whose type the executor
/// cannot handle are skipped. Each executed cell gets its output hash updated.
pub fn execute_all(
    cells: &mut [Cell],
    executor: &dyn CellExecutor,
) -> Result<ExecuteResult, ExecuteError> {
    let mut executed = 0;
    let mut skipped = 0;
    let mut errors = Vec::new();

    for cell in cells.iter_mut() {
        if !executor.can_execute(&cell.cell_type) {
            skipped += 1;
            continue;
        }

        match executor.execute(cell) {
            Ok(output) => {
                cell.output = Some(output);
                cell.hash_output();
                cell.metadata.execution_count += 1;
                cell.metadata.last_executed_at = Some(nexcore_chrono::DateTime::now().to_string());
                executed += 1;
            }
            Err(e) => {
                errors.push(format!("Cell {}: {e}", cell.id));
                cell.output = Some(CellOutput::Error {
                    message: e.to_string(),
                });
                cell.hash_output();
            }
        }
    }

    Ok(ExecuteResult {
        executed,
        skipped,
        errors,
    })
}

/// Result of executing cells in a document.
#[derive(Debug)]
pub struct ExecuteResult {
    /// Number of cells successfully executed.
    pub executed: usize,
    /// Number of cells skipped (unsupported type).
    pub skipped: usize,
    /// Error messages from failed cells.
    pub errors: Vec<String>,
}

impl core::fmt::Display for ExecuteResult {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Executed {} cells, skipped {}, {} errors",
            self.executed,
            self.skipped,
            self.errors.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;
    use crate::document::crystalbook_v2;

    #[test]
    fn static_executor_renders_text() {
        let cell = Cell::text("t", "Hello world");
        let executor = StaticExecutor;
        assert!(executor.can_execute(&cell.cell_type));

        let output = executor
            .execute(&cell)
            .unwrap_or_else(|e| panic!("failed: {e}"));
        if let CellOutput::Rendered { html } = output {
            assert!(html.contains("Hello world"));
        } else {
            panic!("expected Rendered output");
        }
    }

    #[test]
    fn static_executor_renders_law() {
        let doc = crystalbook_v2();
        let law_cell = doc
            .cell_by_id("law-i")
            .unwrap_or_else(|| panic!("missing law-i"));
        let executor = StaticExecutor;
        assert!(executor.can_execute(&law_cell.cell_type));

        let output = executor
            .execute(law_cell)
            .unwrap_or_else(|e| panic!("failed: {e}"));
        if let CellOutput::Rendered { html } = output {
            assert!(html.contains("True Measure"));
            assert!(html.contains("Pride"));
        } else {
            panic!("expected Rendered output");
        }
    }

    #[test]
    fn static_executor_rejects_code() {
        let cell = Cell::new("c", CellType::RustCode, "fn main() {}");
        let executor = StaticExecutor;
        assert!(!executor.can_execute(&cell.cell_type));

        let result = executor.execute(&cell);
        assert!(result.is_err());
    }

    #[test]
    fn execute_all_on_crystalbook() {
        let mut doc = crystalbook_v2();
        let executor = StaticExecutor;

        let result =
            execute_all(&mut doc.cells, &executor).unwrap_or_else(|e| panic!("failed: {e}"));

        // All 11 cells are Text or Law — all should execute
        assert_eq!(result.executed, 11);
        assert_eq!(result.skipped, 0);
        assert!(result.errors.is_empty());

        // Every cell should now have output
        for cell in &doc.cells {
            assert!(cell.output.is_some(), "Cell {} has no output", cell.id);
            assert!(
                cell.output_hash.is_some(),
                "Cell {} has no output hash",
                cell.id
            );
            assert_eq!(cell.metadata.execution_count, 1);
            assert!(cell.metadata.last_executed_at.is_some());
        }
    }

    #[test]
    fn execute_all_skips_code_cells() {
        let mut cells = vec![
            Cell::text("t", "text cell"),
            Cell::new("c", CellType::ShellCode, "ls -la"),
        ];
        let executor = StaticExecutor;

        let result = execute_all(&mut cells, &executor).unwrap_or_else(|e| panic!("failed: {e}"));

        assert_eq!(result.executed, 1);
        assert_eq!(result.skipped, 1);
    }

    #[test]
    fn executed_cells_have_updated_metadata() {
        let mut cells = vec![Cell::text("t", "test")];
        let executor = StaticExecutor;

        execute_all(&mut cells, &executor).unwrap_or_else(|e| panic!("failed: {e}"));

        assert_eq!(cells[0].metadata.execution_count, 1);
        assert!(cells[0].metadata.last_executed_at.is_some());

        // Execute again
        execute_all(&mut cells, &executor).unwrap_or_else(|e| panic!("failed: {e}"));
        assert_eq!(cells[0].metadata.execution_count, 2);
    }

    #[test]
    fn execute_error_displays() {
        let err = ExecuteError::UnsupportedType("ShellCode".into());
        assert!(format!("{err}").contains("ShellCode"));
    }

    #[test]
    fn execute_result_displays() {
        let result = ExecuteResult {
            executed: 11,
            skipped: 0,
            errors: vec![],
        };
        let s = format!("{result}");
        assert!(s.contains("11"));
        assert!(s.contains("0 errors"));
    }
}
