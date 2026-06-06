//! Artist types -- data + styling for each visual chart element.
//!
//! Artists are the data-carrying objects stored in [`Axes`]. Each artist type
//! holds the data-space geometry and styling for one visual element. When the
//! figure is rendered, the renderer iterates over the artist list and draws
//! each one according to its variant.
//!
//! [`Axes`]: crate::axes::Axes
//!
//! # Variants
//!
//! | Variant          | Description                                     |
//! |------------------|-------------------------------------------------|
//! | [`Line`]         | A polyline connecting (x, y) points.             |
//! | [`Scatter`]      | Individual markers at (x, y) positions.          |
//! | [`Bar`]          | Vertical or horizontal bars over categories.     |
//! | [`Histogram`]    | Binned frequency distribution of a single series.|
//! | [`FillBetween`]  | Shaded region between two y-series.              |
//! | [`Pie`]          | A pie chart showing proportional wedge slices.   |
//! | [`Violin`]       | A violin plot showing kernel density estimates.  |
//!
//! [`Line`]: Artist::Line
//! [`Scatter`]: Artist::Scatter
//! [`Bar`]: Artist::Bar
//! [`Histogram`]: Artist::Histogram
//! [`FillBetween`]: Artist::FillBetween
//! [`Pie`]: Artist::Pie
//! [`Violin`]: Artist::Violin

use crate::charts::boxplot::BoxStats;
use crate::colormap::Colormap;
use crate::primitives::Color;
use crate::series::{Categories, Series};
use crate::theme::{LineStyle, Marker};

// ---------------------------------------------------------------------------
// Artist enum
// ---------------------------------------------------------------------------

/// A visual element drawn on an axes.
///
/// `Artist` is the primary unit of chart content. Each variant wraps a
/// concrete artist struct that stores the data, colors, and styling needed
/// to render one visual element. The enum provides convenience accessors
/// ([`label`](Artist::label), [`color`](Artist::color),
/// [`data_bounds`](Artist::data_bounds)) that dispatch to the inner type.
#[derive(Debug, Clone)]
pub enum Artist {
    /// A line chart connecting (x, y) points.
    Line(LineArtist),
    /// A scatter plot of individual points.
    Scatter(ScatterArtist),
    /// A bar chart (vertical or horizontal).
    Bar(BarArtist),
    /// A histogram (binned frequency distribution).
    Histogram(HistArtist),
    /// A filled region between two y-series sharing a common x-series.
    FillBetween(FillBetweenArtist),
    /// A step (staircase) chart connecting data points.
    Step(StepArtist),
    /// A stem (lollipop) chart from data points.
    Stem(StemArtist),
    /// A box-and-whisker plot showing distribution summaries.
    BoxPlot(BoxPlotArtist),
    /// An error bar plot showing data points with uncertainty bars.
    ErrorBar(ErrorBarArtist),
    /// A heatmap showing a 2D grid of values mapped to colors.
    Heatmap(HeatmapArtist),
    /// A pie chart showing proportional wedge slices.
    Pie(PieArtist),
    /// A violin plot showing kernel density estimates of distributions.
    Violin(ViolinArtist),
    /// A contour or filled contour plot over a 2D grid.
    Contour(ContourArtist),
}



impl Artist {
    /// Returns the legend label for this artist, if one has been set.
    ///
    /// The legend renderer uses this to decide which artists appear in the
    /// legend. Artists without a label are silently skipped.
    pub fn label(&self) -> Option<&str> {
        match self {
            Artist::Line(a) => a.label.as_deref(),
            Artist::Scatter(a) => a.label.as_deref(),
            Artist::Bar(a) => a.label.as_deref(),
            Artist::Histogram(a) => a.label.as_deref(),
            Artist::FillBetween(a) => a.label.as_deref(),
            Artist::Step(a) => a.label.as_deref(),
            Artist::Stem(a) => a.label.as_deref(),
            Artist::BoxPlot(a) => a.label.as_deref(),
            Artist::ErrorBar(a) => a.label.as_deref(),
            Artist::Heatmap(a) => a.label.as_deref(),
            Artist::Pie(a) => a.label.as_deref(),
            Artist::Violin(a) => a.label.as_deref(),
            Artist::Contour(a) => a.label.as_deref(),
        }
    }

    /// Returns the primary color of this artist.
    ///
    /// Used by the legend to draw a color swatch next to the label, and by
    /// any other component that needs to identify an artist's color (e.g.
    /// tooltip rendering).
    pub fn color(&self) -> Color {
        match self {
            Artist::Line(a) => a.color,
            Artist::Scatter(a) => a.color,
            Artist::Bar(a) => a.color,
            Artist::Histogram(a) => a.color,
            Artist::FillBetween(a) => a.color,
            Artist::Step(a) => a.color,
            Artist::Stem(a) => a.color,
            Artist::BoxPlot(a) => a.color,
            Artist::ErrorBar(a) => a.color,
            Artist::Heatmap(a) => a.color,
            Artist::Pie(a) => a.color,
            Artist::Violin(a) => a.color,
            Artist::Contour(a) => a.color,
        }
    }

