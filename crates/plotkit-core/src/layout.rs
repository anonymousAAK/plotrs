//! Layout engine for computing plot area and margins.
//!
//! This module determines where the plot area, title, axis labels, tick labels,
//! and legend are positioned within the figure dimensions. It implements a
//! mini tight-layout algorithm to prevent clipping and produce well-spaced plots.

use crate::primitives::Rect;

// ---------------------------------------------------------------------------
// Margins
// ---------------------------------------------------------------------------

/// Margins in pixels around the plot area.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Margins {
    /// Space above the plot area.
    pub top: f64,
    /// Space to the right of the plot area.
    pub right: f64,
    /// Space below the plot area.
    pub bottom: f64,
    /// Space to the left of the plot area.
    pub left: f64,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
}

impl Margins {
    /// Creates margins with the same value on all four sides.
    pub fn uniform(value: f64) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Creates margins with specified values for each side.
    pub fn new(top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self { top, right, bottom, left }
    }

    /// Returns the total horizontal margin (left + right).
    pub fn horizontal(&self) -> f64 {
        self.left + self.right
    }

    /// Returns the total vertical margin (top + bottom).
    pub fn vertical(&self) -> f64 {
        self.top + self.bottom
    }
}

// ---------------------------------------------------------------------------
// LayoutResult
// ---------------------------------------------------------------------------

/// Result of the layout computation.
///
/// All rectangles use figure-pixel coordinates with the origin at the top-left
/// corner and y increasing downward.
#[derive(Debug, Clone)]
pub struct LayoutResult {
    /// The plot/data area rectangle in figure-pixel coordinates.
    pub plot_area: Rect,
    /// Space reserved at top for the title, if present.
    pub title_area: Option<Rect>,
    /// Space reserved at bottom for the x-axis label, if present.
    pub xlabel_area: Option<Rect>,
    /// Space reserved at left for the y-axis label, if present.
    pub ylabel_area: Option<Rect>,
    /// Space reserved at right for the legend, if present.
    pub legend_area: Option<Rect>,
    /// Accumulated margins consumed by tick labels on each side.
    pub tick_label_margins: Margins,
}

// ---------------------------------------------------------------------------
// LayoutConfig
// ---------------------------------------------------------------------------

/// Configuration for layout computation.
///
/// Callers set the figure dimensions and declare which decorations are present;
/// the layout engine then carves out non-overlapping rectangles for each element.
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Total figure width in pixels.
    pub figure_width: f64,
    /// Total figure height in pixels.
    pub figure_height: f64,
    /// Whether a title is displayed above the plot.
    pub has_title: bool,
    /// Whether an x-axis label is displayed below the plot.
    pub has_xlabel: bool,
    /// Whether a y-axis label is displayed to the left of the plot.
    pub has_ylabel: bool,
    /// Whether a legend box is displayed to the right of the plot.
    pub has_legend: bool,
    /// Height of the title text in pixels.
    pub title_height: f64,
    /// Height of the x-axis label text in pixels.
    pub xlabel_height: f64,
    /// Width of the y-axis label text in pixels (measured along the rotated axis).
    pub ylabel_width: f64,
    /// Maximum width of any y-axis tick label in pixels.
    pub tick_label_max_width: f64,
    /// Height of a single tick label line in pixels.
    pub tick_label_height: f64,
    /// Width allocated for the legend box in pixels.
    pub legend_width: f64,
    /// General padding between layout elements in pixels.
    pub padding: f64,
    /// Minimum plot area width; layout will not shrink below this.
    pub min_plot_width: f64,
    /// Minimum plot area height; layout will not shrink below this.
    pub min_plot_height: f64,
}

