//! Theme system controlling all visual defaults.
//!
//! A plot rendered with zero custom styling must look professional. The
//! [`Theme::default()`] configuration follows the Visual Design Brief:
//!
//! - White background, despined axes (top + right hidden), light grid behind data.
//! - Tableau-10 categorical palette, viridis continuous colormap.
//! - Outward ticks, readable font sizes (title 14 pt bold, labels 11 pt, ticks 9 pt).
//!
//! Additional built-in themes are available via [`Theme::dark()`],
//! [`Theme::seaborn()`], [`Theme::ggplot()`], and [`Theme::publication()`].

use crate::primitives::{Color, FontWeight};

// ---------------------------------------------------------------------------
// LineStyle
// ---------------------------------------------------------------------------

/// Line drawing style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LineStyle {
    /// A continuous solid line.
    Solid,
    /// A dashed line (e.g. `[6, 4]`).
    Dashed,
    /// A dotted line (e.g. `[2, 2]`).
    Dotted,
    /// Alternating long dash and dot (e.g. `[6, 3, 2, 3]`).
    DashDot,
}

// ---------------------------------------------------------------------------
// Marker
// ---------------------------------------------------------------------------

/// Scatter plot marker shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Marker {
    /// Filled circle.
    Circle,
    /// Filled square.
    Square,
    /// Filled upward-pointing triangle.
    Triangle,
    /// Filled diamond (rotated square).
    Diamond,
    /// Axis-aligned plus sign (stroked, not filled).
    Plus,
    /// Diagonal cross / X (stroked, not filled).
    Cross,
    /// Five-pointed star.
    Star,
    /// A single pixel-sized point (smallest possible marker).
    Point,
}

// ---------------------------------------------------------------------------
// Loc (legend location)
// ---------------------------------------------------------------------------

/// Legend location relative to the axes area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Loc {
    /// Automatically choose the location that overlaps the fewest data points.
    Best,
    /// Upper-right corner.
    UpperRight,
    /// Upper-left corner.
    UpperLeft,
    /// Lower-left corner.
    LowerLeft,
    /// Lower-right corner.
    LowerRight,
    /// Centered on the right edge.
    Right,
    /// Centered on the left edge.
    CenterLeft,
    /// Centered on the right edge (alias kept for symmetry).
    CenterRight,
    /// Centered on the bottom edge.
    LowerCenter,
    /// Centered on the top edge.
    UpperCenter,
    /// Dead center of the axes area.
    Center,
}

// ---------------------------------------------------------------------------
// GridAxis
// ---------------------------------------------------------------------------

/// Which axes should display grid lines.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GridAxis {
    /// Grid lines for the x-axis only (vertical lines at each x-tick).
    X,
    /// Grid lines for the y-axis only (horizontal lines at each y-tick).
    Y,
    /// Grid lines for both axes (the default).
    #[default]
    Both,
}

// ---------------------------------------------------------------------------
// TickDirection
// ---------------------------------------------------------------------------

/// Direction in which axis tick marks extend from the spine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickDirection {
    /// Ticks extend outward, away from the data area.
    Outward,
    /// Ticks extend inward, into the data area.
    Inward,
}

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