    /// Returns the data-space bounding box as `(xmin, xmax, ymin, ymax)`.
    ///
    /// The axes autoscaling logic calls this on every artist to compute the
    /// tightest axis limits that contain all visible data. If a series is
    /// empty or contains no finite values, the corresponding min/max pair
    /// falls back to `(0.0, 1.0)` so that the axes always have a non-zero
    /// extent.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        match self {
            Artist::Line(a) => a.data_bounds(),
            Artist::Scatter(a) => a.data_bounds(),
            Artist::Bar(a) => a.data_bounds(),
            Artist::Histogram(a) => a.data_bounds(),
            Artist::FillBetween(a) => a.data_bounds(),
            Artist::Step(a) => a.data_bounds(),
            Artist::Stem(a) => a.data_bounds(),
            Artist::BoxPlot(a) => a.data_bounds(),
            Artist::ErrorBar(a) => a.data_bounds(),
            Artist::Heatmap(a) => a.data_bounds(),
            Artist::Pie(a) => a.data_bounds(),
            Artist::Violin(a) => a.data_bounds(),
            Artist::Contour(a) => a.data_bounds(),
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: safe bounds with fallback
// ---------------------------------------------------------------------------

/// Returns `(min, max)` of the finite values in `series`, falling back to
/// `(fallback_min, fallback_max)` when the series is empty or entirely
/// non-finite.
fn series_bounds_or(series: &Series, fallback_min: f64, fallback_max: f64) -> (f64, f64) {
    match series.bounds() {
        Some((lo, hi)) => (lo, hi),
        None => (fallback_min, fallback_max),
    }
}

// ---------------------------------------------------------------------------
// LineArtist
// ---------------------------------------------------------------------------

/// A line chart connecting a sequence of (x, y) data points.
///
/// The `x` and `y` series must have the same length. Points are drawn in
/// order, producing a single connected polyline with the configured stroke
/// style.
#[derive(Debug, Clone)]
pub struct LineArtist {
    /// X-coordinates of the data points.
    pub x: Series,
    /// Y-coordinates of the data points.
    pub y: Series,
    /// Stroke color of the line.
    pub color: Color,
    /// Stroke width in pixels.
    pub width: f64,
    /// Stroke pattern (solid, dashed, dotted, dash-dot).
    pub style: LineStyle,
    /// Optional legend label. When `Some`, the line appears in the legend.
    pub label: Option<String>,
    /// Opacity from 0.0 (fully transparent) to 1.0 (fully opaque).
    pub alpha: f64,
}

impl LineArtist {
    /// Computes the data-space bounding box `(xmin, xmax, ymin, ymax)`.
    ///
    /// Falls back to `(0.0, 1.0)` on each axis when the corresponding
    /// series contains no finite values.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        let (xmin, xmax) = series_bounds_or(&self.x, 0.0, 1.0);
        let (ymin, ymax) = series_bounds_or(&self.y, 0.0, 1.0);
        (xmin, xmax, ymin, ymax)
    }
}

// ---------------------------------------------------------------------------
// ScatterArtist
// ---------------------------------------------------------------------------

/// A scatter plot rendering individual markers at (x, y) positions.
///
/// Each data point is drawn as a marker whose shape, size, and color can be
/// configured. An optional per-point `colors` vector overrides the uniform
/// `color` field, enabling colormap-based visualizations.
#[derive(Debug, Clone)]
pub struct ScatterArtist {
    /// X-coordinates of the data points.
    pub x: Series,
    /// Y-coordinates of the data points.
    pub y: Series,
    /// Default marker color (used when `colors` is `None`).
    pub color: Color,
    /// Marker shape.
    pub marker: Marker,
    /// Marker diameter in pixels.
    pub size: f64,
    /// Optional legend label. When `Some`, the scatter appears in the legend.
    pub label: Option<String>,
    /// Opacity from 0.0 (fully transparent) to 1.0 (fully opaque).
    pub alpha: f64,
    /// Optional per-point colors for colormap-driven scatter plots.
    ///
    /// When set, `colors.len()` must equal `x.len()` (and `y.len()`). Each
    /// entry overrides `color` for the corresponding data point.
    pub colors: Option<Vec<Color>>,
    /// Optional per-point scalar values for colormap-driven coloring.
    ///
    /// When set together with `cmap`, each value is mapped through the
    /// colormap to produce per-point colors. Takes precedence over `colors`.
    pub c: Option<Vec<f64>>,
    /// Optional colormap used to map `c` values to colors.
    pub cmap: Option<Colormap>,
}

impl ScatterArtist {
    /// Computes the data-space bounding box `(xmin, xmax, ymin, ymax)`.
    ///
    /// Falls back to `(0.0, 1.0)` on each axis when the corresponding
    /// series contains no finite values.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        let (xmin, xmax) = series_bounds_or(&self.x, 0.0, 1.0);
        let (ymin, ymax) = series_bounds_or(&self.y, 0.0, 1.0);
        (xmin, xmax, ymin, ymax)
    }
}

// ---------------------------------------------------------------------------
// BarArtist
// ---------------------------------------------------------------------------

/// A bar chart rendering vertical or horizontal bars over categorical data.
///
/// Categories are placed at integer positions `0, 1, 2, ...` on the
/// category axis, with each bar centered on its position. The `bar_width`
/// field controls the fraction of the inter-category spacing that the bar
/// occupies (1.0 = bars touching, 0.5 = half-width with gaps).
///
/// For stacked bars, set `bottom` to offset each bar from a baseline other
/// than zero. For grouped (side-by-side) bars, adjust category positions and
/// `bar_width` for each series.
#[derive(Debug, Clone)]
pub struct BarArtist {
    /// Category labels for the bar axis.
    pub categories: Categories,
    /// Bar heights (or lengths, for horizontal bars).
    pub heights: Series,
    /// Fill color of the bars.
    pub color: Color,
    /// Optional legend label. When `Some`, the bar series appears in the legend.
    pub label: Option<String>,
    /// Opacity from 0.0 (fully transparent) to 1.0 (fully opaque).
    pub alpha: f64,
    /// When `true`, bars extend horizontally (categories on the y-axis).
    pub horizontal: bool,
    /// Bar width as a fraction of the category spacing (0.0, 1.0].
    pub bar_width: f64,
    /// Optional per-bar base offset for stacking.
    ///
    /// When `Some`, each bar starts at `bottom[i]` instead of `0.0` and extends
    /// to `bottom[i] + heights[i]`. The length must equal `heights.len()`.
    pub bottom: Option<Vec<f64>>,
    /// Optional per-bar x-position offset for grouped (side-by-side) bars.
    ///
    /// When `Some`, each bar's category center is shifted by `offset[i]` data
    /// units. The length must equal `heights.len()`.
    pub offset: Option<Vec<f64>>,
}

impl BarArtist {
    /// Computes the data-space bounding box `(xmin, xmax, ymin, ymax)`.
    ///
    /// For vertical bars, the x-axis spans from `-0.5` to `n - 0.5` (where
    /// `n` is the number of categories) so that bars are centered on integer
    /// positions. The y-axis spans from `0.0` to the tallest bar, with a
    /// fallback of `(0.0, 1.0)` when the heights series is empty.
    ///
    /// When `bottom` is set, the value axis includes both the bottom offsets
    /// and `bottom + height` values. When `offset` is set, the category axis
    /// is expanded to accommodate shifted bar positions.
    ///
    /// For horizontal bars the axes are transposed: the y-axis holds the
    /// category positions and the x-axis holds the bar lengths.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        let n = self.categories.len() as f64;