impl LayoutConfig {
    /// Creates a configuration with sensible defaults for the given figure size.
    ///
    /// All decoration flags default to `false`; callers should enable the ones
    /// they need before passing the config to [`compute_layout`].
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            figure_width: width,
            figure_height: height,
            has_title: false,
            has_xlabel: false,
            has_ylabel: false,
            has_legend: false,
            title_height: 20.0,
            xlabel_height: 16.0,
            ylabel_width: 16.0,
            tick_label_max_width: 40.0,
            tick_label_height: 12.0,
            legend_width: 80.0,
            padding: 10.0,
            min_plot_width: 60.0,
            min_plot_height: 40.0,
        }
    }
}

// ---------------------------------------------------------------------------
// compute_layout
// ---------------------------------------------------------------------------

/// Computes the layout for a single axes within a figure.
///
/// The algorithm works inward from the figure edges in the following order:
///
/// 1. Outer padding on all four sides.
/// 2. **Top:** title (if present), then a gap.
/// 3. **Bottom:** x-axis label (if present), then x-axis tick labels, then a gap.
/// 4. **Left:** y-axis label (if present), then y-axis tick labels, then a gap.
/// 5. **Right:** legend (if present), then a gap.
///
/// Whatever remains in the center becomes the `plot_area`. If the remaining
/// space is smaller than the configured minimums the plot area is clamped so
/// that content is never collapsed to zero.
pub fn compute_layout(config: &LayoutConfig) -> LayoutResult {
    let pad = config.padding;

    // Start with the full figure area, then shrink inward.
    let mut top = pad;
    let mut bottom = config.figure_height - pad;
    let mut left = pad;
    let mut right = config.figure_width - pad;

    // -- Title (top) --------------------------------------------------------
    let title_area = if config.has_title {
        let area = Rect::new(left, top, right - left, config.title_height);
        top += config.title_height + pad;
        Some(area)
    } else {
        None
    };

    // -- X-axis label (bottom) ----------------------------------------------
    let xlabel_area = if config.has_xlabel {
        bottom -= config.xlabel_height;
        let area = Rect::new(left, bottom, right - left, config.xlabel_height);
        bottom -= pad;
        Some(area)
    } else {
        None
    };

    // -- X-axis tick labels (bottom) ----------------------------------------
    let tick_bottom = config.tick_label_height + pad;
    bottom -= tick_bottom;

    // -- Y-axis label (left) ------------------------------------------------
    let ylabel_area = if config.has_ylabel {
        let area = Rect::new(left, top, config.ylabel_width, bottom - top);
        left += config.ylabel_width + pad;
        Some(area)
    } else {
        None
    };

    // -- Y-axis tick labels (left) ------------------------------------------
    let tick_left = config.tick_label_max_width + pad;
    left += tick_left;

    // -- Legend (right) -----------------------------------------------------
    let legend_area = if config.has_legend {
        right -= config.legend_width;
        let area = Rect::new(right, top, config.legend_width, bottom - top);
        right -= pad;
        Some(area)
    } else {
        None
    };

    // -- Small padding for right-side tick overhang -------------------------
    // Even without a legend, the rightmost tick label can overhang slightly.
    let tick_right_overhang = config.tick_label_max_width * 0.5;
    right -= tick_right_overhang;

    // -- Small padding for top tick overhang --------------------------------
    let tick_top_overhang = config.tick_label_height * 0.5;
    top += tick_top_overhang;

    // -- Clamp to minimum sizes ---------------------------------------------
    let plot_width = (right - left).max(config.min_plot_width);
    let plot_height = (bottom - top).max(config.min_plot_height);

    // If we had to expand to meet minimums, center the expanded area in the
    // available space.
    let actual_width = right - left;
    let actual_height = bottom - top;

    let plot_x = if plot_width > actual_width {
        left - (plot_width - actual_width) / 2.0
    } else {
        left
    };
    let plot_y = if plot_height > actual_height {
        top - (plot_height - actual_height) / 2.0
    } else {
        top
    };

    let plot_area = Rect::new(plot_x, plot_y, plot_width, plot_height);

    let tick_label_margins = Margins {
        top: tick_top_overhang,
        right: tick_right_overhang,
        bottom: tick_bottom,
        left: tick_left,
    };

    LayoutResult {
        plot_area,
        title_area,
        xlabel_area,
        ylabel_area,
        legend_area,
        tick_label_margins,
    }
}

