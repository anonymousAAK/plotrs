//! Legend layout and rendering.
//!
//! This module handles the positioning, measurement, and drawing of legend
//! boxes within a plot area. A legend consists of one or more [`LegendEntry`]
//! items, each displaying a color swatch (line segment or filled rectangle)
//! alongside a text label.
//!
//! # Usage
//!
//! 1. Build a `Vec<LegendEntry>` from the artists on your axes (typically by
//!    iterating over labeled artists and calling [`LegendEntry::line`] or
//!    [`LegendEntry::filled`]).
//! 2. Call [`draw_legend`] with the renderer, entries, plot area, desired
//!    [`Loc`], and active [`Theme`].
//!
//! The legend is drawn *last* in the render pipeline so it appears on top of
//! all data elements.

use crate::primitives::{Affine, Color, Paint, Path, Point, Rect, Stroke, TextStyle};
use crate::renderer::Renderer;
use crate::theme::{Loc, Theme};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Padding between the legend box border and its contents (in pixels).
const PADDING: f64 = 8.0;

/// Width of a swatch (the colored indicator) in pixels.
const SWATCH_WIDTH: f64 = 22.0;

/// Half-height of a filled (box) swatch in pixels.
const SWATCH_HALF_HEIGHT: f64 = 4.0;

/// Horizontal gap between the swatch and the label text (in pixels).
const TEXT_GAP: f64 = 6.0;

/// Vertical spacing per legend row (in pixels).
const ROW_HEIGHT: f64 = 18.0;

/// Margin between the legend box and the plot area edge (in pixels).
const EDGE_MARGIN: f64 = 8.0;

/// Stroke width used to draw line swatches.
const LINE_SWATCH_STROKE: f64 = 2.0;

/// Stroke width of the legend box border.
const BORDER_STROKE: f64 = 0.5;

// ---------------------------------------------------------------------------
// LegendEntry
// ---------------------------------------------------------------------------

/// A single legend entry consisting of a color swatch and a text label.
///
/// Each entry describes how to render one row inside the legend box:
/// - **Line** entries draw a short horizontal line segment in the given color.
/// - **Filled** entries draw a small filled rectangle in the given color.
#[derive(Debug, Clone)]
pub struct LegendEntry {
    /// The display label shown next to the swatch.
    pub label: String,
    /// The color used to render the swatch.
    pub color: Color,
    /// The visual style of the swatch.
    pub swatch: SwatchKind,
}

/// Describes the visual representation of a legend swatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwatchKind {
    /// A short horizontal line segment, typically used for line plots.
    Line,
    /// A small filled rectangle, typically used for bar charts, histograms,
    /// and fill-between areas.
    Filled,
}

impl LegendEntry {
    /// Creates a legend entry with a line swatch.
    ///
    /// Use this for line plots and any artist that is primarily represented
    /// by a stroked path.
    pub fn line(label: impl Into<String>, color: Color) -> Self {
        Self {
            label: label.into(),
            color,
            swatch: SwatchKind::Line,
        }
    }

    /// Creates a legend entry with a filled-rectangle swatch.
    ///
    /// Use this for bar charts, histograms, fill-between areas, and any
    /// artist that is primarily represented by a filled region.
    pub fn filled(label: impl Into<String>, color: Color) -> Self {
        Self {
            label: label.into(),
            color,
            swatch: SwatchKind::Filled,
        }
    }
}

// ---------------------------------------------------------------------------
// Legend measurement
// ---------------------------------------------------------------------------

/// Measures the bounding box dimensions required by the legend.
///
/// Returns `(width, height)` in pixels. If `entries` is empty the returned
/// size is `(0.0, 0.0)`.
fn measure_legend(
    renderer: &impl Renderer,
    entries: &[LegendEntry],
    text_style: &TextStyle,
) -> (f64, f64) {
    if entries.is_empty() {
        return (0.0, 0.0);
    }

    let max_label_width: f64 = entries
        .iter()
        .map(|e| {
            let (w, _) = renderer.measure_text(&e.label, text_style);
            w
        })
        .fold(0.0_f64, f64::max);

    let width = PADDING * 2.0 + SWATCH_WIDTH + TEXT_GAP + max_label_width;
    let height = PADDING * 2.0 + entries.len() as f64 * ROW_HEIGHT;

    (width, height)
}

// ---------------------------------------------------------------------------
// Legend positioning
// ---------------------------------------------------------------------------

