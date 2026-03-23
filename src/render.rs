//! Crystalbook HTML renderer — the Voilà replacement.
//!
//! Takes a `CrystalbookDocument` and produces a complete, self-contained
//! HTML page with the crystal dark theme, Cormorant Garamond typography,
//! all Laws rendered as styled cards, and seal chain status.
//!
//! No JavaScript. No external CSS. No dependencies beyond this crate.
//! One function. One file. The output is a static HTML document that
//! can be opened in any browser.
//!
//! ## Pattern
//!
//! Follows `nexcore-transform/render.rs`: push-to-String with escaped content.

use std::fmt::Write;

use crate::cell::{Cell, CellType, LawContent};
use crate::document::CrystalbookDocument;
use crate::theme;

// ── Public API ──────────────────────────────────────────

/// Render a Crystalbook document as a complete, self-contained HTML page.
///
/// The output includes:
/// - Inline CSS with the crystal dark theme
/// - Google Fonts import (Cormorant Garamond + Inter)
/// - All cells rendered according to their type
/// - Integrity badge (Merkle root + seal status)
/// - Print-friendly styles
///
/// # Example
///
/// ```rust
/// use nexcore_crystalbook::document::crystalbook_v2;
/// use nexcore_crystalbook::render::render_to_html;
///
/// let doc = crystalbook_v2();
/// let html = render_to_html(&doc);
/// assert!(html.contains("<!DOCTYPE html>"));
/// assert!(html.contains("True Measure"));
/// ```
#[must_use]
pub fn render_to_html(doc: &CrystalbookDocument) -> String {
    let mut html = String::with_capacity(64 * 1024); // 64KB initial — Crystalbook is ~40KB

    // DOCTYPE + head
    write_head(&mut html, doc);

    // Body open
    html.push_str("<body>\n<div class=\"crystal-page\">\n");

    // Title block
    write_title(&mut html, doc);

    // Integrity badge
    write_integrity_badge(&mut html, doc);

    // Cells
    for cell in &doc.cells {
        write_cell(&mut html, cell);
    }

    // Colophon
    write_colophon(&mut html, doc);

    // Body close
    html.push_str("</div>\n</body>\n</html>\n");

    html
}

/// Render a single cell to an HTML fragment (no wrapping page).
///
/// Useful for embedding individual cells in a larger page (e.g., Nucleus).
#[must_use]
pub fn render_cell_fragment(cell: &Cell) -> String {
    let mut html = String::with_capacity(4096);
    write_cell(&mut html, cell);
    html
}

// ── Head ────────────────────────────────────────────────

fn write_head(html: &mut String, doc: &CrystalbookDocument) {
    let title = escape_html(&doc.metadata.title);
    let author = escape_html(&doc.metadata.author);

    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"utf-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    let _ = write!(html, "<title>{title}</title>\n");
    let _ = write!(html, "<meta name=\"author\" content=\"{author}\">\n");
    html.push_str("<meta name=\"generator\" content=\"nexcore-crystalbook\">\n");

    // Google Fonts
    html.push_str("<link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">\n");
    html.push_str("<link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin>\n");
    html.push_str("<link href=\"https://fonts.googleapis.com/css2?family=Cormorant+Garamond:ital,wght@0,300;0,400;0,500;0,600;0,700;1,400&family=Inter:wght@300;400;500;600&display=swap\" rel=\"stylesheet\">\n");

    // Inline CSS
    html.push_str("<style>\n");
    write_css(html);
    html.push_str("</style>\n");

    html.push_str("</head>\n");
}

// ── CSS ─────────────────────────────────────────────────

fn write_css(html: &mut String) {
    let _ = write!(
        html,
        r#"
:root {{
  --crystal-bg: {bg};
  --crystal-surface: {surface};
  --crystal-border: {border};
  --crystal-text: {text};
  --crystal-muted: {muted};
  --crystal-accent: {accent};
  --crystal-accent-dim: {accent_dim};
  --crystal-vice: {vice};
  --crystal-virtue: {virtue};
  --crystal-principle: {principle};
  --crystal-mechanism: {mechanism};
}}

*, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}

html, body {{
  background: var(--crystal-bg);
  color: var(--crystal-text);
  font-family: '{font_display}', Georgia, serif;
  font-size: 18px;
  line-height: 1.7;
  -webkit-font-smoothing: antialiased;
}}

.crystal-page {{
  max-width: 780px;
  margin: 0 auto;
  padding: 40px 20px 80px;
}}

/* ── Title ────────────────────── */

.crystal-title {{
  text-align: center;
  padding: 60px 0 40px;
  border-bottom: 1px solid var(--crystal-accent-dim);
  margin-bottom: 60px;
}}
.crystal-title h1 {{
  font-size: 2.8em;
  font-weight: 300;
  letter-spacing: 0.15em;
  color: var(--crystal-accent);
  margin-bottom: 8px;
}}
.crystal-title .subtitle {{
  font-size: 1.1em;
  font-style: italic;
  color: var(--crystal-muted);
}}
.crystal-title .meta {{
  font-family: '{font_body}', sans-serif;
  font-size: 0.75em;
  color: var(--crystal-muted);
  margin-top: 16px;
}}

/* ── Integrity Badge ─────────── */

.integrity-badge {{
  font-family: '{font_body}', sans-serif;
  font-size: 0.7em;
  text-align: center;
  padding: 12px 20px;
  margin-bottom: 48px;
  border: 1px solid var(--crystal-border);
  border-radius: 6px;
  color: var(--crystal-muted);
  background: var(--crystal-surface);
}}
.integrity-badge .root {{ font-family: monospace; color: var(--crystal-accent); }}
.integrity-badge .sealed {{ color: var(--crystal-virtue); }}
.integrity-badge .unsealed {{ color: var(--crystal-vice); }}

/* ── Text Cells ──────────────── */

.cell-text {{
  margin-bottom: 48px;
}}
.cell-text p {{
  margin-bottom: 1em;
  text-indent: 0;
}}

/* ── Law Cells ───────────────── */

.cell-law {{
  margin-bottom: 64px;
  page-break-inside: avoid;
}}
.law-header {{
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--crystal-border);
}}
.law-number {{
  font-family: '{font_body}', sans-serif;
  font-size: 0.7em;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--crystal-accent);
  margin-bottom: 4px;
}}
.law-title {{
  font-size: 1.6em;
  font-weight: 400;
  color: var(--crystal-text);
}}