// ---------------------------------------------------------------------------
// compute_subplot_rects
// ---------------------------------------------------------------------------

/// Computes subplot positions for a grid of axes.
///
/// Returns a `Vec<Rect>` with one entry per subplot cell in **row-major order**
/// (left to right, top to bottom). Each rectangle represents the total
/// available area for that subplot — callers should run [`compute_layout`] on
/// each rect individually to determine the inner plot area and decoration
/// positions.
///
/// # Arguments
///
/// * `figure_width`  — Total figure width in pixels.
/// * `figure_height` — Total figure height in pixels.
/// * `nrows`         — Number of rows in the subplot grid.
/// * `ncols`         — Number of columns in the subplot grid.
/// * `spacing`       — Gap between adjacent subplots in pixels.
/// * `outer_padding` — Padding between the figure edges and the outermost subplots.
///
/// # Panics
///
/// Panics if `nrows` or `ncols` is zero.
pub fn compute_subplot_rects(
    figure_width: f64,
    figure_height: f64,
    nrows: usize,
    ncols: usize,
    spacing: f64,
    outer_padding: f64,
) -> Vec<Rect> {
    assert!(nrows > 0, "nrows must be at least 1");
    assert!(ncols > 0, "ncols must be at least 1");

    // Total space consumed by inter-cell gaps.
    let total_h_spacing = spacing * (ncols as f64 - 1.0);
    let total_v_spacing = spacing * (nrows as f64 - 1.0);

    // Space available for all cells after removing outer padding and gaps.
    let avail_width = (figure_width - 2.0 * outer_padding - total_h_spacing).max(0.0);
    let avail_height = (figure_height - 2.0 * outer_padding - total_v_spacing).max(0.0);

    let cell_width = avail_width / ncols as f64;
    let cell_height = avail_height / nrows as f64;

    let mut rects = Vec::with_capacity(nrows * ncols);

    for row in 0..nrows {
        for col in 0..ncols {
            let x = outer_padding + col as f64 * (cell_width + spacing);
            let y = outer_padding + row as f64 * (cell_height + spacing);
            rects.push(Rect::new(x, y, cell_width, cell_height));
        }
    }

    rects
}

// ---------------------------------------------------------------------------
// compute_layout_in_rect
// ---------------------------------------------------------------------------

/// Convenience wrapper that runs [`compute_layout`] within a specific
/// rectangle (e.g., one cell of a subplot grid).
///
/// The returned [`LayoutResult`] uses the **same coordinate system** as the
/// input `cell` — that is, all positions are in figure-pixel coordinates, not
/// relative to the cell origin.
pub fn compute_layout_in_rect(cell: &Rect, config: &LayoutConfig) -> LayoutResult {
    let mut local_config = config.clone();
    local_config.figure_width = cell.width;
    local_config.figure_height = cell.height;

    let mut result = compute_layout(&local_config);

    // Translate everything from local (0,0) to the cell's position.
    translate_rect(&mut result.plot_area, cell.x, cell.y);

    if let Some(ref mut r) = result.title_area {
        translate_rect(r, cell.x, cell.y);
    }
    if let Some(ref mut r) = result.xlabel_area {
        translate_rect(r, cell.x, cell.y);
    }
    if let Some(ref mut r) = result.ylabel_area {
        translate_rect(r, cell.x, cell.y);
    }
    if let Some(ref mut r) = result.legend_area {
        translate_rect(r, cell.x, cell.y);
    }

    result
}