/// Visual theme controlling all rendering defaults.
///
/// Every visual parameter that a renderer or layout engine might need lives
/// here. Chart builders read from the active theme to fill in any value the
/// user did not override explicitly.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Background color for the entire figure (outside the axes area).
    pub figure_background: Color,
    /// Background color for the axes face (the data-drawing region).
    pub axes_background: Color,

    // -- Grid ---------------------------------------------------------------
    /// Color of major grid lines.
    pub grid_color: Color,
    /// Width (in px) of major grid lines.
    pub grid_width: f64,
    /// Whether the grid is shown by default (line/scatter: true; bar/hist: false).
    pub show_grid: bool,

    // -- Spines -------------------------------------------------------------
    /// Color of visible axis spines.
    pub spine_color: Color,
    /// Width (in px) of axis spines.
    pub spine_width: f64,
    /// Whether the top spine is drawn.
    pub show_top_spine: bool,
    /// Whether the right spine is drawn.
    pub show_right_spine: bool,
    /// Whether the bottom spine is drawn.
    pub show_bottom_spine: bool,
    /// Whether the left spine is drawn.
    pub show_left_spine: bool,

    // -- Ticks --------------------------------------------------------------
    /// Color of tick marks and tick labels.
    pub tick_color: Color,
    /// Length (in px) of major tick marks.
    pub tick_length: f64,
    /// Direction ticks extend from the spine.
    pub tick_direction: TickDirection,
    /// Font size (in pt) for tick labels.
    pub tick_label_size: f64,

    // -- Labels & Title -----------------------------------------------------
    /// Font size (in pt) for axis labels.
    pub axis_label_size: f64,
    /// Font size (in pt) for the plot title.
    pub title_size: f64,
    /// Font weight for the plot title.
    pub title_weight: FontWeight,
    /// Color used for all text (titles, labels, tick labels).
    pub text_color: Color,

    // -- Data elements ------------------------------------------------------
    /// Default line width (in px) for line plots.
    pub line_width: f64,
    /// Default marker diameter (in px) for scatter plots.
    pub marker_size: f64,
    /// Default marker opacity (0.0 = fully transparent, 1.0 = fully opaque).
    pub marker_alpha: f64,

    // -- Palette ------------------------------------------------------------
    /// Categorical color cycle used when the user does not specify colors.
    pub color_cycle: Vec<Color>,

    // -- Font ---------------------------------------------------------------
    /// Optional font family override. `None` means the renderer picks its
    /// built-in default (typically a clean sans-serif such as Helvetica).
    pub font_family: Option<String>,
}

// ---------------------------------------------------------------------------
// Tableau-10 palette (convenience constant)
// ---------------------------------------------------------------------------

/// The Tableau-10 categorical palette as a fixed-size array.
const TABLEAU_10: [Color; 10] = Color::TABLEAU_10;

// ---------------------------------------------------------------------------
// Default theme
// ---------------------------------------------------------------------------