        // Determine the extent along the value axis (heights / lengths),
        // accounting for an optional bottom offset.
        let (height_min, height_max) = if let Some(ref bot) = self.bottom {
            let mut vmin = f64::INFINITY;
            let mut vmax = f64::NEG_INFINITY;
            for i in 0..self.heights.len() {
                let b = if i < bot.len() { bot[i] } else { 0.0 };
                let h = self.heights.data[i];
                let top = b + h;
                vmin = vmin.min(b).min(top);
                vmax = vmax.max(b).max(top);
            }
            if !vmin.is_finite() {
                vmin = 0.0;
            }
            if !vmax.is_finite() {
                vmax = 1.0;
            }
            // Ensure 0.0 is included when all values are positive or negative.
            (vmin.min(0.0), vmax)
        } else {
            let hmin = self.heights.min().unwrap_or(0.0).min(0.0);
            let hmax = self.heights.max().unwrap_or(1.0);
            (hmin, hmax)
        };

        // Category axis runs from -0.5 to n-0.5 so bars are centered on 0..n-1.
        // Expand if offsets push bars outside this range.
        let mut cat_min: f64 = -0.5;
        let mut cat_max: f64 = if n > 0.0 { n - 0.5 } else { 0.5 };
        if let Some(ref off) = self.offset {
            let half_bar = self.bar_width * 0.5;
            for i in 0..self.heights.len() {
                let o = if i < off.len() { off[i] } else { 0.0 };
                let center = i as f64 + o;
                cat_min = cat_min.min(center - half_bar);
                cat_max = cat_max.max(center + half_bar);
            }
        }

        if self.horizontal {
            // Horizontal bars: x = value axis, y = category axis.
            (height_min, height_max, cat_min, cat_max)
        } else {
            // Vertical bars: x = category axis, y = value axis.
            (cat_min, cat_max, height_min, height_max)
        }
    }
}

// ---------------------------------------------------------------------------
// HistArtist
// ---------------------------------------------------------------------------

/// A histogram showing the frequency distribution of a single data series.
///
/// The raw data is retained in `data`, but the binning results (`bin_edges`
/// and `counts`) are expected to be pre-computed when the artist is created
/// (typically by the histogram chart builder). This avoids re-binning during
/// every render pass.
///
/// When `density` is `true`, the `counts` vector stores probability density
/// values (each count divided by `n * bin_width`) rather than raw counts, so
/// that the total area under the histogram integrates to 1.0.
#[derive(Debug, Clone)]
pub struct HistArtist {
    /// The original (un-binned) data values.
    pub data: Series,
    /// The requested number of bins (used for display/debugging; the actual
    /// bin count is `bin_edges.len() - 1`).
    pub bins: usize,
    /// Sorted bin edges of length `bins + 1`. The i-th bin spans
    /// `[bin_edges[i], bin_edges[i+1])`.
    pub bin_edges: Vec<f64>,
    /// The count (or density) for each bin. Length equals `bin_edges.len() - 1`.
    pub counts: Vec<f64>,
    /// Fill color of the histogram bars.
    pub color: Color,
    /// Optional legend label. When `Some`, the histogram appears in the legend.
    pub label: Option<String>,
    /// Opacity from 0.0 (fully transparent) to 1.0 (fully opaque).
    pub alpha: f64,
    /// When `true`, `counts` stores probability density instead of raw counts.
    pub density: bool,
}

impl HistArtist {
    /// Computes the data-space bounding box `(xmin, xmax, ymin, ymax)`.
    ///
    /// The x-axis spans from the first bin edge to the last bin edge. The
    /// y-axis spans from `0.0` to the tallest bin count (or density value).
    /// Returns `(0.0, 1.0, 0.0, 1.0)` when there are no bin edges.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        if self.bin_edges.len() < 2 {
            return (0.0, 1.0, 0.0, 1.0);
        }

        // x-axis: first edge to last edge.
        let xmin = self.bin_edges[0];
        let xmax = self.bin_edges[self.bin_edges.len() - 1];

        // y-axis: 0 to tallest bin.
        let ymax = self
            .counts
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .fold(0.0_f64, f64::max);

        // Guarantee a non-zero y extent so the axes are always drawable.
        let ymax = if ymax <= 0.0 { 1.0 } else { ymax };

        (xmin, xmax, 0.0, ymax)
    }
}

// ---------------------------------------------------------------------------
// FillBetweenArtist
// ---------------------------------------------------------------------------

/// A filled region between two y-series that share a common x-series.
///
/// The renderer draws a closed polygon connecting `(x, y1)` forward and
/// `(x, y2)` backward, then fills it with the configured color and opacity.
/// This is commonly used for confidence bands, area charts, and shaded
/// difference regions.
#[derive(Debug, Clone)]
pub struct FillBetweenArtist {
    /// X-coordinates shared by both y-series.
    pub x: Series,
    /// Y-coordinates of the first boundary curve.
    pub y1: Series,
    /// Y-coordinates of the second boundary curve.
    pub y2: Series,
    /// Fill color of the shaded region.
    pub color: Color,
    /// Optional legend label. When `Some`, the fill region appears in the legend.
    pub label: Option<String>,
    /// Opacity from 0.0 (fully transparent) to 1.0 (fully opaque).
    pub alpha: f64,
}

impl FillBetweenArtist {
    /// Computes the data-space bounding box `(xmin, xmax, ymin, ymax)`.
    ///
    /// The x-bounds come from the shared `x` series. The y-bounds are the
    /// union of `y1` and `y2` (i.e. the overall min and max across both
    /// boundary curves). Falls back to `(0.0, 1.0)` on any axis that has
    /// no finite values.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        let (xmin, xmax) = series_bounds_or(&self.x, 0.0, 1.0);

        // Union the y-bounds of both boundary series.
        let y1_min = self.y1.min();
        let y2_min = self.y2.min();
        let y1_max = self.y1.max();
        let y2_max = self.y2.max();

        let ymin = match (y1_min, y2_min) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => 0.0,
        };

        let ymax = match (y1_max, y2_max) {
            (Some(a), Some(b)) => a.max(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => 1.0,
        };

        (xmin, xmax, ymin, ymax)
    }
}

// ---------------------------------------------------------------------------
// BoxPlotArtist
// ---------------------------------------------------------------------------