/// Computes the top-left corner `(x, y)` of the legend box for the given
/// [`Loc`] variant within `plot_area`.
///
/// All eleven `Loc` variants are handled explicitly. The box is inset from
/// the plot area edges by [`EDGE_MARGIN`] pixels.
fn position_legend(loc: Loc, plot_area: &Rect, box_width: f64, box_height: f64) -> (f64, f64) {
    let left = plot_area.x + EDGE_MARGIN;
    let right = plot_area.right() - box_width - EDGE_MARGIN;
    let top = plot_area.y + EDGE_MARGIN;
    let bottom = plot_area.bottom() - box_height - EDGE_MARGIN;
    let center_x = plot_area.x + (plot_area.width - box_width) / 2.0;
    let center_y = plot_area.y + (plot_area.height - box_height) / 2.0;

    match loc {
        // Corner positions
        Loc::UpperRight => (right, top),
        Loc::UpperLeft => (left, top),
        Loc::LowerLeft => (left, bottom),
        Loc::LowerRight => (right, bottom),

        // Edge-centered positions
        Loc::Right | Loc::CenterRight => (right, center_y),
        Loc::CenterLeft => (left, center_y),
        Loc::UpperCenter => (center_x, top),
        Loc::LowerCenter => (center_x, bottom),

        // Dead center
        Loc::Center => (center_x, center_y),

        // Best: fall back to upper-right for now. A future implementation
        // will score candidate positions by data-point occlusion and pick
        // the one with the least overlap (see feature spec for details).
        Loc::Best => (right, top),

        // Exhaustive today, but `Loc` is `#[non_exhaustive]` so we add a
        // catch-all that defaults to upper-right to stay forward-compatible.
        #[allow(unreachable_patterns)]
        _ => (right, top),
    }
}

// ---------------------------------------------------------------------------
// Legend drawing
// ---------------------------------------------------------------------------

/// Renders a legend box at the specified location within the plot area.
///
/// The legend is drawn with a semi-transparent white background and a thin
/// border so that it does not fully obscure data underneath. Text styling
/// is derived from the active [`Theme`] (specifically `tick_label_size` and
/// `text_color`).
///
/// If `entries` is empty this function returns immediately without drawing
/// anything.
///
/// # Arguments
///
/// * `renderer`  - The rendering backend to draw into.
/// * `entries`   - The legend entries to display (swatch + label each).
/// * `plot_area` - The rectangle describing the axes / data area.
/// * `loc`       - Where to place the legend relative to the plot area.
/// * `theme`     - The active theme (used for font size, text color).
///
/// # Example
///
/// ```ignore
/// use plotkit_core::legend::{LegendEntry, draw_legend};
/// use plotkit_core::primitives::{Color, Rect};
/// use plotkit_core::theme::{Loc, Theme};
///
/// let entries = vec![
///     LegendEntry::line("Temperature", Color::TAB_BLUE),
///     LegendEntry::filled("Rainfall", Color::TAB_GREEN),
/// ];
/// let area = Rect::new(60.0, 40.0, 680.0, 520.0);
/// draw_legend(&mut renderer, &entries, &area, Loc::UpperRight, &Theme::default());
/// ```
pub fn draw_legend(
    renderer: &mut impl Renderer,
    entries: &[LegendEntry],
    plot_area: &Rect,
    loc: Loc,
    theme: &Theme,
) {
    if entries.is_empty() {
        return;
    }

    // Build the text style from the theme.
    let mut text_style = TextStyle::new(theme.tick_label_size);
    text_style.color = theme.text_color;
    if let Some(ref family) = theme.font_family {
        text_style.family = Some(family.clone());
    }

    // Measure and position.
    let (box_width, box_height) = measure_legend(renderer, entries, &text_style);
    let (bx, by) = position_legend(loc, plot_area, box_width, box_height);
    let legend_rect = Rect::new(bx, by, box_width, box_height);

    // --- Background fill ---
    let bg_path = Path::rect(legend_rect);
    let bg_paint = Paint::new(Color::new(255, 255, 255, 230));
    renderer.fill_path(&bg_path, &bg_paint, Affine::IDENTITY);

    // --- Border stroke ---
    let border_paint = Paint::new(Color::rgb(200, 200, 200));
    let border_stroke = Stroke::new(BORDER_STROKE);
    renderer.stroke_path(&bg_path, &border_paint, &border_stroke, Affine::IDENTITY);

    // --- Draw each entry ---
    for (i, entry) in entries.iter().enumerate() {
        let row_center_y = by + PADDING + i as f64 * ROW_HEIGHT + ROW_HEIGHT / 2.0;
        let swatch_x = bx + PADDING;

        draw_swatch(renderer, entry, swatch_x, row_center_y);
        draw_label(renderer, entry, swatch_x, row_center_y, &text_style);
    }
}