/// Shifts a rectangle by `(dx, dy)`.
fn translate_rect(rect: &mut Rect, dx: f64, dy: f64) {
    rect.x += dx;
    rect.y += dy;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A small helper that asserts a rect has positive dimensions.
    fn assert_positive_rect(r: &Rect, label: &str) {
        assert!(
            r.width > 0.0 && r.height > 0.0,
            "{label}: expected positive dimensions, got {w}x{h}",
            w = r.width,
            h = r.height,
        );
    }

    #[test]
    fn basic_layout_no_decorations() {
        let config = LayoutConfig::new(800.0, 600.0);
        let result = compute_layout(&config);

        assert_positive_rect(&result.plot_area, "plot_area");
        assert!(result.title_area.is_none());
        assert!(result.xlabel_area.is_none());
        assert!(result.ylabel_area.is_none());
        assert!(result.legend_area.is_none());
    }

    #[test]
    fn layout_with_all_decorations() {
        let mut config = LayoutConfig::new(800.0, 600.0);
        config.has_title = true;
        config.has_xlabel = true;
        config.has_ylabel = true;
        config.has_legend = true;

        let result = compute_layout(&config);

        assert_positive_rect(&result.plot_area, "plot_area");

        let title = result.title_area.as_ref().unwrap();
        let xlabel = result.xlabel_area.as_ref().unwrap();
        let ylabel = result.ylabel_area.as_ref().unwrap();
        let legend = result.legend_area.as_ref().unwrap();

        assert_positive_rect(title, "title");
        assert_positive_rect(xlabel, "xlabel");
        assert_positive_rect(ylabel, "ylabel");
        assert_positive_rect(legend, "legend");

        // Title should be above the plot area.
        assert!(
            title.bottom() <= result.plot_area.y,
            "title bottom ({}) should be <= plot_area top ({})",
            title.bottom(),
            result.plot_area.y,
        );

        // X-axis label should be below the plot area.
        assert!(
            xlabel.y >= result.plot_area.bottom(),
            "xlabel top ({}) should be >= plot_area bottom ({})",
            xlabel.y,
            result.plot_area.bottom(),
        );

        // Y-axis label should be to the left of the plot area.
        assert!(
            ylabel.right() <= result.plot_area.x,
            "ylabel right ({}) should be <= plot_area left ({})",
            ylabel.right(),
            result.plot_area.x,
        );
    }

    #[test]
    fn plot_area_stays_within_figure() {
        let mut config = LayoutConfig::new(800.0, 600.0);
        config.has_title = true;
        config.has_xlabel = true;
        config.has_ylabel = true;
        config.has_legend = true;

        let result = compute_layout(&config);
        let pa = &result.plot_area;

        assert!(pa.x >= 0.0, "plot_area left edge is negative");
        assert!(pa.y >= 0.0, "plot_area top edge is negative");
        assert!(
            pa.right() <= config.figure_width,
            "plot_area right ({}) exceeds figure width ({})",
            pa.right(),
            config.figure_width,
        );
        assert!(
            pa.bottom() <= config.figure_height,
            "plot_area bottom ({}) exceeds figure height ({})",
            pa.bottom(),
            config.figure_height,
        );
    }

    #[test]
    fn small_figure_respects_minimums() {
        let mut config = LayoutConfig::new(120.0, 100.0);
        config.has_title = true;
        config.has_xlabel = true;
        config.has_ylabel = true;

        let result = compute_layout(&config);
        let pa = &result.plot_area;

        assert!(
            pa.width >= config.min_plot_width,
            "plot_area width ({}) < min ({})",
            pa.width,
            config.min_plot_width,
        );
        assert!(
            pa.height >= config.min_plot_height,
            "plot_area height ({}) < min ({})",
            pa.height,
            config.min_plot_height,
        );
    }

    #[test]
    fn subplot_grid_basic() {
        let rects = compute_subplot_rects(800.0, 600.0, 2, 3, 10.0, 20.0);
        assert_eq!(rects.len(), 6);

        // All rects should have positive dimensions.
        for (i, r) in rects.iter().enumerate() {
            assert_positive_rect(r, &format!("subplot[{i}]"));
        }

        // First cell starts at the outer padding.
        assert!((rects[0].x - 20.0).abs() < 1e-9);
        assert!((rects[0].y - 20.0).abs() < 1e-9);

        // Cells in the same row should have the same y and height.
        assert!((rects[0].y - rects[1].y).abs() < 1e-9);
        assert!((rects[0].height - rects[1].height).abs() < 1e-9);

        // Cells in the same column should have the same x and width.
        assert!((rects[0].x - rects[3].x).abs() < 1e-9);
        assert!((rects[0].width - rects[3].width).abs() < 1e-9);
    }

    #[test]
    fn subplot_single_cell() {
        let rects = compute_subplot_rects(800.0, 600.0, 1, 1, 10.0, 20.0);
        assert_eq!(rects.len(), 1);

        let r = &rects[0];
        assert!((r.x - 20.0).abs() < 1e-9);
        assert!((r.y - 20.0).abs() < 1e-9);
        assert!((r.width - 760.0).abs() < 1e-9);
        assert!((r.height - 560.0).abs() < 1e-9);
    }

    #[test]
    fn subplot_cells_cover_figure() {
        let rects = compute_subplot_rects(800.0, 600.0, 2, 2, 10.0, 15.0);

        // Last cell's right/bottom edge should align with figure_width/height
        // minus outer_padding.
        let last = &rects[3];
        assert!(
            (last.right() - (800.0 - 15.0)).abs() < 1e-9,
            "last cell right ({}) != figure_width - padding ({})",
            last.right(),
            800.0 - 15.0,
        );
        assert!(
            (last.bottom() - (600.0 - 15.0)).abs() < 1e-9,
            "last cell bottom ({}) != figure_height - padding ({})",
            last.bottom(),
            600.0 - 15.0,
        );
    }

    #[test]
    #[should_panic(expected = "nrows must be at least 1")]
    fn subplot_zero_rows_panics() {
        compute_subplot_rects(800.0, 600.0, 0, 2, 10.0, 20.0);
    }

    #[test]
    #[should_panic(expected = "ncols must be at least 1")]
    fn subplot_zero_cols_panics() {
        compute_subplot_rects(800.0, 600.0, 2, 0, 10.0, 20.0);
    }

    #[test]
    fn layout_in_rect_translates_correctly() {
        let cell = Rect::new(100.0, 50.0, 400.0, 300.0);
        let config = LayoutConfig::new(400.0, 300.0);

        let result = compute_layout_in_rect(&cell, &config);
        let pa = &result.plot_area;

        // The plot area must be inside the cell bounds.
        assert!(
            pa.x >= cell.x,
            "plot_area x ({}) < cell x ({})",
            pa.x,
            cell.x,
        );
        assert!(
            pa.y >= cell.y,
            "plot_area y ({}) < cell y ({})",
            pa.y,
            cell.y,
        );
        assert!(
            pa.right() <= cell.right(),
            "plot_area right ({}) > cell right ({})",
            pa.right(),
            cell.right(),
        );
        assert!(
            pa.bottom() <= cell.bottom(),
            "plot_area bottom ({}) > cell bottom ({})",
            pa.bottom(),
            cell.bottom(),
        );
    }

    #[test]
    fn margins_helpers() {
        let m = Margins::new(10.0, 20.0, 30.0, 40.0);
        assert!((m.horizontal() - 60.0).abs() < 1e-9);
        assert!((m.vertical() - 40.0).abs() < 1e-9);

        let u = Margins::uniform(15.0);
        assert_eq!(u.top, 15.0);
        assert_eq!(u.right, 15.0);
        assert_eq!(u.bottom, 15.0);
        assert_eq!(u.left, 15.0);
    }

    #[test]
    fn default_margins_are_zero() {
        let m = Margins::default();
        assert_eq!(m.top, 0.0);
        assert_eq!(m.right, 0.0);
        assert_eq!(m.bottom, 0.0);
        assert_eq!(m.left, 0.0);
    }
}