/// A box-and-whisker plot showing distribution summaries for one or more
/// groups of data.
///
/// Each group produces a box spanning Q1 to Q3 with a median line, whiskers
/// extending to the most extreme data points within the configured fence, and
/// optional outlier dots beyond the whiskers.
#[derive(Debug, Clone)]
pub struct BoxPlotArtist {
    /// Pre-computed summary statistics for each group.
    pub stats: Vec<BoxStats>,
    /// Category labels for the x-axis (one per group).
    pub labels: Vec<String>,
    /// Fill color of the boxes.
    pub color: Color,
    /// Optional legend label.
    pub label: Option<String>,
    /// Opacity from 0.0 (fully transparent) to 1.0 (fully opaque).
    pub alpha: f64,
    /// Box width as a fraction of the category spacing.
    pub box_width: f64,
    /// Whether to draw outlier dots.
    pub show_outliers: bool,
    /// Whisker extent as a multiple of IQR.
    pub whisker_iq_factor: f64,
    /// Raw data retained for re-computing stats when parameters change.
    pub raw_data: Vec<Vec<f64>>,
}

impl BoxPlotArtist {
    /// Computes the data-space bounding box `(xmin, xmax, ymin, ymax)`.
    ///
    /// The x-axis spans from `-0.5` to `n - 0.5` (where `n` is the number
    /// of groups), centering each box on an integer position. The y-axis
    /// spans from the lowest whisker (or outlier) to the highest.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        let n = self.stats.len();
        if n == 0 {
            return (0.0, 1.0, 0.0, 1.0);
        }
        let xmin = -0.5;
        let xmax = n as f64 - 0.5;
        let mut ymin = f64::INFINITY;
        let mut ymax = f64::NEG_INFINITY;
        for s in &self.stats {
            ymin = ymin.min(s.whisker_low);
            ymax = ymax.max(s.whisker_high);
            for &o in &s.outliers {
                ymin = ymin.min(o);
                ymax = ymax.max(o);
            }
        }
        if !ymin.is_finite() {
            ymin = 0.0;
        }
        if !ymax.is_finite() {
            ymax = 1.0;
        }
        (xmin, xmax, ymin, ymax)
    }
}


// ---------------------------------------------------------------------------
// ErrorBarData
// ---------------------------------------------------------------------------

/// The error data for one axis of an error bar plot.
///
/// Symmetric errors apply the same magnitude on both sides of the data point.
/// Asymmetric errors allow separate low and high magnitudes.
#[derive(Debug, Clone)]
pub enum ErrorBarData {
    /// Equal error on both sides: `y - e` to `y + e`.
    Symmetric(Vec<f64>),
    /// Separate low and high errors: `y - low[i]` to `y + high[i]`.
    Asymmetric {
        /// Error magnitudes below each data point.
        low: Vec<f64>,
        /// Error magnitudes above each data point.
        high: Vec<f64>,
    },
}

// ---------------------------------------------------------------------------
// ErrorBarArtist
// ---------------------------------------------------------------------------

/// An error bar plot showing data points with uncertainty bars.
///
/// Each data point `(x, y)` can have optional horizontal (`xerr`) and/or
/// vertical (`yerr`) error bars. The error bars are drawn as lines with
/// optional caps at the ends.
#[derive(Debug, Clone)]
pub struct ErrorBarArtist {
    /// X-coordinates of the data points.
    pub x: Series,
    /// Y-coordinates of the data points.
    pub y: Series,
    /// Optional x-axis error data.
    pub xerr: Option<ErrorBarData>,
    /// Optional y-axis error data.
    pub yerr: Option<ErrorBarData>,
    /// Color for the center line, error bars, and caps.
    pub color: Color,
    /// Optional legend label.
    pub label: Option<String>,
    /// Cap size in pixels for the error bar ends.
    pub cap_size: f64,
    /// Stroke width of the error bar lines and caps.
    pub line_width: f64,
}

impl ErrorBarArtist {
    /// Computes the data-space bounding box `(xmin, xmax, ymin, ymax)`.
    ///
    /// Includes the extent of error bars when present, so that auto-scaling
    /// shows the full error range.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        let (mut xmin, mut xmax) = series_bounds_or(&self.x, 0.0, 1.0);
        let (mut ymin, mut ymax) = series_bounds_or(&self.y, 0.0, 1.0);

        // Expand x-bounds by xerr.
        if let Some(ref xerr) = self.xerr {
            for i in 0..self.x.len() {
                let xv = self.x.data[i];
                let (lo, hi) = match xerr {
                    ErrorBarData::Symmetric(e) => (xv - e[i], xv + e[i]),
                    ErrorBarData::Asymmetric { low, high } => (xv - low[i], xv + high[i]),
                };
                xmin = xmin.min(lo);
                xmax = xmax.max(hi);
            }
        }

        // Expand y-bounds by yerr.
        if let Some(ref yerr) = self.yerr {
            for i in 0..self.y.len() {
                let yv = self.y.data[i];
                let (lo, hi) = match yerr {
                    ErrorBarData::Symmetric(e) => (yv - e[i], yv + e[i]),
                    ErrorBarData::Asymmetric { low, high } => (yv - low[i], yv + high[i]),
                };
                ymin = ymin.min(lo);
                ymax = ymax.max(hi);
            }
        }

        (xmin, xmax, ymin, ymax)
    }
}

// ---------------------------------------------------------------------------
// HeatmapArtist
// ---------------------------------------------------------------------------

/// A heatmap showing a 2D grid of values mapped to colors via a colormap.
///
/// Each cell in the grid is filled with a color determined by mapping its
/// value through the configured [`Colormap`]. Optional text annotations
/// can display the numeric value inside each cell.
#[derive(Debug, Clone)]
pub struct HeatmapArtist {
    /// Row-major grid of values. `data[row][col]`.
    pub data: Vec<Vec<f64>>,
    /// Optional column labels for the x-axis.
    pub x_labels: Option<Vec<String>>,
    /// Optional row labels for the y-axis.
    pub y_labels: Option<Vec<String>>,
    /// Colormap used to map cell values to colors.
    pub cmap: Colormap,
    /// Minimum value for colormap normalisation. `None` means auto.
    pub vmin: Option<f64>,
    /// Maximum value for colormap normalisation. `None` means auto.
    pub vmax: Option<f64>,
    /// Whether to draw cell values as text.
    pub show_values: bool,
    /// Primary color (used for legend swatch).
    pub color: Color,
    /// Optional legend label.
    pub label: Option<String>,
    /// Whether to auto-attach a colorbar when this heatmap is drawn.
    pub show_colorbar: bool,
}