/// Draws the swatch (line or filled rectangle) for a single legend entry.
fn draw_swatch(renderer: &mut impl Renderer, entry: &LegendEntry, x: f64, center_y: f64) {
    let paint = Paint::new(entry.color);

    match entry.swatch {
        SwatchKind::Line => {
            let mut line = Path::new();
            line.move_to(x, center_y);
            line.line_to(x + SWATCH_WIDTH, center_y);
            let stroke = Stroke::new(LINE_SWATCH_STROKE);
            renderer.stroke_path(&line, &paint, &stroke, Affine::IDENTITY);
        }
        SwatchKind::Filled => {
            let rect = Rect::new(
                x,
                center_y - SWATCH_HALF_HEIGHT,
                SWATCH_WIDTH,
                SWATCH_HALF_HEIGHT * 2.0,
            );
            let path = Path::rect(rect);
            renderer.fill_path(&path, &paint, Affine::IDENTITY);
        }
    }
}

/// Draws the text label for a single legend entry, positioned to the right
/// of the swatch.
fn draw_label(
    renderer: &mut impl Renderer,
    entry: &LegendEntry,
    swatch_x: f64,
    center_y: f64,
    text_style: &TextStyle,
) {
    let text_x = swatch_x + SWATCH_WIDTH + TEXT_GAP;
    // Offset the text slightly below the row center to approximate baseline
    // alignment with the swatch.
    let text_y = center_y + text_style.size * 0.35;
    renderer.draw_text(
        &entry.label,
        Point::new(text_x, text_y),
        text_style,
        Affine::IDENTITY,
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Image;

    /// A minimal stub renderer that records calls for assertion.
    struct StubRenderer {
        width: u32,
        height: u32,
        fill_count: usize,
        stroke_count: usize,
        text_count: usize,
        texts: Vec<String>,
    }

    impl StubRenderer {
        fn new(w: u32, h: u32) -> Self {
            Self {
                width: w,
                height: h,
                fill_count: 0,
                stroke_count: 0,
                text_count: 0,
                texts: Vec::new(),
            }
        }
    }

    impl Renderer for StubRenderer {
        fn size(&self) -> (u32, u32) {
            (self.width, self.height)
        }

        fn fill_path(&mut self, _path: &Path, _paint: &Paint, _transform: Affine) {
            self.fill_count += 1;
        }

        fn stroke_path(
            &mut self,
            _path: &Path,
            _paint: &Paint,
            _stroke: &Stroke,
            _transform: Affine,
        ) {
            self.stroke_count += 1;
        }

        fn draw_text(&mut self, text: &str, _pos: Point, _style: &TextStyle, _transform: Affine) {
            self.text_count += 1;
            self.texts.push(text.to_string());
        }

        fn draw_image(&mut self, _img: &Image, _dst: Rect, _transform: Affine) {}

        fn push_clip(&mut self, _path: &Path, _transform: Affine) {}

        fn pop_clip(&mut self) {}

        fn measure_text(&self, text: &str, style: &TextStyle) -> (f64, f64) {
            // Approximate: 0.6 * font_size per character.
            let w = text.len() as f64 * style.size * 0.6;
            let h = style.size;
            (w, h)
        }

        fn finalize(self) -> Vec<u8> {
            Vec::new()
        }
    }

    // -----------------------------------------------------------------------
    // LegendEntry construction
    // -----------------------------------------------------------------------

    #[test]
    fn entry_line_constructor() {
        let e = LegendEntry::line("Temperature", Color::TAB_BLUE);
        assert_eq!(e.label, "Temperature");
        assert_eq!(e.color, Color::TAB_BLUE);
        assert_eq!(e.swatch, SwatchKind::Line);
    }

    #[test]
    fn entry_filled_constructor() {
        let e = LegendEntry::filled("Rainfall", Color::TAB_GREEN);
        assert_eq!(e.label, "Rainfall");
        assert_eq!(e.color, Color::TAB_GREEN);
        assert_eq!(e.swatch, SwatchKind::Filled);
    }

    #[test]
    fn entry_accepts_string_type() {
        let owned = String::from("Owned label");
        let e = LegendEntry::line(owned, Color::TAB_RED);
        assert_eq!(e.label, "Owned label");
    }

    // -----------------------------------------------------------------------
    // Measurement
    // -----------------------------------------------------------------------

    #[test]
    fn measure_empty_returns_zero() {
        let r = StubRenderer::new(800, 600);
        let style = TextStyle::new(9.0);
        let (w, h) = measure_legend(&r, &[], &style);
        assert_eq!(w, 0.0);
        assert_eq!(h, 0.0);
    }

    #[test]
    fn measure_single_entry() {
        let r = StubRenderer::new(800, 600);
        let style = TextStyle::new(9.0);
        let entries = vec![LegendEntry::line("sin(x)", Color::TAB_BLUE)];
        let (w, h) = measure_legend(&r, &entries, &style);

        // Expected width: padding*2 + swatch_width + text_gap + label_width
        let label_w = 6.0 * 9.0 * 0.6; // "sin(x)" is 6 chars
        let expected_w = PADDING * 2.0 + SWATCH_WIDTH + TEXT_GAP + label_w;
        assert!((w - expected_w).abs() < 1e-9);

        // Expected height: padding*2 + 1 * row_height
        let expected_h = PADDING * 2.0 + ROW_HEIGHT;
        assert!((h - expected_h).abs() < 1e-9);
    }

    #[test]
    fn measure_uses_longest_label() {
        let r = StubRenderer::new(800, 600);
        let style = TextStyle::new(9.0);
        let entries = vec![
            LegendEntry::line("A", Color::TAB_BLUE),
            LegendEntry::filled("Much longer label", Color::TAB_RED),
        ];
        let (w, _) = measure_legend(&r, &entries, &style);

        let long_label_w = 17.0 * 9.0 * 0.6; // "Much longer label" is 17 chars
        let expected_w = PADDING * 2.0 + SWATCH_WIDTH + TEXT_GAP + long_label_w;
        assert!((w - expected_w).abs() < 1e-9);
    }

    // -----------------------------------------------------------------------
    // Positioning
    // -----------------------------------------------------------------------

    #[test]
    fn position_upper_right() {
        let area = Rect::new(50.0, 30.0, 700.0, 500.0);
        let (x, y) = position_legend(Loc::UpperRight, &area, 100.0, 60.0);
        assert!((x - (area.right() - 100.0 - EDGE_MARGIN)).abs() < 1e-9);
        assert!((y - (area.y + EDGE_MARGIN)).abs() < 1e-9);
    }

    #[test]
    fn position_upper_left() {
        let area = Rect::new(50.0, 30.0, 700.0, 500.0);
        let (x, y) = position_legend(Loc::UpperLeft, &area, 100.0, 60.0);
        assert!((x - (area.x + EDGE_MARGIN)).abs() < 1e-9);
        assert!((y - (area.y + EDGE_MARGIN)).abs() < 1e-9);
    }

    #[test]
    fn position_lower_left() {
        let area = Rect::new(50.0, 30.0, 700.0, 500.0);
        let (x, y) = position_legend(Loc::LowerLeft, &area, 100.0, 60.0);
        assert!((x - (area.x + EDGE_MARGIN)).abs() < 1e-9);
        assert!((y - (area.bottom() - 60.0 - EDGE_MARGIN)).abs() < 1e-9);
    }

    #[test]
    fn position_lower_right() {
        let area = Rect::new(50.0, 30.0, 700.0, 500.0);
        let (x, y) = position_legend(Loc::LowerRight, &area, 100.0, 60.0);
        assert!((x - (area.right() - 100.0 - EDGE_MARGIN)).abs() < 1e-9);
        assert!((y - (area.bottom() - 60.0 - EDGE_MARGIN)).abs() < 1e-9);
    }

    #[test]
    fn position_center() {
        let area = Rect::new(0.0, 0.0, 800.0, 600.0);
        let (x, y) = position_legend(Loc::Center, &area, 100.0, 60.0);
        assert!((x - 350.0).abs() < 1e-9);
        assert!((y - 270.0).abs() < 1e-9);
    }

    #[test]
    fn position_center_left() {
        let area = Rect::new(50.0, 30.0, 700.0, 500.0);
        let (x, y) = position_legend(Loc::CenterLeft, &area, 100.0, 60.0);
        assert!((x - (area.x + EDGE_MARGIN)).abs() < 1e-9);
        let expected_y = area.y + (area.height - 60.0) / 2.0;
        assert!((y - expected_y).abs() < 1e-9);
    }

    #[test]
    fn position_center_right() {
        let area = Rect::new(50.0, 30.0, 700.0, 500.0);
        let (x, y) = position_legend(Loc::CenterRight, &area, 100.0, 60.0);
        assert!((x - (area.right() - 100.0 - EDGE_MARGIN)).abs() < 1e-9);
        let expected_y = area.y + (area.height - 60.0) / 2.0;
        assert!((y - expected_y).abs() < 1e-9);
    }

    #[test]
    fn position_right_aliases_center_right() {
        let area = Rect::new(50.0, 30.0, 700.0, 500.0);
        let right = position_legend(Loc::Right, &area, 100.0, 60.0);
        let center_right = position_legend(Loc::CenterRight, &area, 100.0, 60.0);
        assert_eq!(right, center_right);
    }

    #[test]
    fn position_upper_center() {
        let area = Rect::new(0.0, 0.0, 800.0, 600.0);
        let (x, y) = position_legend(Loc::UpperCenter, &area, 100.0, 60.0);
        assert!((x - 350.0).abs() < 1e-9);
        assert!((y - EDGE_MARGIN).abs() < 1e-9);
    }

    #[test]
    fn position_lower_center() {
        let area = Rect::new(0.0, 0.0, 800.0, 600.0);
        let (x, y) = position_legend(Loc::LowerCenter, &area, 100.0, 60.0);
        assert!((x - 350.0).abs() < 1e-9);
        assert!((y - (600.0 - 60.0 - EDGE_MARGIN)).abs() < 1e-9);
    }

    #[test]
    fn position_best_defaults_to_upper_right() {
        let area = Rect::new(50.0, 30.0, 700.0, 500.0);
        let best = position_legend(Loc::Best, &area, 100.0, 60.0);
        let upper_right = position_legend(Loc::UpperRight, &area, 100.0, 60.0);
        assert_eq!(best, upper_right);
    }

    // -----------------------------------------------------------------------
    // Full draw_legend integration
    // -----------------------------------------------------------------------

    #[test]
    fn draw_legend_empty_is_noop() {
        let mut r = StubRenderer::new(800, 600);
        let area = Rect::new(0.0, 0.0, 800.0, 600.0);
        draw_legend(&mut r, &[], &area, Loc::UpperRight, &Theme::default());
        assert_eq!(r.fill_count, 0);
        assert_eq!(r.stroke_count, 0);
        assert_eq!(r.text_count, 0);
    }

    #[test]
    fn draw_legend_single_line_entry() {
        let mut r = StubRenderer::new(800, 600);
        let area = Rect::new(0.0, 0.0, 800.0, 600.0);
        let entries = vec![LegendEntry::line("Series A", Color::TAB_BLUE)];
        draw_legend(&mut r, &entries, &area, Loc::UpperRight, &Theme::default());

        // Background fill (1) + no swatch fill for line entries = 1 fill total
        assert_eq!(r.fill_count, 1);
        // Border stroke (1) + line swatch (1) = 2 strokes
        assert_eq!(r.stroke_count, 2);
        // One text label
        assert_eq!(r.text_count, 1);
        assert_eq!(r.texts[0], "Series A");
    }

    #[test]
    fn draw_legend_single_filled_entry() {
        let mut r = StubRenderer::new(800, 600);
        let area = Rect::new(0.0, 0.0, 800.0, 600.0);
        let entries = vec![LegendEntry::filled("Bars", Color::TAB_ORANGE)];
        draw_legend(&mut r, &entries, &area, Loc::LowerLeft, &Theme::default());

        // Background fill (1) + swatch fill (1) = 2 fills
        assert_eq!(r.fill_count, 2);
        // Border stroke (1) only, no line swatch
        assert_eq!(r.stroke_count, 1);
        assert_eq!(r.text_count, 1);
        assert_eq!(r.texts[0], "Bars");
    }

    #[test]
    fn draw_legend_multiple_entries() {
        let mut r = StubRenderer::new(800, 600);
        let area = Rect::new(0.0, 0.0, 800.0, 600.0);
        let entries = vec![
            LegendEntry::line("Alpha", Color::TAB_BLUE),
            LegendEntry::filled("Beta", Color::TAB_GREEN),
            LegendEntry::line("Gamma", Color::TAB_RED),
        ];
        draw_legend(&mut r, &entries, &area, Loc::Center, &Theme::default());

        // Background fill (1) + 1 filled swatch = 2 fills
        assert_eq!(r.fill_count, 2);
        // Border stroke (1) + 2 line swatches = 3 strokes
        assert_eq!(r.stroke_count, 3);
        // 3 text labels
        assert_eq!(r.text_count, 3);
        assert_eq!(r.texts, vec!["Alpha", "Beta", "Gamma"]);
    }

    #[test]
    fn draw_legend_respects_theme_font_family() {
        // Ensure we do not panic when the theme has a font family set.
        let mut r = StubRenderer::new(800, 600);
        let area = Rect::new(0.0, 0.0, 800.0, 600.0);
        let entries = vec![LegendEntry::line("Test", Color::TAB_BLUE)];
        let theme = Theme::publication(); // has font_family = Some("serif")
        draw_legend(&mut r, &entries, &area, Loc::UpperLeft, &theme);
        assert_eq!(r.text_count, 1);
    }
}