.vice-virtue-row {{
  display: flex;
  gap: 16px;
  margin-bottom: 24px;
}}
.vice-box, .virtue-box {{
  flex: 1;
  padding: 16px 20px;
  border-radius: 4px;
}}
.vice-box {{
  border-left: 3px solid var(--crystal-vice);
  background: rgba(196, 64, 64, 0.06);
}}
.virtue-box {{
  border-left: 3px solid var(--crystal-virtue);
  background: rgba(74, 158, 110, 0.06);
}}
.vv-label {{
  font-family: '{font_body}', sans-serif;
  font-size: 0.7em;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  margin-bottom: 4px;
}}
.vice-box .vv-label {{ color: var(--crystal-vice); }}
.virtue-box .vv-label {{ color: var(--crystal-virtue); }}
.vv-name {{ font-weight: 500; }}
.latin {{ font-style: italic; color: var(--crystal-muted); font-size: 0.9em; }}

.law-section {{
  margin-bottom: 20px;
}}
.law-section-label {{
  font-family: '{font_body}', sans-serif;
  font-size: 0.7em;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  margin-bottom: 8px;
}}
.deviation .law-section-label {{ color: var(--crystal-vice); }}
.correction .law-section-label {{ color: var(--crystal-virtue); }}
.mechanism .law-section-label {{ color: var(--crystal-mechanism); }}

.principle-box {{
  border: 1px solid var(--crystal-principle);
  background: rgba(110, 140, 200, 0.06);
  padding: 20px 24px;
  border-radius: 4px;
  margin-top: 20px;
  font-style: italic;
  color: var(--crystal-principle);
  text-align: center;
  font-size: 1.05em;
}}

/* ── Separator ───────────────── */

.law-separator {{
  height: 1px;
  background: var(--crystal-border);
  margin: 48px auto;
  max-width: 200px;
}}

/* ── Colophon ────────────────── */

.colophon {{
  margin-top: 80px;
  padding-top: 32px;
  border-top: 1px solid var(--crystal-accent-dim);
  text-align: center;
  font-family: '{font_body}', sans-serif;
  font-size: 0.75em;
  color: var(--crystal-muted);
}}
.colophon .gen {{ font-family: monospace; font-size: 0.9em; }}

/* ── Print ───────────────────── */