impl HeatmapArtist {
    /// Computes the data-space bounding box `(xmin, xmax, ymin, ymax)`.
    ///
    /// The grid spans from `(0, 0)` to `(ncols, nrows)`. Returns
    /// `(0.0, 1.0, 0.0, 1.0)` when the data is empty.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        let nrows = self.data.len();
        if nrows == 0 {
            return (0.0, 1.0, 0.0, 1.0);
        }
        let ncols = self.data[0].len();
        if ncols == 0 {
            return (0.0, 1.0, 0.0, 1.0);
        }
        (0.0, ncols as f64, 0.0, nrows as f64)
    }
}

// ---------------------------------------------------------------------------
// PieArtist
// ---------------------------------------------------------------------------

/// A pie chart rendering proportional wedge slices from a set of sizes.
///
/// Each entry in `sizes` is automatically normalised so that the wedges
/// sum to a full circle. The starting angle, explode offsets, and colors
/// can be customised through the builder API.
#[derive(Debug, Clone)]
pub struct PieArtist {
    /// Wedge sizes (auto-normalised to sum to 1.0 during rendering).
    pub sizes: Vec<f64>,
    /// Optional labels for each wedge, drawn outside the wedge arc.
    pub labels: Option<Vec<String>>,
    /// Optional custom colors for each wedge. When `None`, the theme
    /// color cycle is used.
    pub colors: Option<Vec<Color>>,
    /// Optional explode offset for each wedge, as a fraction of the
    /// radius. A value of `0.0` means no offset.
    pub explode: Option<Vec<f64>>,
    /// When `true`, percentage labels are drawn at the midpoint of each
    /// wedge arc.
    pub autopct: bool,
    /// Starting angle in degrees, counter-clockwise from the positive
    /// x-axis. Default is `90.0` (top of the circle).
    pub start_angle: f64,
    /// Radius of the pie in data-space units. Default is `1.0`.
    pub radius: f64,
    /// Optional legend label. When `Some`, the pie appears in the legend.
    pub label: Option<String>,
    /// Primary color (used for legend swatch when no custom colors are set).
    pub color: Color,
}

impl PieArtist {
    /// Computes the data-space bounding box `(xmin, xmax, ymin, ymax)`.
    ///
    /// Returns a fixed square region that accommodates the pie radius plus
    /// any explode offsets, with a small margin so that labels do not get
    /// clipped.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        let max_explode = self
            .explode
            .as_ref()
            .map(|e| e.iter().copied().fold(0.0_f64, f64::max))
            .unwrap_or(0.0);
        let extent = self.radius * (1.0 + max_explode) + 0.1 * self.radius;
        (-extent, extent, -extent, extent)
    }
}

// ---------------------------------------------------------------------------
// ViolinArtist
// ---------------------------------------------------------------------------

/// A violin plot showing the probability density of data distributions.
///
/// Each dataset produces a mirrored kernel density estimate (KDE) shape,
/// similar to a boxplot but showing the full distribution. Optional median
/// and quartile lines can be drawn inside the violin.
#[derive(Debug, Clone)]
pub struct ViolinArtist {
    /// One dataset per violin. Each inner `Vec<f64>` contains the raw values.
    pub datasets: Vec<Vec<f64>>,
    /// Optional x-positions for each violin. Defaults to 1, 2, 3, etc.
    pub positions: Option<Vec<f64>>,
    /// Maximum width of each violin shape.
    pub widths: f64,
    /// Whether to draw a median line inside each violin.
    pub show_median: bool,
    /// Whether to draw Q1/Q3 quartile lines inside each violin.
    pub show_quartiles: bool,
    /// Fill color of the violin shapes.
    pub color: Color,
    /// Opacity from 0.0 (fully transparent) to 1.0 (fully opaque).
    pub alpha: f64,
    /// Optional legend label.
    pub label: Option<String>,
    /// KDE bandwidth override. When <= 0.0, Silverman's rule is used.
    pub bw_method: f64,
}

// ---------------------------------------------------------------------------
// ContourArtist
// ---------------------------------------------------------------------------

/// A contour or filled contour plot over a 2D grid of z = f(x, y) values.
///
/// In unfilled mode (`filled = false`), iso-lines are drawn at each contour
/// level using the marching squares algorithm. In filled mode (`filled = true`),
/// the regions between contour levels are filled with colors from a colormap.
#[derive(Debug, Clone)]
pub struct ContourArtist {
    /// X grid coordinates (length `nx`).
    pub x: Vec<f64>,
    /// Y grid coordinates (length `ny`).
    pub y: Vec<f64>,
    /// Z values on the grid, shape `[ny][nx]` (row-major).
    pub z: Vec<Vec<f64>>,
    /// Explicit contour levels. When `None`, levels are auto-computed.
    pub levels: Option<Vec<f64>>,
    /// Whether to fill regions between levels (`true` for contourf).
    pub filled: bool,
    /// Colormap used to map contour levels to colors.
    pub cmap: Colormap,
    /// Optional explicit colors for each contour level, overriding the colormap.
    pub colors: Option<Vec<Color>>,
    /// Stroke width for contour lines (unfilled mode). Default `1.0`.
    pub linewidths: f64,
    /// Optional legend label.
    pub label: Option<String>,
    /// Primary color (used for legend swatch).
    pub color: Color,
    /// Number of auto-computed levels when `levels` is `None`. Default `10`.
    pub num_levels: usize,
}

impl ContourArtist {
    /// Computes the data-space bounding box `(xmin, xmax, ymin, ymax)`.
    ///
    /// Returns the extent of the x and y grid coordinates. Falls back to
    /// `(0.0, 1.0, 0.0, 1.0)` when either coordinate vector is empty.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        if self.x.is_empty() || self.y.is_empty() {
            return (0.0, 1.0, 0.0, 1.0);
        }
        let xmin = self.x.iter().copied().fold(f64::INFINITY, f64::min);
        let xmax = self.x.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let ymin = self.y.iter().copied().fold(f64::INFINITY, f64::min);
        let ymax = self.y.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        let (xmin, xmax) = if xmin.is_finite() && xmax.is_finite() {
            (xmin, xmax)
        } else {
            (0.0, 1.0)
        };
        let (ymin, ymax) = if ymin.is_finite() && ymax.is_finite() {
            (ymin, ymax)
        } else {
            (0.0, 1.0)
        };
        (xmin, xmax, ymin, ymax)
    }
}