impl Default for Theme {
    /// Returns the canonical default theme matching the Visual Design Brief.
    ///
    /// - Background: `#FFFFFF`, axes face: `#FFFFFF`
    /// - Grid: `#E6E6E6`, 1 px, shown by default
    /// - Spines: `#333333`, 1 px, top + right hidden (despine look)
    /// - Ticks: outward, 4 px, `#333333`
    /// - Font sizes: title 14 pt bold, axis labels 11 pt, tick labels 9 pt
    /// - Text color: `#333333`
    /// - Line width 1.5 px, marker 6 px diameter, marker alpha 0.8
    /// - Tableau-10 color cycle
    fn default() -> Self {
        let spine = Color::rgb(0x33, 0x33, 0x33);

        Self {
            figure_background: Color::WHITE,
            axes_background: Color::WHITE,

            grid_color: Color::rgb(0xE6, 0xE6, 0xE6),
            grid_width: 1.0,
            show_grid: true,

            spine_color: spine,
            spine_width: 1.0,
            show_top_spine: false,
            show_right_spine: false,
            show_bottom_spine: true,
            show_left_spine: true,

            tick_color: spine,
            tick_length: 4.0,
            tick_direction: TickDirection::Outward,
            tick_label_size: 9.0,

            axis_label_size: 11.0,
            title_size: 14.0,
            title_weight: FontWeight::Bold,
            text_color: spine,

            line_width: 1.5,
            marker_size: 6.0,
            marker_alpha: 0.8,

            color_cycle: TABLEAU_10.to_vec(),

            font_family: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Named themes
// ---------------------------------------------------------------------------

impl Theme {
    /// Dark theme with a near-black background and bright, neon-ish data colors.
    ///
    /// Suited for dashboards and presentations on dark backgrounds.
    pub fn dark() -> Self {
        let bg = Color::rgb(0x1C, 0x1C, 0x1C);
        let text = Color::rgb(0xE0, 0xE0, 0xE0);
        let grid = Color::rgb(0x3A, 0x3A, 0x3A);
        let spine = Color::rgb(0x55, 0x55, 0x55);

        // Bright / neon-ish palette optimised for dark backgrounds.
        let cycle = vec![
            Color::rgb(0x00, 0xD4, 0xFF), // cyan
            Color::rgb(0xFF, 0x6F, 0x61), // coral-red
            Color::rgb(0x7B, 0xED, 0x72), // lime-green
            Color::rgb(0xFF, 0xA6, 0x00), // amber
            Color::rgb(0xD1, 0x7D, 0xFF), // violet
            Color::rgb(0xFF, 0xE1, 0x00), // yellow
            Color::rgb(0x00, 0xFF, 0xAB), // mint
            Color::rgb(0xFF, 0x4D, 0xA6), // hot-pink
            Color::rgb(0x48, 0xBF, 0xE3), // sky-blue
            Color::rgb(0xE8, 0xE8, 0xE8), // light-grey
        ];

        Self {
            figure_background: bg,
            axes_background: bg,

            grid_color: grid,
            grid_width: 1.0,
            show_grid: true,

            spine_color: spine,
            spine_width: 1.0,
            show_top_spine: false,
            show_right_spine: false,
            show_bottom_spine: true,
            show_left_spine: true,

            tick_color: text,
            tick_length: 4.0,
            tick_direction: TickDirection::Outward,
            tick_label_size: 9.0,

            axis_label_size: 11.0,
            title_size: 14.0,
            title_weight: FontWeight::Bold,
            text_color: text,

            line_width: 1.5,
            marker_size: 6.0,
            marker_alpha: 0.9,

            color_cycle: cycle,

            font_family: None,
        }
    }

    /// Seaborn-inspired theme with a tinted axes background and white grid.
    ///
    /// Mimics the popular seaborn `"whitegrid"` aesthetic: a pale blue-grey
    /// axes face (`#EAEAF2`) with white grid lines over it.
    pub fn seaborn() -> Self {
        let text = Color::rgb(0x33, 0x33, 0x33);
        let axes_bg = Color::rgb(0xEA, 0xEA, 0xF2);

        Self {
            figure_background: Color::WHITE,
            axes_background: axes_bg,

            grid_color: Color::WHITE,
            grid_width: 1.0,
            show_grid: true,

            spine_color: Color::rgb(0xCC, 0xCC, 0xCC),
            spine_width: 1.0,
            show_top_spine: false,
            show_right_spine: false,
            show_bottom_spine: true,
            show_left_spine: true,

            tick_color: text,
            tick_length: 0.0, // seaborn typically hides tick marks
            tick_direction: TickDirection::Outward,
            tick_label_size: 9.0,

            axis_label_size: 11.0,
            title_size: 14.0,
            title_weight: FontWeight::Bold,
            text_color: text,

            line_width: 1.5,
            marker_size: 6.0,
            marker_alpha: 0.8,

            color_cycle: TABLEAU_10.to_vec(),

            font_family: None,
        }
    }

    /// ggplot2-inspired theme with a grey panel and white grid.
    ///
    /// Reproduces the characteristic look of R's ggplot2: a medium-grey panel
    /// (`#E5E5E5`), white major grid lines, all four spines hidden, and the
    /// ggplot2 default qualitative palette.
    pub fn ggplot() -> Self {
        let panel = Color::rgb(0xE5, 0xE5, 0xE5);
        let text = Color::rgb(0x30, 0x30, 0x30);

        // Classic ggplot2 qualitative palette (first 8 hues at C=100, L=65).
        let cycle = vec![
            Color::rgb(0xF8, 0x76, 0x6D), // red
            Color::rgb(0xA3, 0xA5, 0x00), // olive-yellow
            Color::rgb(0x00, 0xBA, 0x38), // green
            Color::rgb(0x00, 0xBF, 0xC4), // teal
            Color::rgb(0x61, 0x9C, 0xFF), // blue
            Color::rgb(0xF5, 0x64, 0xE3), // magenta
            Color::rgb(0xFF, 0x64, 0xB0), // pink
            Color::rgb(0xB7, 0x9F, 0x00), // gold
        ];

        Self {
            figure_background: Color::WHITE,
            axes_background: panel,

            grid_color: Color::WHITE,
            grid_width: 1.0,
            show_grid: true,

            // ggplot2 hides all spines by default.
            spine_color: Color::WHITE,
            spine_width: 0.0,
            show_top_spine: false,
            show_right_spine: false,
            show_bottom_spine: false,
            show_left_spine: false,

            tick_color: text,
            tick_length: 0.0, // no visible ticks in ggplot2 default
            tick_direction: TickDirection::Outward,
            tick_label_size: 9.0,

            axis_label_size: 11.0,
            title_size: 14.0,
            title_weight: FontWeight::Bold,
            text_color: text,

            line_width: 1.0,
            marker_size: 5.0,
            marker_alpha: 1.0,

            color_cycle: cycle,

            font_family: None,
        }
    }

    /// Publication-ready theme: crisp, minimal, and suitable for print.
    ///
    /// - Pure white background, no grid.
    /// - All four spines visible but thin (0.5 px) in near-black.
    /// - Inward ticks for a compact footprint.
    /// - Serif font family for journal aesthetics.
    pub fn publication() -> Self {
        let ink = Color::rgb(0x1A, 0x1A, 0x1A);

        Self {
            figure_background: Color::WHITE,
            axes_background: Color::WHITE,

            grid_color: Color::rgb(0xD0, 0xD0, 0xD0),
            grid_width: 0.5,
            show_grid: false,

            spine_color: ink,
            spine_width: 0.5,
            show_top_spine: true,
            show_right_spine: true,
            show_bottom_spine: true,
            show_left_spine: true,

            tick_color: ink,
            tick_length: 3.0,
            tick_direction: TickDirection::Inward,
            tick_label_size: 8.0,

            axis_label_size: 10.0,
            title_size: 12.0,
            title_weight: FontWeight::Bold,
            text_color: ink,

            line_width: 1.0,
            marker_size: 4.0,
            marker_alpha: 1.0,

            color_cycle: TABLEAU_10.to_vec(),

            font_family: Some("serif".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_background_is_white() {
        let t = Theme::default();
        assert_eq!(t.figure_background, Color::WHITE);
        assert_eq!(t.axes_background, Color::WHITE);
    }

    #[test]
    fn default_theme_despine_look() {
        let t = Theme::default();
        assert!(!t.show_top_spine);
        assert!(!t.show_right_spine);
        assert!(t.show_bottom_spine);
        assert!(t.show_left_spine);
    }

    #[test]
    fn default_theme_grid() {
        let t = Theme::default();
        assert_eq!(t.grid_color, Color::rgb(0xE6, 0xE6, 0xE6));
        assert!((t.grid_width - 1.0).abs() < f64::EPSILON);
        assert!(t.show_grid);
    }

    #[test]
    fn default_theme_spines() {
        let t = Theme::default();
        let expected = Color::rgb(0x33, 0x33, 0x33);
        assert_eq!(t.spine_color, expected);
        assert!((t.spine_width - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn default_theme_ticks() {
        let t = Theme::default();
        assert_eq!(t.tick_color, Color::rgb(0x33, 0x33, 0x33));
        assert!((t.tick_length - 4.0).abs() < f64::EPSILON);
        assert_eq!(t.tick_direction, TickDirection::Outward);
    }

    #[test]
    fn default_theme_font_sizes() {
        let t = Theme::default();
        assert!((t.tick_label_size - 9.0).abs() < f64::EPSILON);
        assert!((t.axis_label_size - 11.0).abs() < f64::EPSILON);
        assert!((t.title_size - 14.0).abs() < f64::EPSILON);
        assert_eq!(t.title_weight, FontWeight::Bold);
    }

    #[test]
    fn default_theme_text_color() {
        let t = Theme::default();
        assert_eq!(t.text_color, Color::rgb(0x33, 0x33, 0x33));
    }

    #[test]
    fn default_theme_data_defaults() {
        let t = Theme::default();
        assert!((t.line_width - 1.5).abs() < f64::EPSILON);
        assert!((t.marker_size - 6.0).abs() < f64::EPSILON);
        assert!((t.marker_alpha - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn default_theme_tableau_10_cycle() {
        let t = Theme::default();
        assert_eq!(t.color_cycle.len(), 10);
        assert_eq!(t.color_cycle[0], Color::TAB_BLUE);
        assert_eq!(t.color_cycle[9], Color::TAB_CYAN);
    }

    #[test]
    fn dark_theme_has_dark_background() {
        let t = Theme::dark();
        assert_eq!(t.figure_background, Color::rgb(0x1C, 0x1C, 0x1C));
        assert_eq!(t.axes_background, Color::rgb(0x1C, 0x1C, 0x1C));
    }

    #[test]
    fn dark_theme_light_text() {
        let t = Theme::dark();
        assert_eq!(t.text_color, Color::rgb(0xE0, 0xE0, 0xE0));
    }

    #[test]
    fn dark_theme_neon_cycle() {
        let t = Theme::dark();
        assert_eq!(t.color_cycle.len(), 10);
        // First color is a bright cyan.
        assert_eq!(t.color_cycle[0], Color::rgb(0x00, 0xD4, 0xFF));
    }

    #[test]
    fn seaborn_theme_tinted_face() {
        let t = Theme::seaborn();
        assert_eq!(t.axes_background, Color::rgb(0xEA, 0xEA, 0xF2));
    }

    #[test]
    fn seaborn_theme_white_grid() {
        let t = Theme::seaborn();
        assert_eq!(t.grid_color, Color::WHITE);
        assert!(t.show_grid);
    }

    #[test]
    fn ggplot_theme_grey_panel() {
        let t = Theme::ggplot();
        assert_eq!(t.axes_background, Color::rgb(0xE5, 0xE5, 0xE5));
    }

    #[test]
    fn ggplot_theme_white_grid() {
        let t = Theme::ggplot();
        assert_eq!(t.grid_color, Color::WHITE);
        assert!(t.show_grid);
    }

    #[test]
    fn ggplot_theme_no_spines() {
        let t = Theme::ggplot();
        assert!(!t.show_top_spine);
        assert!(!t.show_right_spine);
        assert!(!t.show_bottom_spine);
        assert!(!t.show_left_spine);
    }

    #[test]
    fn ggplot_theme_palette() {
        let t = Theme::ggplot();
        assert_eq!(t.color_cycle.len(), 8);
        assert_eq!(t.color_cycle[0], Color::rgb(0xF8, 0x76, 0x6D));
    }

    #[test]
    fn publication_theme_all_spines_visible() {
        let t = Theme::publication();
        assert!(t.show_top_spine);
        assert!(t.show_right_spine);
        assert!(t.show_bottom_spine);
        assert!(t.show_left_spine);
    }

    #[test]
    fn publication_theme_no_grid() {
        let t = Theme::publication();
        assert!(!t.show_grid);
    }

    #[test]
    fn publication_theme_thin_spines() {
        let t = Theme::publication();
        assert!((t.spine_width - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn publication_theme_inward_ticks() {
        let t = Theme::publication();
        assert_eq!(t.tick_direction, TickDirection::Inward);
    }

    #[test]
    fn publication_theme_serif_font() {
        let t = Theme::publication();
        assert_eq!(t.font_family, Some("serif".to_string()));
    }

    #[test]
    fn publication_theme_white_background() {
        let t = Theme::publication();
        assert_eq!(t.figure_background, Color::WHITE);
        assert_eq!(t.axes_background, Color::WHITE);
    }

    #[test]
    fn grid_axis_default_is_both() {
        assert_eq!(GridAxis::default(), GridAxis::Both);
    }
}