@media print {{
  html, body {{ background: white; color: #333; font-size: 14px; }}
  .crystal-page {{ max-width: 100%; padding: 0; }}
  .crystal-title h1 {{ color: #333; }}
  .integrity-badge {{ display: none; }}
  .vice-box {{ background: #fff5f5; border-color: #c44040; }}
  .virtue-box {{ background: #f0faf5; border-color: #4a9e6e; }}
  .principle-box {{ background: #f0f4fa; border-color: #6e8cc8; }}
}}

@media (max-width: 600px) {{
  .vice-virtue-row {{ flex-direction: column; gap: 12px; }}
  .crystal-title h1 {{ font-size: 2em; letter-spacing: 0.08em; }}
}}
"#,
        bg = theme::BG,
        surface = theme::SURFACE,
        border = theme::BORDER,
        text = theme::TEXT,
        muted = theme::MUTED,
        accent = theme::ACCENT,
        accent_dim = theme::ACCENT_DIM,
        vice = theme::VICE,
        virtue = theme::VIRTUE,
        principle = theme::PRINCIPLE,
        mechanism = theme::MECHANISM,
        font_display = theme::FONT_DISPLAY,
        font_body = theme::FONT_BODY,
    );
}

// ── Title Block ─────────────────────────────────────────

fn write_title(html: &mut String, doc: &CrystalbookDocument) {
    let title = escape_html(&doc.metadata.title);
    let author = escape_html(&doc.metadata.author);

    html.push_str("<div class=\"crystal-title\">\n");
    let _ = write!(html, "  <h1>{title}</h1>\n");

    if let Some(ref subtitle) = doc.metadata.subtitle {
        let sub = escape_html(subtitle);
        let _ = write!(html, "  <p class=\"subtitle\">{sub}</p>\n");
    }

    let _ = write!(
        html,
        "  <p class=\"meta\">By {author} &middot; v{ver} &middot; Founded {created}</p>\n",
        ver = escape_html(&doc.metadata.version),
        created = escape_html(&doc.metadata.created),
    );

    html.push_str("</div>\n\n");
}

// ── Integrity Badge ─────────────────────────────────────

fn write_integrity_badge(html: &mut String, doc: &CrystalbookDocument) {
    html.push_str("<div class=\"integrity-badge\">\n");

    // Merkle root (truncated)
    let root_short = if doc.merkle_root.len() >= 16 {
        &doc.merkle_root[..16]
    } else {
        &doc.merkle_root
    };
    let _ = write!(
        html,
        "  Merkle Root: <span class=\"root\">{root_short}&hellip;</span> &middot; "
    );
    let _ = write!(html, "{} cells &middot; ", doc.cells.len());

    // Seal status
    if let Some(seal) = doc.seals.latest() {
        let _ = write!(
            html,
            "<span class=\"sealed\">Sealed {}</span> by {}",
            escape_html(&seal.sealed_at),
            escape_html(&seal.signer),
        );
    } else {
        html.push_str("<span class=\"unsealed\">Unsealed</span>");
    }

    html.push_str("\n</div>\n\n");
}

// ── Cell Dispatch ───────────────────────────────────────

fn write_cell(html: &mut String, cell: &Cell) {
    match &cell.cell_type {
        CellType::Law => write_law_cell(html, cell),
        CellType::Text => write_text_cell(html, cell),
        CellType::Diagnostic => write_text_cell(html, cell), // render as prose for now
        CellType::RustCode | CellType::ShellCode | CellType::PvdslCode => {
            write_code_cell(html, cell)
        }
    }
}

// ── Text Cell ───────────────────────────────────────────

fn write_text_cell(html: &mut String, cell: &Cell) {
    html.push_str("<div class=\"cell-text\">\n");

    // Split by double newlines into paragraphs
    for paragraph in cell.source.split("\n\n") {
        let trimmed = paragraph.trim();
        if trimmed.is_empty() {
            continue;
        }
        let escaped = escape_html(trimmed);
        let _ = write!(html, "  <p>{escaped}</p>\n");
    }

    html.push_str("</div>\n\n");
}

// ── Law Cell ────────────────────────────────────────────

fn write_law_cell(html: &mut String, cell: &Cell) {
    let law: LawContent = match serde_json::from_str(&cell.source) {
        Ok(l) => l,
        Err(_) => {
            // Fallback: render source as text if JSON parse fails
            write_text_cell(html, cell);
            return;
        }
    };

    html.push_str("<div class=\"cell-law\">\n");

    // Header
    html.push_str("  <div class=\"law-header\">\n");
    let _ = write!(
        html,
        "    <div class=\"law-number\">Law {}</div>\n",
        escape_html(&law.num)
    );
    let _ = write!(
        html,
        "    <h2 class=\"law-title\">{}</h2>\n",
        escape_html(&law.title)
    );
    html.push_str("  </div>\n\n");

    // Vice / Virtue row
    html.push_str("  <div class=\"vice-virtue-row\">\n");
    html.push_str("    <div class=\"vice-box\">\n");
    html.push_str("      <div class=\"vv-label\">Vice</div>\n");
    let _ = write!(
        html,
        "      <div class=\"vv-name\">{} <span class=\"latin\">({})</span></div>\n",
        escape_html(&law.vice.name),
        escape_html(&law.vice.latin),
    );
    html.push_str("    </div>\n");
    html.push_str("    <div class=\"virtue-box\">\n");
    html.push_str("      <div class=\"vv-label\">Virtue</div>\n");
    let _ = write!(
        html,
        "      <div class=\"vv-name\">{} <span class=\"latin\">({})</span></div>\n",
        escape_html(&law.virtue.name),
        escape_html(&law.virtue.latin),
    );
    html.push_str("    </div>\n");
    html.push_str("  </div>\n\n");

    // Deviation
    html.push_str("  <div class=\"law-section deviation\">\n");
    html.push_str("    <div class=\"law-section-label\">The Deviation</div>\n");
    let _ = write!(html, "    <p>{}</p>\n", escape_html(&law.deviation));
    html.push_str("  </div>\n\n");

    // Correction
    html.push_str("  <div class=\"law-section correction\">\n");
    html.push_str("    <div class=\"law-section-label\">The Correction</div>\n");
    let _ = write!(html, "    <p>{}</p>\n", escape_html(&law.correction));
    html.push_str("  </div>\n\n");

    // Mechanism (if present)
    if let Some(ref mechanism) = law.mechanism {
        html.push_str("  <div class=\"law-section mechanism\">\n");
        html.push_str("    <div class=\"law-section-label\">The Mechanism</div>\n");
        let _ = write!(html, "    <p>{}</p>\n", escape_html(mechanism));
        html.push_str("  </div>\n\n");
    }

    // Principle
    let _ = write!(
        html,
        "  <div class=\"principle-box\">{}</div>\n\n",
        escape_html(&law.principle),
    );

    html.push_str("</div>\n");
    html.push_str("<div class=\"law-separator\"></div>\n\n");
}

// ── Code Cell ───────────────────────────────────────────

fn write_code_cell(html: &mut String, cell: &Cell) {
    let lang = match cell.cell_type {
        CellType::RustCode => "rust",
        CellType::ShellCode => "bash",
        CellType::PvdslCode => "pvdsl",
        _ => "text",
    };

    html.push_str("<div class=\"cell-code\">\n");
    let _ = write!(
        html,
        "  <pre><code class=\"language-{lang}\">{}</code></pre>\n",
        escape_html(&cell.source),
    );

    // Output if present
    if let Some(ref output) = cell.output {
        html.push_str("  <div class=\"cell-output\">\n");
        match output {
            crate::cell::CellOutput::Value { value, duration_us } => {
                let _ = write!(
                    html,
                    "    <pre class=\"output-value\">{}</pre>\n    <span class=\"output-meta\">{}us</span>\n",
                    escape_html(value),
                    duration_us,
                );
            }
            crate::cell::CellOutput::Terminal {
                stdout,
                stderr,
                exit_code,
            } => {
                if !stdout.is_empty() {
                    let _ = write!(
                        html,
                        "    <pre class=\"output-stdout\">{}</pre>\n",
                        escape_html(stdout)
                    );
                }
                if !stderr.is_empty() {
                    let _ = write!(
                        html,
                        "    <pre class=\"output-stderr\">{}</pre>\n",
                        escape_html(stderr)
                    );
                }
                let _ = write!(
                    html,
                    "    <span class=\"output-meta\">exit {exit_code}</span>\n"
                );
            }
            crate::cell::CellOutput::Rendered { html: rendered } => {
                let _ = write!(html, "    {rendered}\n");
            }
            crate::cell::CellOutput::Error { message } => {
                let _ = write!(
                    html,
                    "    <pre class=\"output-error\">{}</pre>\n",
                    escape_html(message)
                );
            }
        }
        html.push_str("  </div>\n");
    }

    html.push_str("</div>\n\n");
}

// ── Colophon ────────────────────────────────────────────

fn write_colophon(html: &mut String, doc: &CrystalbookDocument) {
    html.push_str("<div class=\"colophon\">\n");
    let _ = write!(
        html,
        "  <p>{} &middot; v{} &middot; {}</p>\n",
        escape_html(&doc.metadata.title),
        escape_html(&doc.metadata.version),
        escape_html(&doc.metadata.author),
    );
    let _ = write!(
        html,
        "  <p class=\"gen\">Generated by nexcore-crystalbook &middot; Merkle root: {}</p>\n",
        escape_html(&doc.merkle_root),
    );
    html.push_str("</div>\n");
}

// ── HTML Escaping ───────────────────────────────────────

/// Escape HTML special characters to prevent XSS.
#[must_use]
fn escape_html(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

// ── Tests ───────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::crystalbook_v2;

    #[test]
    fn renders_complete_html_page() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("</html>"));
        assert!(html.contains("<title>THE CRYSTALBOOK</title>"));
    }

    #[test]
    fn contains_all_eight_laws() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        assert!(html.contains("Law I"), "missing Law I");
        assert!(html.contains("Law II"), "missing Law II");
        assert!(html.contains("Law III"), "missing Law III");
        assert!(html.contains("Law IV"), "missing Law IV");
        assert!(html.contains("Law V"), "missing Law V");
        assert!(html.contains("Law VI"), "missing Law VI");
        assert!(html.contains("Law VII"), "missing Law VII");
        assert!(html.contains("Law VIII"), "missing Law VIII");
    }

    #[test]
    fn contains_law_titles() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        assert!(html.contains("True Measure"));
        assert!(html.contains("Sufficient Portion"));
        assert!(html.contains("Bounded Pursuit"));
        assert!(html.contains("Generous Witness"));
        assert!(html.contains("Measured Intake"));
        assert!(html.contains("Measured Response"));
        assert!(html.contains("Active Maintenance"));
        assert!(html.contains("Sovereign Boundary"));
    }

    #[test]
    fn contains_vice_virtue_pairs() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        assert!(html.contains("Pride") && html.contains("superbia"));
        assert!(html.contains("Greed") && html.contains("avaritia"));
        assert!(html.contains("Corruption") && html.contains("corruptio"));
        assert!(html.contains("Humility") && html.contains("humilitas"));
        assert!(html.contains("Independence") && html.contains("libertas"));
    }

    #[test]
    fn contains_crystal_theme_colors() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        assert!(html.contains(theme::BG));
        assert!(html.contains(theme::ACCENT));
        assert!(html.contains(theme::VICE));
        assert!(html.contains(theme::VIRTUE));
        assert!(html.contains(theme::PRINCIPLE));
    }

    #[test]
    fn contains_google_fonts_import() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        assert!(html.contains("Cormorant+Garamond"));
        assert!(html.contains("fonts.googleapis.com"));
    }

    #[test]
    fn contains_integrity_badge() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        assert!(html.contains("Merkle Root:"));
        assert!(html.contains("integrity-badge"));
        assert!(html.contains("Unsealed")); // not sealed yet
    }

    #[test]
    fn sealed_document_shows_sealed_badge() {
        let mut doc = crystalbook_v2();
        doc.seal("Matthew A. Campion, PharmD");
        let html = render_to_html(&doc);

        assert!(html.contains("Sealed"));
        assert!(html.contains("Matthew A. Campion, PharmD"));
        assert!(!html.contains("Unsealed"));
    }

    #[test]
    fn contains_preamble_text() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        assert!(html.contains("Every system that persists does so because it corrects"));
    }

    #[test]
    fn contains_conservation_law() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        assert!(html.contains("Existence = Boundary applied to the Product"));
    }

    #[test]
    fn contains_crystal_oath() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        assert!(html.contains("I will calibrate against reality"));
        assert!(html.contains("physics of persistence"));
    }

    #[test]
    fn contains_colophon() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        assert!(html.contains("nexcore-crystalbook"));
        assert!(html.contains(&doc.merkle_root));
    }

    #[test]
    fn contains_author_metadata() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        assert!(html.contains("Matthew A. Campion, PharmD"));
        assert!(html.contains("v2.0"));
    }

    #[test]
    fn escapes_html_entities() {
        assert_eq!(
            escape_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn cell_fragment_renders_without_page_wrapper() {
        let cell = Cell::text("test", "Fragment content");
        let fragment = render_cell_fragment(&cell);

        assert!(!fragment.contains("<!DOCTYPE"));
        assert!(!fragment.contains("<html"));
        assert!(fragment.contains("Fragment content"));
    }

    #[test]
    fn has_responsive_css() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        assert!(html.contains("@media (max-width: 600px)"));
    }

    #[test]
    fn has_print_css() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        assert!(html.contains("@media print"));
    }

    #[test]
    fn output_size_is_reasonable() {
        let doc = crystalbook_v2();
        let html = render_to_html(&doc);

        // Should be in the 20-60KB range for 11 cells
        assert!(html.len() > 10_000, "HTML too small: {} bytes", html.len());
        assert!(html.len() < 100_000, "HTML too large: {} bytes", html.len());
    }
}