// ---------------------------------------------------------------------------
// StepWhere
// ---------------------------------------------------------------------------

/// Controls where the horizontal segment of a step chart is placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepWhere {
    /// The y-value changes *before* the x-value (vertical then horizontal).
    Pre,
    /// The y-value changes *after* the x-value (horizontal then vertical).
    Post,
    /// The y-value changes at the midpoint between consecutive x-values.
    Mid,
}

// ---------------------------------------------------------------------------
// StepArtist
// ---------------------------------------------------------------------------

/// A step (staircase) chart.
#[derive(Debug, Clone)]
pub struct StepArtist {
    /// X-coordinates of the data points.
    pub x: Series,
    /// Y-coordinates of the data points.
    pub y: Series,
    /// Stroke color of the step line.
    pub color: Color,
    /// Stroke width in pixels.
    pub width: f64,
    /// Step alignment mode.
    pub where_step: StepWhere,
    /// Optional legend label.
    pub label: Option<String>,
    /// Opacity from 0.0 (fully transparent) to 1.0 (fully opaque).
    pub alpha: f64,
}

impl StepArtist {
    /// Computes the data-space bounding box `(xmin, xmax, ymin, ymax)`.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        let (xmin, xmax) = series_bounds_or(&self.x, 0.0, 1.0);
        let (ymin, ymax) = series_bounds_or(&self.y, 0.0, 1.0);
        (xmin, xmax, ymin, ymax)
    }
}

// ---------------------------------------------------------------------------
// StemArtist
// ---------------------------------------------------------------------------

/// A stem (lollipop) chart.
#[derive(Debug, Clone)]
pub struct StemArtist {
    /// X-coordinates of the data points.
    pub x: Series,
    /// Y-coordinates of the data points.
    pub y: Series,
    /// Color of the stem lines and markers.
    pub color: Color,
    /// Stroke width of the stem lines in pixels.
    pub line_width: f64,
    /// Diameter of the marker circle in pixels.
    pub marker_size: f64,
    /// The y-value from which stems originate.
    pub baseline: f64,
    /// Optional legend label.
    pub label: Option<String>,
    /// Opacity from 0.0 (fully transparent) to 1.0 (fully opaque).
    pub alpha: f64,
}

impl StemArtist {
    /// Computes the data-space bounding box `(xmin, xmax, ymin, ymax)`.
    ///
    /// The y-bounds include the baseline.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        let (xmin, xmax) = series_bounds_or(&self.x, 0.0, 1.0);
        let (ymin, ymax) = series_bounds_or(&self.y, 0.0, 1.0);
        (xmin, xmax, ymin.min(self.baseline), ymax.max(self.baseline))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a simple `LineArtist` for testing.
    fn sample_line() -> LineArtist {
        LineArtist {
            x: Series::new(vec![1.0, 2.0, 3.0]),
            y: Series::new(vec![10.0, 20.0, 30.0]),
            color: Color::TAB_BLUE,
            width: 1.5,
            style: LineStyle::Solid,
            label: Some("line".to_string()),
            alpha: 1.0,
        }
    }

    /// Helper: build a simple `ScatterArtist` for testing.
    fn sample_scatter() -> ScatterArtist {
        ScatterArtist {
            x: Series::new(vec![0.0, 5.0, 10.0]),
            y: Series::new(vec![-1.0, 0.0, 1.0]),
            color: Color::TAB_ORANGE,
            marker: Marker::Circle,
            size: 6.0,
            label: None,
            alpha: 0.8,
            colors: None,
            c: None,
            cmap: None,
        }
    }

    /// Helper: build a simple `BarArtist` for testing.
    fn sample_bar() -> BarArtist {
        BarArtist {
            categories: Categories::new(vec!["A".into(), "B".into(), "C".into()]),
            heights: Series::new(vec![4.0, 7.0, 2.0]),
            color: Color::TAB_GREEN,
            label: Some("bars".to_string()),
            alpha: 1.0,
            horizontal: false,
            bar_width: 0.8,
           bottom: None,
            offset: None,
        }
    }

    /// Helper: build a simple `HistArtist` for testing.
    fn sample_hist() -> HistArtist {
        HistArtist {
            data: Series::new(vec![1.0, 2.0, 2.5, 3.0, 3.5, 4.0]),
            bins: 3,
            bin_edges: vec![1.0, 2.0, 3.0, 4.0],
            counts: vec![1.0, 2.0, 3.0],
            color: Color::TAB_RED,
            label: Some("hist".to_string()),
            alpha: 0.7,
            density: false,
        }
    }

    /// Helper: build a simple `FillBetweenArtist` for testing.
    fn sample_fill_between() -> FillBetweenArtist {
        FillBetweenArtist {
            x: Series::new(vec![0.0, 1.0, 2.0]),
            y1: Series::new(vec![1.0, 3.0, 2.0]),
            y2: Series::new(vec![0.0, 1.0, 0.5]),
            color: Color::TAB_PURPLE,
            label: Some("fill".to_string()),
            alpha: 0.3,
        }
    }

    // -- Artist enum dispatch -----------------------------------------------

    #[test]
    fn artist_label_returns_inner_label() {
        let a = Artist::Line(sample_line());
        assert_eq!(a.label(), Some("line"));

        let a = Artist::Scatter(sample_scatter());
        assert_eq!(a.label(), None);

        let a = Artist::Bar(sample_bar());
        assert_eq!(a.label(), Some("bars"));

        let a = Artist::Histogram(sample_hist());
        assert_eq!(a.label(), Some("hist"));

        let a = Artist::FillBetween(sample_fill_between());
        assert_eq!(a.label(), Some("fill"));
    }

    #[test]
    fn artist_color_returns_inner_color() {
        assert_eq!(Artist::Line(sample_line()).color(), Color::TAB_BLUE);
        assert_eq!(Artist::Scatter(sample_scatter()).color(), Color::TAB_ORANGE);
        assert_eq!(Artist::Bar(sample_bar()).color(), Color::TAB_GREEN);
        assert_eq!(Artist::Histogram(sample_hist()).color(), Color::TAB_RED);
        assert_eq!(
            Artist::FillBetween(sample_fill_between()).color(),
            Color::TAB_PURPLE
        );
    }

