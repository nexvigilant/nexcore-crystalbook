//! Crystal theme constants — single source of truth for all rendering surfaces.
//!
//! These colors originate from the Crystalbook Jupyter notebook's CSS variables
//! and are consumed by Nucleus (via API), Tauri (via IPC), and `render_to_html`.

/// Crystal dark background.
pub const BG: &str = "#0a0a0f";

/// Elevated surface (cards, panels).
pub const SURFACE: &str = "#111118";

/// Subtle borders.
pub const BORDER: &str = "#1e1e2e";

/// Primary text.
pub const TEXT: &str = "#e8e6e3";

/// Muted/secondary text.
pub const MUTED: &str = "#9490a0";

/// Gold accent — the crystal's signature.
pub const ACCENT: &str = "#c9a84c";

/// Dimmed gold for borders and separators.
pub const ACCENT_DIM: &str = "#8a7535";

/// Vice color (deviation, danger).
pub const VICE: &str = "#c44040";

/// Virtue color (correction, health).
pub const VIRTUE: &str = "#4a9e6e";

/// Principle color (homeostatic rules).
pub const PRINCIPLE: &str = "#6e8cc8";

/// Mechanism color (explanatory depth).
pub const MECHANISM: &str = "#9e7ab8";

/// Display font for titles and law headings.
pub const FONT_DISPLAY: &str = "Cormorant Garamond";

/// Body font for prose and code.
pub const FONT_BODY: &str = "Inter";

/// The complete theme as a serializable struct for API consumers.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrystalTheme {
    /// Background color.
    pub bg: String,
    /// Surface color.
    pub surface: String,
    /// Border color.
    pub border: String,
    /// Primary text color.
    pub text: String,
    /// Muted text color.
    pub muted: String,
    /// Gold accent color.
    pub accent: String,
    /// Dimmed accent color.
    pub accent_dim: String,
    /// Vice (deviation) color.
    pub vice: String,
    /// Virtue (correction) color.
    pub virtue: String,
    /// Principle color.
    pub principle: String,
    /// Mechanism color.
    pub mechanism: String,
    /// Display font family.
    pub font_display: String,
    /// Body font family.
    pub font_body: String,
}

impl Default for CrystalTheme {
    fn default() -> Self {
        Self {
            bg: BG.to_string(),
            surface: SURFACE.to_string(),
            border: BORDER.to_string(),
            text: TEXT.to_string(),
            muted: MUTED.to_string(),
            accent: ACCENT.to_string(),
            accent_dim: ACCENT_DIM.to_string(),
            vice: VICE.to_string(),
            virtue: VIRTUE.to_string(),
            principle: PRINCIPLE.to_string(),
            mechanism: MECHANISM.to_string(),
            font_display: FONT_DISPLAY.to_string(),
            font_body: FONT_BODY.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_has_crystal_colors() {
        let theme = CrystalTheme::default();
        assert_eq!(theme.bg, "#0a0a0f");
        assert_eq!(theme.accent, "#c9a84c");
        assert_eq!(theme.vice, "#c44040");
        assert_eq!(theme.virtue, "#4a9e6e");
    }

    #[test]
    fn theme_serializes_to_json() {
        let theme = CrystalTheme::default();
        let json = serde_json::to_string(&theme).unwrap_or_default();
        assert!(json.contains("\"bg\":\"#0a0a0f\""));
        assert!(json.contains("Cormorant Garamond"));
    }
}