    #[test]
    fn artist_data_bounds_dispatches_correctly() {
        let a = Artist::Line(sample_line());
        assert_eq!(a.data_bounds(), (1.0, 3.0, 10.0, 30.0));
    }

    // -- LineArtist ---------------------------------------------------------

    #[test]
    fn line_data_bounds_basic() {
        let a = sample_line();
        assert_eq!(a.data_bounds(), (1.0, 3.0, 10.0, 30.0));
    }

    #[test]
    fn line_data_bounds_empty_series() {
        let a = LineArtist {
            x: Series::new(vec![]),
            y: Series::new(vec![]),
            color: Color::BLACK,
            width: 1.0,
            style: LineStyle::Solid,
            label: None,
            alpha: 1.0,
        };
        assert_eq!(a.data_bounds(), (0.0, 1.0, 0.0, 1.0));
    }

    #[test]
    fn line_data_bounds_with_nan() {
        let a = LineArtist {
            x: Series::new(vec![f64::NAN, 2.0, 5.0]),
            y: Series::new(vec![1.0, f64::NAN, 3.0]),
            color: Color::BLACK,
            width: 1.0,
            style: LineStyle::Solid,
            label: None,
            alpha: 1.0,
        };
        assert_eq!(a.data_bounds(), (2.0, 5.0, 1.0, 3.0));
    }

    // -- ScatterArtist ------------------------------------------------------

    #[test]
    fn scatter_data_bounds_basic() {
        let a = sample_scatter();
        assert_eq!(a.data_bounds(), (0.0, 10.0, -1.0, 1.0));
    }

    #[test]
    fn scatter_data_bounds_empty() {
        let a = ScatterArtist {
            x: Series::new(vec![]),
            y: Series::new(vec![]),
            color: Color::BLACK,
            marker: Marker::Circle,
            size: 6.0,
            label: None,
            alpha: 1.0,
            colors: None,
            c: None,
            cmap: None,
        };
        assert_eq!(a.data_bounds(), (0.0, 1.0, 0.0, 1.0));
    }

    // -- BarArtist ----------------------------------------------------------

    #[test]
    fn bar_data_bounds_vertical() {
        let a = sample_bar();
        let (xmin, xmax, ymin, ymax) = a.data_bounds();
        assert!((xmin - (-0.5)).abs() < f64::EPSILON);
        assert!((xmax - 2.5).abs() < f64::EPSILON);
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bar_data_bounds_horizontal() {
        let mut a = sample_bar();
        a.horizontal = true;
        let (xmin, xmax, ymin, ymax) = a.data_bounds();
        // Horizontal: x = value axis, y = category axis.
        assert!((xmin - 0.0).abs() < f64::EPSILON);
        assert!((xmax - 7.0).abs() < f64::EPSILON);
        assert!((ymin - (-0.5)).abs() < f64::EPSILON);
        assert!((ymax - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn bar_data_bounds_negative_heights() {
        let a = BarArtist {
            categories: Categories::new(vec!["A".into(), "B".into()]),
            heights: Series::new(vec![-3.0, 5.0]),
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            horizontal: false,
            bar_width: 0.8,
           bottom: None,
            offset: None,
        };
        let (_, _, ymin, ymax) = a.data_bounds();
        assert!((ymin - (-3.0)).abs() < f64::EPSILON);
        assert!((ymax - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bar_data_bounds_empty() {
        let a = BarArtist {
            categories: Categories::new(vec![]),
            heights: Series::new(vec![]),
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            horizontal: false,
            bar_width: 0.8,
           bottom: None,
            offset: None,
        };
        let (xmin, xmax, ymin, ymax) = a.data_bounds();
        assert!((xmin - (-0.5)).abs() < f64::EPSILON);
        assert!((xmax - 0.5).abs() < f64::EPSILON);
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 1.0).abs() < f64::EPSILON);
    }

    // -- BarArtist with bottom (stacking) -----------------------------------

    #[test]
    fn bar_data_bounds_with_bottom() {
        let a = BarArtist {
            categories: Categories::new(vec!["A".into(), "B".into(), "C".into()]),
            heights: Series::new(vec![3.0, 4.0, 2.0]),
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            horizontal: false,
            bar_width: 0.8,
            bottom: Some(vec![1.0, 2.0, 3.0]),
            offset: None,
        };
        let (_, _, ymin, ymax) = a.data_bounds();
        // bottom[0]=1, top[0]=4; bottom[1]=2, top[1]=6; bottom[2]=3, top[2]=5
        // min includes 0.0 (ensured), max = 6.0
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bar_data_bounds_with_bottom_negative_base() {
        let a = BarArtist {
            categories: Categories::new(vec!["A".into(), "B".into()]),
            heights: Series::new(vec![5.0, 3.0]),
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            horizontal: false,
            bar_width: 0.8,
            bottom: Some(vec![-2.0, 1.0]),
            offset: None,
        };
        let (_, _, ymin, ymax) = a.data_bounds();
        assert!((ymin - (-2.0)).abs() < f64::EPSILON);
        assert!((ymax - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bar_data_bounds_with_bottom_horizontal() {
        let a = BarArtist {
            categories: Categories::new(vec!["X".into(), "Y".into()]),
            heights: Series::new(vec![4.0, 6.0]),
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            horizontal: true,
            bar_width: 0.8,
            bottom: Some(vec![1.0, 2.0]),
            offset: None,
        };
        let (xmin, xmax, ymin, ymax) = a.data_bounds();
        // Horizontal: x = value axis, y = category axis.
        assert!((xmin - 0.0).abs() < f64::EPSILON);
        assert!((xmax - 8.0).abs() < f64::EPSILON);
        assert!((ymin - (-0.5)).abs() < f64::EPSILON);
        assert!((ymax - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn bar_data_bounds_with_offset() {
        let a = BarArtist {
            categories: Categories::new(vec!["A".into(), "B".into()]),
            heights: Series::new(vec![5.0, 3.0]),
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            horizontal: false,
            bar_width: 0.4,
            bottom: None,
            offset: Some(vec![-0.2, -0.2]),
        };
        let (xmin, _xmax, _, _) = a.data_bounds();
        // center for bar 0 = 0 + (-0.2) = -0.2, left edge = -0.2 - 0.2 = -0.4
        assert!(xmin <= -0.4);
    }

    #[test]
    fn bar_data_bounds_bottom_and_offset_combined() {
        let a = BarArtist {
            categories: Categories::new(vec!["A".into(), "B".into()]),
            heights: Series::new(vec![3.0, 4.0]),
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            horizontal: false,
            bar_width: 0.4,
            bottom: Some(vec![2.0, 1.0]),
            offset: Some(vec![0.2, 0.2]),
        };
        let (_, _, ymin, ymax) = a.data_bounds();
        // bottoms: 2,1; tops: 5,5; min(all,0)=0; max=5
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bar_data_bounds_single_bar_with_bottom() {
        let a = BarArtist {
            categories: Categories::new(vec!["Solo".into()]),
            heights: Series::new(vec![10.0]),
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            horizontal: false,
            bar_width: 0.8,
            bottom: Some(vec![5.0]),
            offset: None,
        };
        let (_, _, ymin, ymax) = a.data_bounds();
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bar_data_bounds_zero_bottom() {
        // Setting bottom to all zeros should behave identically to no bottom.
        let a = BarArtist {
            categories: Categories::new(vec!["A".into(), "B".into()]),
            heights: Series::new(vec![3.0, 5.0]),
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            horizontal: false,
            bar_width: 0.8,
            bottom: Some(vec![0.0, 0.0]),
            offset: None,
        };
        let (_, _, ymin, ymax) = a.data_bounds();
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bar_data_bounds_empty_with_bottom() {
        let a = BarArtist {
            categories: Categories::new(vec![]),
            heights: Series::new(vec![]),
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            horizontal: false,
            bar_width: 0.8,
            bottom: Some(vec![]),
            offset: None,
        };
        let (xmin, xmax, ymin, ymax) = a.data_bounds();
        assert!((xmin - (-0.5)).abs() < f64::EPSILON);
        assert!((xmax - 0.5).abs() < f64::EPSILON);
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bar_data_bounds_stacked_three_layers() {
        // Simulates top layer of a 3-layer stack: bottom=5, height=1 => top=6.
        let a = BarArtist {
            categories: Categories::new(vec!["A".into()]),
            heights: Series::new(vec![1.0]),
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            horizontal: false,
            bar_width: 0.8,
            bottom: Some(vec![5.0]),
            offset: None,
        };
        let (_, _, ymin, ymax) = a.data_bounds();
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bar_builder_bottom_sets_field() {
        let mut a = sample_bar();
        a.bottom(vec![1.0, 2.0, 3.0]);
        assert_eq!(a.bottom.as_ref().unwrap(), &vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn bar_builder_offset_sets_field() {
        let mut a = sample_bar();
        a.offset(vec![0.1, 0.2, 0.3]);
        assert_eq!(a.offset.as_ref().unwrap(), &vec![0.1, 0.2, 0.3]);
    }

    // -- HistArtist ---------------------------------------------------------

    #[test]
    fn hist_data_bounds_basic() {
        let a = sample_hist();
        let (xmin, xmax, ymin, ymax) = a.data_bounds();
        assert!((xmin - 1.0).abs() < f64::EPSILON);
        assert!((xmax - 4.0).abs() < f64::EPSILON);
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hist_data_bounds_empty_bins() {
        let a = HistArtist {
            data: Series::new(vec![]),
            bins: 0,
            bin_edges: vec![],
            counts: vec![],
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            density: false,
        };
        assert_eq!(a.data_bounds(), (0.0, 1.0, 0.0, 1.0));
    }

    #[test]
    fn hist_data_bounds_single_edge_pair() {
        let a = HistArtist {
            data: Series::new(vec![1.0]),
            bins: 1,
            bin_edges: vec![0.5, 1.5],
            counts: vec![1.0],
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            density: false,
        };
        let (xmin, xmax, ymin, ymax) = a.data_bounds();
        assert!((xmin - 0.5).abs() < f64::EPSILON);
        assert!((xmax - 1.5).abs() < f64::EPSILON);
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hist_data_bounds_all_zero_counts() {
        let a = HistArtist {
            data: Series::new(vec![]),
            bins: 2,
            bin_edges: vec![0.0, 1.0, 2.0],
            counts: vec![0.0, 0.0],
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            density: false,
        };
        let (_, _, _, ymax) = a.data_bounds();
        // All-zero counts should produce a fallback ymax of 1.0.
        assert!((ymax - 1.0).abs() < f64::EPSILON);
    }

    // -- FillBetweenArtist --------------------------------------------------

    #[test]
    fn fill_between_data_bounds_basic() {
        let a = sample_fill_between();
        let (xmin, xmax, ymin, ymax) = a.data_bounds();
        assert!((xmin - 0.0).abs() < f64::EPSILON);
        assert!((xmax - 2.0).abs() < f64::EPSILON);
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fill_between_data_bounds_empty() {
        let a = FillBetweenArtist {
            x: Series::new(vec![]),
            y1: Series::new(vec![]),
            y2: Series::new(vec![]),
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
        };
        assert_eq!(a.data_bounds(), (0.0, 1.0, 0.0, 1.0));
    }

    #[test]
    fn fill_between_data_bounds_y2_extends_beyond_y1() {
        let a = FillBetweenArtist {
            x: Series::new(vec![0.0, 1.0]),
            y1: Series::new(vec![1.0, 2.0]),
            y2: Series::new(vec![-5.0, 10.0]),
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
        };
        let (_, _, ymin, ymax) = a.data_bounds();
        assert!((ymin - (-5.0)).abs() < f64::EPSILON);
        assert!((ymax - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fill_between_data_bounds_one_series_empty() {
        // y1 has data, y2 is empty -- bounds should come from y1 alone.
        let a = FillBetweenArtist {
            x: Series::new(vec![0.0, 1.0]),
            y1: Series::new(vec![2.0, 8.0]),
            y2: Series::new(vec![]),
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
        };
        let (_, _, ymin, ymax) = a.data_bounds();
        assert!((ymin - 2.0).abs() < f64::EPSILON);
        assert!((ymax - 8.0).abs() < f64::EPSILON);
    }
}
