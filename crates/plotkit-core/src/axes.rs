//! The Axes container -- holds chart data and renders a single subplot.
//!
//! `Axes` is the central workhorse of the plotting library. It stores a list of
//! [`Artist`] values (one per chart call), manages auto-scaling, tick generation,
//! and drives the full ten-step render pipeline that produces the final visual
//! output within a subplot rectangle.

use crate::annotations::{Annotation, ArrowStyle, TextAnnotation};
use crate::artist::*;
use crate::colorbar::{self, Colorbar};
use crate::error::{PlotError, Result};
use crate::layout::{self, LayoutConfig};
use crate::legend::{self, LegendEntry, SwatchKind};
use crate::primitives::*;
use crate::renderer::Renderer;
use crate::scale::Scale;
use crate::series::{IntoCategories, IntoSeries};
use crate::theme::{GridAxis, LineStyle, Loc, Marker, Theme, TickDirection};
use crate::ticks;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default number of ticks to aim for on each axis.
const DEFAULT_TICK_COUNT: usize = 7;

/// Padding fraction applied to auto-computed data limits so that data points
/// do not sit directly on the axis spines.
const AUTOSCALE_PAD: f64 = 0.05;

// ---------------------------------------------------------------------------
// TwinSide
// ---------------------------------------------------------------------------

/// Specifies which side a twin axes occupies relative to its parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwinSide {
    /// The twin shares the x-axis and draws an independent y-axis on the right.
    Right,
    /// The twin shares the y-axis and draws an independent x-axis on the top.
    Top,
}

// ---------------------------------------------------------------------------
// Axes
// ---------------------------------------------------------------------------

/// A single set of axes within a figure, containing chart artists and
/// configuration.
///
/// Users do not construct `Axes` directly; they are created by
/// [`Figure::add_subplot`] or the convenience function [`Figure::subplots`].
/// Once an `Axes` handle is obtained, chart methods such as [`plot`](Axes::plot),
/// [`scatter`](Axes::scatter), and [`bar`](Axes::bar) add data, and
/// configuration methods like [`set_title`](Axes::set_title) control labels,
/// limits, and styling.
#[derive(Debug)]
pub struct Axes {
    /// The list of artists (one per chart call) in draw order.
    pub(crate) artists: Vec<Artist>,
    /// Optional title displayed above the plot area.
    pub(crate) title: Option<String>,
    /// Optional label for the x-axis.
    pub(crate) xlabel: Option<String>,
    /// Optional label for the y-axis.
    pub(crate) ylabel: Option<String>,
    /// User-specified x-axis limits; `None` means auto-scale.
    pub(crate) xlim: Option<(f64, f64)>,
    /// User-specified y-axis limits; `None` means auto-scale.
    pub(crate) ylim: Option<(f64, f64)>,
    /// Scale for the x-axis (linear, log, symlog).
    pub(crate) xscale: Scale,
    /// Scale for the y-axis (linear, log, symlog).
    pub(crate) yscale: Scale,
    /// Whether to show grid lines. `None` defers to the theme default.
    pub(crate) show_grid: Option<bool>,
    /// Which axes display grid lines (x-only, y-only, or both).
    pub(crate) grid_axis: GridAxis,
    /// Grid line opacity override (0.0 = transparent, 1.0 = opaque). `None`
    /// means fully opaque.
    pub(crate) grid_alpha: Option<f64>,
    /// Grid line style override. `None` means use the theme default (solid).
    pub(crate) grid_style: Option<LineStyle>,
    /// Whether the x-axis is inverted (max on left, min on right).
    pub(crate) x_inverted: bool,
    /// Whether the y-axis is inverted (max on bottom, min on top).
    pub(crate) y_inverted: bool,
    /// Custom x-axis tick positions. `None` means auto-generate.
    pub(crate) custom_xticks: Option<Vec<f64>>,
    /// Custom y-axis tick positions. `None` means auto-generate.
    pub(crate) custom_yticks: Option<Vec<f64>>,
    /// Custom x-axis tick labels. Must match the custom tick count.
    pub(crate) custom_xticklabels: Option<Vec<String>>,
    /// Custom y-axis tick labels. Must match the custom tick count.
    pub(crate) custom_yticklabels: Option<Vec<String>>,
    /// Rotation angle in degrees for x-axis tick labels.
    pub(crate) xtick_rotation: f64,
    /// Rotation angle in degrees for y-axis tick labels.
    pub(crate) ytick_rotation: f64,
    /// Whether the legend should be drawn.
    pub(crate) show_legend: bool,
    /// Where to place the legend.
    pub(crate) legend_loc: Loc,
    /// Per-axes theme override. `None` means use the figure theme.
    pub(crate) theme_override: Option<Theme>,
    /// Text labels placed at data-space coordinates via [`Axes::text`].
    pub(crate) texts: Vec<TextAnnotation>,
    /// Annotations (text + optional arrow) placed via [`Axes::annotate`].
    pub(crate) annotations: Vec<Annotation>,
    /// Tracks the current position in the color cycle so that each successive
    /// artist receives a distinct color automatically.
    pub(crate) color_index: usize,
    /// Whether this axes is a twin of another axes.
    pub(crate) is_twin: bool,
    /// Which side this twin axes occupies.
    pub(crate) twin_side: Option<TwinSide>,
    /// Optional colorbar attached to this axes.
    pub(crate) colorbar: Option<Colorbar>,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

impl Axes {
    /// Creates a new, empty axes with default settings.
    pub(crate) fn new() -> Self {
        Self {
            artists: Vec::new(),
            title: None,
            xlabel: None,
            ylabel: None,
            xlim: None,
            ylim: None,
            xscale: Scale::default(),
            yscale: Scale::default(),
            show_grid: None,
            grid_axis: GridAxis::default(),
            grid_alpha: None,
            grid_style: None,
            x_inverted: false,
            y_inverted: false,
            custom_xticks: None,
            custom_yticks: None,
            custom_xticklabels: None,
            custom_yticklabels: None,
            xtick_rotation: 0.0,
            ytick_rotation: 0.0,
            show_legend: false,
            legend_loc: Loc::Best,
            theme_override: None,
            texts: Vec::new(),
            annotations: Vec::new(),
            color_index: 0,
            is_twin: false,
            twin_side: None,
            colorbar: None,
        }
    }

    /// Creates a new twin axes.
    pub(crate) fn new_twin(side: TwinSide, color_index: usize) -> Self {
        Self {
            is_twin: true,
            twin_side: Some(side),
            color_index,
            ..Self::new()
        }
    }

    /// Returns `true` if this axes is a twin of another axes.
    pub fn is_twin(&self) -> bool {
        self.is_twin
    }

    /// Returns the twin side, if this axes is a twin.
    pub fn twin_side(&self) -> Option<TwinSide> {
        self.twin_side
    }
}

// ---------------------------------------------------------------------------
// Chart methods
// ---------------------------------------------------------------------------

impl Axes {
    /// Plots a line connecting `(x, y)` data points.
    ///
    /// Returns a mutable reference to the newly created [`LineArtist`] so
    /// that the caller can chain builder methods (`.color()`, `.width()`,
    /// `.label()`, etc.).
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::SeriesLengthMismatch`] if `x` and `y` have
    /// different lengths, or [`PlotError::EmptyData`] if either is empty.
    pub fn plot<X, Y>(&mut self, x: X, y: Y) -> Result<&mut LineArtist>
    where
        X: IntoSeries,
        Y: IntoSeries,
    {
        let xs = x.into_series();
        let ys = y.into_series();
        if xs.len() != ys.len() {
            return Err(PlotError::SeriesLengthMismatch {
                expected: xs.len(),
                got: ys.len(),
            });
        }
        if xs.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = LineArtist {
            x: xs,
            y: ys,
            color,
            width: 1.5,
            style: crate::theme::LineStyle::Solid,
            label: None,
            alpha: 1.0,
            decimate: crate::decimate::DecimateMode::Auto,
        };
        self.artists.push(Artist::Line(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Line(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Creates a scatter plot of `(x, y)` data points.
    ///
    /// Returns a mutable reference to the [`ScatterArtist`] for chaining.
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::SeriesLengthMismatch`] or [`PlotError::EmptyData`]
    /// on invalid input.
    pub fn scatter<X, Y>(&mut self, x: X, y: Y) -> Result<&mut ScatterArtist>
    where
        X: IntoSeries,
        Y: IntoSeries,
    {
        let xs = x.into_series();
        let ys = y.into_series();
        if xs.len() != ys.len() {
            return Err(PlotError::SeriesLengthMismatch {
                expected: xs.len(),
                got: ys.len(),
            });
        }
        if xs.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = ScatterArtist {
            x: xs,
            y: ys,
            color,
            marker: Marker::Circle,
            size: 6.0,
            label: None,
            alpha: 0.8,
            colors: None,
            c: None,
            cmap: None,
            decimate: crate::decimate::DecimateMode::Auto,
        };
        self.artists.push(Artist::Scatter(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Scatter(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Creates a vertical bar chart from categorical data and heights.
    ///
    /// Each category label maps to one bar whose height is the corresponding
    /// value in `heights`.
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::SeriesLengthMismatch`] if `categories` and
    /// `heights` have different lengths, or [`PlotError::EmptyData`] if empty.
    pub fn bar<C, H>(&mut self, categories: C, heights: H) -> Result<&mut BarArtist>
    where
        C: IntoCategories,
        H: IntoSeries,
    {
        let cats = categories.into_categories();
        let vals = heights.into_series();
        if cats.len() != vals.len() {
            return Err(PlotError::SeriesLengthMismatch {
                expected: cats.len(),
                got: vals.len(),
            });
        }
        if cats.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = BarArtist {
            categories: cats,
            heights: vals,
            color,
            horizontal: false,
            bar_width: 0.8,
            label: None,
            alpha: 1.0,
            bottom: None,
            offset: None,
        };
        self.artists.push(Artist::Bar(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Bar(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Creates a horizontal bar chart from categorical data and widths.
    ///
    /// Behaves like [`bar`](Axes::bar) but draws horizontal bars from the
    /// y-axis.
    ///
    /// # Errors
    ///
    /// Same as [`bar`](Axes::bar).
    pub fn barh<C, W>(&mut self, categories: C, widths: W) -> Result<&mut BarArtist>
    where
        C: IntoCategories,
        W: IntoSeries,
    {
        let cats = categories.into_categories();
        let vals = widths.into_series();
        if cats.len() != vals.len() {
            return Err(PlotError::SeriesLengthMismatch {
                expected: cats.len(),
                got: vals.len(),
            });
        }
        if cats.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = BarArtist {
            categories: cats,
            heights: vals,
            color,
            horizontal: true,
            bar_width: 0.8,
            label: None,
            alpha: 1.0,
            bottom: None,
            offset: None,
        };
        self.artists.push(Artist::Bar(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Bar(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Creates a histogram from raw data.
    ///
    /// The data is partitioned into `bins` equal-width bins spanning the data
    /// range, and each bin is drawn as a vertical bar whose height equals the
    /// count of values falling in that bin.
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::EmptyData`] if `data` is empty.
    pub fn hist<D>(&mut self, data: D, bins: usize) -> Result<&mut HistArtist>
    where
        D: IntoSeries,
    {
        let series = data.into_series();
        if series.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let bins = bins.max(1);

        // Compute range from finite values.
        let (data_min, data_max) = series.bounds().unwrap_or((0.0, 1.0));

        // Handle degenerate case where all values are identical.
        let (lo, hi) = if (data_max - data_min).abs() < f64::EPSILON {
            (data_min - 0.5, data_max + 0.5)
        } else {
            (data_min, data_max)
        };

        let bin_width = (hi - lo) / bins as f64;

        // Build bin edges: bins+1 edges.
        let mut edges: Vec<f64> = (0..=bins).map(|i| lo + i as f64 * bin_width).collect();
        // Ensure the last edge exactly equals hi to avoid floating-point gaps.
        *edges.last_mut().expect("edges is non-empty") = hi;

        // Count values per bin.
        let mut counts = vec![0.0f64; bins];
        for &v in &series.data {
            if !v.is_finite() {
                continue;
            }
            // Determine bin index.
            let idx = if v >= hi {
                // Values exactly at the upper edge go into the last bin.
                bins - 1
            } else {
                let raw = ((v - lo) / bin_width) as usize;
                raw.min(bins - 1)
            };
            counts[idx] += 1.0;
        }

        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = HistArtist {
            data: series,
            bins,
            bin_edges: edges,
            counts,
            color,
            label: None,
            alpha: 0.85,
            density: false,
        };

        self.artists.push(Artist::Histogram(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Histogram(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Fills the area between two y-series that share the same x values.
    ///
    /// Useful for confidence intervals, envelopes, and area charts.
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::SeriesLengthMismatch`] if any of the three
    /// series differ in length, or [`PlotError::EmptyData`] if empty.
    pub fn fill_between<X, Y1, Y2>(
        &mut self,
        x: X,
        y1: Y1,
        y2: Y2,
    ) -> Result<&mut FillBetweenArtist>
    where
        X: IntoSeries,
        Y1: IntoSeries,
        Y2: IntoSeries,
    {
        let xs = x.into_series();
        let y1s = y1.into_series();
        let y2s = y2.into_series();
        if xs.len() != y1s.len() {
            return Err(PlotError::SeriesLengthMismatch {
                expected: xs.len(),
                got: y1s.len(),
            });
        }
        if xs.len() != y2s.len() {
            return Err(PlotError::SeriesLengthMismatch {
                expected: xs.len(),
                got: y2s.len(),
            });
        }
        if xs.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = FillBetweenArtist {
            x: xs,
            y1: y1s,
            y2: y2s,
            color,
            label: None,
            alpha: 0.3,
        };
        self.artists.push(Artist::FillBetween(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::FillBetween(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Plots a step (staircase) chart connecting `(x, y)` data points.
    ///
    /// Returns a mutable reference to the [`StepArtist`] for chaining.
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::SeriesLengthMismatch`] or [`PlotError::EmptyData`]
    /// on invalid input.
    pub fn step<X: IntoSeries, Y: IntoSeries>(&mut self, x: X, y: Y) -> Result<&mut StepArtist> {
        let xs = x.into_series();
        let ys = y.into_series();
        if xs.len() != ys.len() {
            return Err(PlotError::SeriesLengthMismatch {
                expected: xs.len(),
                got: ys.len(),
            });
        }
        if xs.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = StepArtist {
            x: xs,
            y: ys,
            color,
            width: 1.5,
            where_step: StepWhere::Pre,
            label: None,
            alpha: 1.0,
        };
        self.artists.push(Artist::Step(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Step(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Plots a stem (lollipop) chart from `(x, y)` data points.
    ///
    /// Returns a mutable reference to the [`StemArtist`] for chaining.
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::SeriesLengthMismatch`] or [`PlotError::EmptyData`]
    /// on invalid input.
    pub fn stem<X: IntoSeries, Y: IntoSeries>(&mut self, x: X, y: Y) -> Result<&mut StemArtist> {
        let xs = x.into_series();
        let ys = y.into_series();
        if xs.len() != ys.len() {
            return Err(PlotError::SeriesLengthMismatch {
                expected: xs.len(),
                got: ys.len(),
            });
        }
        if xs.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = StemArtist {
            x: xs,
            y: ys,
            color,
            line_width: 1.5,
            marker_size: 6.0,
            baseline: 0.0,
            label: None,
            alpha: 1.0,
        };
        self.artists.push(Artist::Stem(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Stem(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Creates a box-and-whisker plot from one or more data groups.
    ///
    /// Each inner `Vec<f64>` produces one box. Groups are placed at integer
    /// x-positions starting from 0.
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::EmptyData`] if `datasets` is empty.
    pub fn boxplot(&mut self, datasets: Vec<Vec<f64>>) -> Result<&mut BoxPlotArtist> {
        use crate::charts::boxplot::compute_stats;
        if datasets.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let factor = 1.5;
        let stats: Vec<_> = datasets.iter().map(|d| compute_stats(d, factor)).collect();
        let labels: Vec<String> = (0..datasets.len()).map(|i| format!("{}", i + 1)).collect();
        let artist = BoxPlotArtist {
            stats,
            labels,
            color,
            label: None,
            alpha: 1.0,
            box_width: 0.5,
            show_outliers: true,
            whisker_iq_factor: factor,
            raw_data: datasets,
        };
        self.artists.push(Artist::BoxPlot(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::BoxPlot(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Creates an error bar plot from `(x, y)` data points.
    ///
    /// Returns an owned [`ErrorBarArtist`] that can be configured with
    /// builder methods (`.yerr_symmetric()`, `.cap_size()`, etc.) and then
    /// added to the axes via [`add_errorbar`](Axes::add_errorbar).
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::SeriesLengthMismatch`] or [`PlotError::EmptyData`]
    /// on invalid input.
    pub fn errorbar<X: IntoSeries, Y: IntoSeries>(&mut self, x: X, y: Y) -> Result<ErrorBarArtist> {
        let xs = x.into_series();
        let ys = y.into_series();
        if xs.len() != ys.len() {
            return Err(PlotError::SeriesLengthMismatch {
                expected: xs.len(),
                got: ys.len(),
            });
        }
        if xs.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        Ok(ErrorBarArtist {
            x: xs,
            y: ys,
            xerr: None,
            yerr: None,
            color,
            label: None,
            cap_size: 4.0,
            line_width: 1.0,
        })
    }

    /// Adds a finalized [`ErrorBarArtist`] to the axes.
    ///
    /// Use this after configuring the artist returned by
    /// [`errorbar`](Axes::errorbar).
    pub fn add_errorbar(&mut self, artist: ErrorBarArtist) {
        self.artists.push(Artist::ErrorBar(artist));
    }

    /// Creates a heatmap from a 2D grid of values.
    ///
    /// Returns a mutable reference to the [`HeatmapArtist`] for chaining
    /// builder methods (`.colormap()`, `.vmin()`, `.show_values()`, etc.).
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::EmptyData`] if `data` is empty.
    pub fn heatmap(&mut self, data: Vec<Vec<f64>>) -> Result<&mut HeatmapArtist> {
        if data.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = HeatmapArtist {
            data,
            x_labels: None,
            y_labels: None,
            cmap: crate::colormap::Colormap::Viridis,
            vmin: None,
            vmax: None,
            show_values: false,
            color,
            label: None,
            show_colorbar: false,
        };
        self.artists.push(Artist::Heatmap(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Heatmap(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Creates a pie chart from wedge sizes.
    ///
    /// The `sizes` are automatically normalised so that all wedges sum to
    /// a full circle. Returns a mutable reference to the [`PieArtist`]
    /// for chaining builder methods (`.labels()`, `.autopct()`,
    /// `.explode()`, etc.).
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::EmptyData`] if `sizes` is empty.
    pub fn pie<S: IntoSeries>(&mut self, sizes: S) -> Result<&mut PieArtist> {
        let series = sizes.into_series();
        if series.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = PieArtist {
            sizes: series.data,
            labels: None,
            colors: None,
            explode: None,
            autopct: false,
            start_angle: 90.0,
            radius: 1.0,
            label: None,
            color,
        };
        self.artists.push(Artist::Pie(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Pie(a) => Ok(a),
            _ => unreachable!(),
        }
    }
}

// ---------------------------------------------------------------------------
// Configuration methods (builder-style, return &mut Self)
// ---------------------------------------------------------------------------

impl Axes {
    /// Sets the title displayed above the axes area.
    ///
    /// Supports `^`/`_` super/subscript markup — see [`crate::text`].
    pub fn set_title(&mut self, title: &str) -> &mut Self {
        self.title = Some(crate::text::format_markup(title));
        self
    }

    /// Sets the x-axis label. Supports `^`/`_` markup — see [`crate::text`].
    pub fn set_xlabel(&mut self, label: &str) -> &mut Self {
        self.xlabel = Some(crate::text::format_markup(label));
        self
    }

    /// Sets the y-axis label. Supports `^`/`_` markup — see [`crate::text`].
    pub fn set_ylabel(&mut self, label: &str) -> &mut Self {
        self.ylabel = Some(crate::text::format_markup(label));
        self
    }

    /// Sets explicit x-axis limits. Pass `(min, max)`.
    pub fn set_xlim(&mut self, min: f64, max: f64) -> &mut Self {
        self.xlim = Some((min, max));
        self
    }

    /// Sets explicit y-axis limits. Pass `(min, max)`.
    pub fn set_ylim(&mut self, min: f64, max: f64) -> &mut Self {
        self.ylim = Some((min, max));
        self
    }

    /// Sets the x-axis scale (linear, log10, symlog).
    pub fn set_xscale(&mut self, scale: Scale) -> &mut Self {
        self.xscale = scale;
        self
    }

    /// Sets the y-axis scale (linear, log10, symlog).
    pub fn set_yscale(&mut self, scale: Scale) -> &mut Self {
        self.yscale = scale;
        self
    }

    /// Enables or disables grid lines on this axes.
    pub fn grid(&mut self, show: bool) -> &mut Self {
        self.show_grid = Some(show);
        self
    }

    /// Enables the legend on this axes.
    pub fn legend(&mut self) -> &mut Self {
        self.show_legend = true;
        self
    }

    /// Sets the legend location.
    pub fn set_legend_loc(&mut self, loc: Loc) -> &mut Self {
        self.legend_loc = loc;
        self
    }

    /// Overrides the figure theme for this axes only.
    pub fn set_theme(&mut self, theme: Theme) -> &mut Self {
        self.theme_override = Some(theme);
        self
    }

    /// Sets which axes display grid lines.
    ///
    /// Accepts `"x"`, `"y"`, or `"both"` (the default). Unrecognised values
    /// are silently treated as `"both"`.
    pub fn grid_axis(&mut self, axis: &str) -> &mut Self {
        self.grid_axis = match axis {
            "x" => GridAxis::X,
            "y" => GridAxis::Y,
            _ => GridAxis::Both,
        };
        self
    }

    /// Sets the grid line opacity (0.0 = fully transparent, 1.0 = fully opaque).
    pub fn grid_alpha(&mut self, alpha: f64) -> &mut Self {
        self.grid_alpha = Some(alpha.clamp(0.0, 1.0));
        self
    }

    /// Sets the grid line style.
    pub fn grid_style(&mut self, style: LineStyle) -> &mut Self {
        self.grid_style = Some(style);
        self
    }

    /// Inverts the x-axis so that larger values appear on the left.
    pub fn invert_xaxis(&mut self) -> &mut Self {
        self.x_inverted = true;
        self
    }

    /// Inverts the y-axis so that larger values appear at the bottom.
    pub fn invert_yaxis(&mut self) -> &mut Self {
        self.y_inverted = true;
        self
    }

    /// Sets custom tick positions on the x-axis.
    ///
    /// When set, the auto-generated ticks are replaced by these positions.
    pub fn set_xticks(&mut self, ticks: &[f64]) -> &mut Self {
        self.custom_xticks = Some(ticks.to_vec());
        self
    }

    /// Sets custom tick positions on the y-axis.
    ///
    /// When set, the auto-generated ticks are replaced by these positions.
    pub fn set_yticks(&mut self, ticks: &[f64]) -> &mut Self {
        self.custom_yticks = Some(ticks.to_vec());
        self
    }

    /// Sets custom tick labels for the x-axis.
    ///
    /// The number of labels should match the number of custom tick positions
    /// set via [`set_xticks`](Axes::set_xticks). Extra labels are ignored;
    /// missing labels default to the formatted tick value.
    pub fn set_xticklabels(&mut self, labels: &[&str]) -> &mut Self {
        self.custom_xticklabels = Some(labels.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Sets custom tick labels for the y-axis.
    ///
    /// The number of labels should match the number of custom tick positions
    /// set via [`set_yticks`](Axes::set_yticks). Extra labels are ignored;
    /// missing labels default to the formatted tick value.
    pub fn set_yticklabels(&mut self, labels: &[&str]) -> &mut Self {
        self.custom_yticklabels = Some(labels.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Sets the rotation angle (in degrees) for x-axis tick labels.
    pub fn tick_params_x_rotation(&mut self, degrees: f64) -> &mut Self {
        self.xtick_rotation = degrees;
        self
    }

    /// Sets the rotation angle (in degrees) for y-axis tick labels.
    pub fn tick_params_y_rotation(&mut self, degrees: f64) -> &mut Self {
        self.ytick_rotation = degrees;
        self
    }

    /// Places a text label at data coordinates `(x, y)`.
    ///
    /// Returns a mutable reference to the [`TextAnnotation`] for builder-style
    /// configuration (`.fontsize()`, `.color()`, `.ha()`, `.va()`, `.rotation()`).
    ///
    /// Text annotations do **not** affect autoscale; they annotate existing
    /// data without expanding the axis limits.
    pub fn text(&mut self, x: f64, y: f64, text: &str) -> &mut TextAnnotation {
        self.texts.push(TextAnnotation {
            text: crate::text::format_markup(text),
            x,
            y,
            fontsize: None,
            color: None,
            ha: HAlign::Left,
            va: VAlign::Baseline,
            rotation: 0.0,
        });
        self.texts.last_mut().expect("just pushed")
    }

    /// Annotates the data point `xy` with `text` placed at `xytext`.
    ///
    /// Returns a mutable reference to the [`Annotation`] for builder-style
    /// configuration (`.fontsize()`, `.color()`, `.ha()`, `.va()`,
    /// `.arrowstyle()`, `.arrow_color()`).
    ///
    /// When an [`ArrowStyle`] other than [`ArrowStyle::None`] is set, an arrow
    /// is drawn from `xytext` to `xy`. Annotations do **not** affect autoscale.
    pub fn annotate(&mut self, text: &str, xy: (f64, f64), xytext: (f64, f64)) -> &mut Annotation {
        self.annotations.push(Annotation {
            text: crate::text::format_markup(text),
            xy,
            xytext,
            fontsize: None,
            color: None,
            ha: HAlign::Center,
            va: VAlign::Bottom,
            arrowstyle: ArrowStyle::None,
            arrow_color: None,
        });
        self.annotations.last_mut().expect("just pushed")
    }
}

// ---------------------------------------------------------------------------
// Colorbar
// ---------------------------------------------------------------------------

impl Axes {
    /// Adds a colorbar to this axes.
    pub fn colorbar(
        &mut self,
        cmap: crate::colormap::Colormap,
        vmin: f64,
        vmax: f64,
    ) -> &mut Colorbar {
        self.colorbar = Some(Colorbar::new(cmap, vmin, vmax));
        self.colorbar.as_mut().expect("just set")
    }
}

// ---------------------------------------------------------------------------
// Contour, Violin, Bar group
// ---------------------------------------------------------------------------

impl Axes {
    /// Creates a contour plot (iso-lines) from a 2D grid of z values.
    pub fn contour(
        &mut self,
        x: &[f64],
        y: &[f64],
        z: Vec<Vec<f64>>,
    ) -> Result<&mut ContourArtist> {
        if x.is_empty() || y.is_empty() || z.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = ContourArtist {
            x: x.to_vec(),
            y: y.to_vec(),
            z,
            levels: None,
            filled: false,
            cmap: crate::colormap::Colormap::Viridis,
            colors: None,
            linewidths: 1.0,
            label: None,
            color,
            num_levels: 10,
        };
        self.artists.push(Artist::Contour(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Contour(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Creates a filled contour plot from a 2D grid of z values.
    pub fn contourf(
        &mut self,
        x: &[f64],
        y: &[f64],
        z: Vec<Vec<f64>>,
    ) -> Result<&mut ContourArtist> {
        if x.is_empty() || y.is_empty() || z.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = ContourArtist {
            x: x.to_vec(),
            y: y.to_vec(),
            z,
            levels: None,
            filled: true,
            cmap: crate::colormap::Colormap::Viridis,
            colors: None,
            linewidths: 1.0,
            label: None,
            color,
            num_levels: 10,
        };
        self.artists.push(Artist::Contour(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Contour(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Creates a violin plot from one or more datasets.
    pub fn violin(&mut self, datasets: Vec<Vec<f64>>) -> Result<&mut ViolinArtist> {
        if datasets.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = ViolinArtist {
            datasets,
            positions: None,
            widths: 0.7,
            show_median: true,
            show_quartiles: true,
            color,
            alpha: 0.7,
            label: None,
            bw_method: 0.0,
        };
        self.artists.push(Artist::Violin(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Violin(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Creates grouped (side-by-side) bars for multiple series.
    ///
    /// `series` is a slice of `(label, heights)` pairs. Each series gets
    /// bars placed side-by-side within each category.
    pub fn bar_group<C: IntoCategories>(
        &mut self,
        categories: C,
        series: &[(&str, Vec<f64>)],
    ) -> Result<()> {
        let cat_labels: Vec<String> = categories
            .into_categories()
            .labels
            .iter()
            .map(|s| s.to_string())
            .collect();
        let n_series = series.len();
        if n_series == 0 {
            return Err(PlotError::EmptyData);
        }
        let total_width = 0.8;
        let bar_width = total_width / n_series as f64;

        for (si, (label, heights)) in series.iter().enumerate() {
            let offset_val = (si as f64 - (n_series as f64 - 1.0) / 2.0) * bar_width;
            let offsets: Vec<f64> = vec![offset_val; heights.len()];
            let artist_ref = self.bar(cat_labels.clone(), heights.as_slice())?;
            artist_ref.bar_width(bar_width);
            artist_ref.offset(offsets);
            artist_ref.label(label);
        }
        Ok(())
    }

    /// Creates a hexagonal binning plot from `(x, y)` data points.
    ///
    /// Returns a mutable reference to the [`HexbinArtist`] for chaining
    /// builder methods (`.gridsize()`, `.colormap()`, `.mincnt()`, etc.).
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::EmptyData`] if `x` or `y` is empty.
    /// Returns [`PlotError::SeriesLengthMismatch`] if `x` and `y` differ in length.
    pub fn hexbin<X, Y>(&mut self, x: X, y: Y) -> Result<&mut HexbinArtist>
    where
        X: IntoSeries,
        Y: IntoSeries,
    {
        let xs = x.into_series();
        let ys = y.into_series();
        if xs.is_empty() || ys.is_empty() {
            return Err(PlotError::EmptyData);
        }
        if xs.len() != ys.len() {
            return Err(PlotError::SeriesLengthMismatch {
                expected: xs.len(),
                got: ys.len(),
            });
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = HexbinArtist {
            x: xs.data,
            y: ys.data,
            gridsize: 20,
            cmap: crate::colormap::Colormap::Viridis,
            mincnt: 1,
            alpha: 1.0,
            color,
            label: None,
            edgecolor: None,
            show_colorbar: false,
        };
        self.artists.push(Artist::Hexbin(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Hexbin(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Creates a polar line plot from `(theta, r)` data in polar coordinates.
    ///
    /// `theta` contains angles in radians, `r` contains the corresponding
    /// radial distances. The data is drawn as a connected polyline in polar
    /// space.
    ///
    /// Returns a mutable reference to the [`PolarArtist`] for chaining.
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::SeriesLengthMismatch`] if `theta` and `r` have
    /// different lengths, or [`PlotError::EmptyData`] if either is empty.
    pub fn polar_plot<T, R>(&mut self, theta: T, r: R) -> Result<&mut PolarArtist>
    where
        T: IntoSeries,
        R: IntoSeries,
    {
        let ts = theta.into_series();
        let rs = r.into_series();
        if ts.len() != rs.len() {
            return Err(PlotError::SeriesLengthMismatch {
                expected: ts.len(),
                got: rs.len(),
            });
        }
        if ts.is_empty() {
            return Err(PlotError::EmptyData);
        }
        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = PolarArtist {
            theta: ts.data,
            r: rs.data,
            color,
            label: None,
            alpha: 1.0,
            linewidth: 1.5,
            filled: false,
            marker: None,
        };
        self.artists.push(Artist::Polar(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Polar(a) => Ok(a),
            _ => unreachable!(),
        }
    }

    /// Creates a filled polar (radar/area) chart from `(theta, r)` data.
    ///
    /// Like [`polar_plot`](Axes::polar_plot), but the path is automatically
    /// closed and filled, producing a radar or area chart.
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::SeriesLengthMismatch`] if `theta` and `r` have
    /// different lengths, or [`PlotError::EmptyData`] if either is empty.
    pub fn polar_fill<T, R>(&mut self, theta: T, r: R) -> Result<&mut PolarArtist>
    where
        T: IntoSeries,
        R: IntoSeries,
    {
        let artist_ref = self.polar_plot(theta, r)?;
        artist_ref.filled(true);
        artist_ref.alpha(0.3);
        Ok(artist_ref)
    }

    /// Creates a waterfall chart from categorical data and change values.
    ///
    /// Each bar represents an incremental change (positive or negative) from
    /// the previous running total. Positive changes are colored green,
    /// negative changes red, and bars marked as totals are colored blue-gray.
    ///
    /// The method also sets the x-axis tick positions and labels to match the
    /// category labels so they render on the axis automatically.
    ///
    /// # Errors
    ///
    /// Returns [`PlotError::SeriesLengthMismatch`] if `categories` and
    /// `values` have different lengths, or [`PlotError::EmptyData`] if empty.
    pub fn waterfall<C, V>(&mut self, categories: C, values: V) -> Result<&mut WaterfallArtist>
    where
        C: IntoCategories,
        V: IntoSeries,
    {
        let cats = categories.into_categories();
        let vals = values.into_series();
        if cats.len() != vals.len() {
            return Err(PlotError::SeriesLengthMismatch {
                expected: cats.len(),
                got: vals.len(),
            });
        }
        if cats.is_empty() {
            return Err(PlotError::EmptyData);
        }

        // Set x-axis ticks to category positions with category labels.
        let n = cats.len();
        let tick_positions: Vec<f64> = (0..n).map(|i| i as f64).collect();
        self.custom_xticks = Some(tick_positions);
        self.custom_xticklabels = Some(cats.labels.iter().map(|s| s.to_string()).collect());

        let color = Color::TABLEAU_10[self.color_index % 10];
        self.color_index += 1;
        let artist = WaterfallArtist {
            categories: cats,
            values: vals,
            total_indices: Vec::new(),
            increase_color: Color::rgb(0x2C, 0xA0, 0x2C), // Professional green
            decrease_color: Color::rgb(0xD6, 0x27, 0x28), // Professional red
            total_color: Color::rgb(0x4E, 0x79, 0xA7),    // Tableau blue-gray
            connector_lines: true,
            show_values: false,
            bar_width: 0.6,
            label: None,
            color,
            alpha: 1.0,
        };
        self.artists.push(Artist::Waterfall(artist));
        match self.artists.last_mut().expect("just pushed") {
            Artist::Waterfall(a) => Ok(a),
            _ => unreachable!(),
        }
    }
}

// ---------------------------------------------------------------------------
// Rendering pipeline
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
impl Axes {
    /// Renders this axes into the given rectangle on the renderer.
    ///
    /// This is called by the `Figure` during its render pass. The ten-step
    /// pipeline is:
    ///
    /// 1. Compute data limits (autoscale or user-set).
    /// 2. Generate tick positions and labels.
    /// 3. Compute internal layout (margins for title, labels, ticks).
    /// 4. Draw axes background.
    /// 5. Draw grid lines (behind data).
    /// 6. Clip to the plot area and draw each artist.
    /// 7. Draw axis spines.
    /// 8. Draw ticks and tick labels.
    /// 9. Draw axis labels and title.
    /// 10. Draw legend (if enabled).
    pub(crate) fn render(&self, renderer: &mut impl Renderer, bounds: Rect, fig_theme: &Theme) {
        self.render_primary(renderer, bounds, fig_theme, false);
    }

    pub(crate) fn render_primary(
        &self,
        renderer: &mut impl Renderer,
        bounds: Rect,
        fig_theme: &Theme,
        suppress_legend: bool,
    ) {
        let theme = self.theme_override.as_ref().unwrap_or(fig_theme);

        // Step 1: Compute data limits.
        let (xmin, xmax, ymin, ymax) = self.compute_data_limits();

        // Step 2: Generate ticks (use custom ticks if provided).
        let xticks = self.resolve_xticks(xmin, xmax);
        let yticks = self.resolve_yticks(ymin, ymax);

        // Step 3: Compute layout (reserve space for labels, ticks, title).
        let mut layout_config = LayoutConfig::new(bounds.width, bounds.height);
        layout_config.has_title = self.title.is_some();
        layout_config.has_xlabel = self.xlabel.is_some();
        layout_config.has_ylabel = self.ylabel.is_some();
        layout_config.has_legend = self.show_legend;

        let layout_result = layout::compute_layout(&layout_config);

        let full_plot_area = Rect::new(
            bounds.x + layout_result.plot_area.x,
            bounds.y + layout_result.plot_area.y,
            layout_result.plot_area.width,
            layout_result.plot_area.height,
        );

        // Resolve the effective colorbar: either an explicit one or auto from heatmap.
        let effective_colorbar = if self.colorbar.is_some() {
            self.colorbar.clone()
        } else {
            self.auto_colorbar_from_artists()
        };

        // When a colorbar is present, shrink the plot area to make room.
        let (plot_area, colorbar_rect) = if effective_colorbar.is_some() {
            let cb_width = (full_plot_area.width * colorbar::COLORBAR_WIDTH_FRACTION).max(30.0);
            let shrunk = Rect::new(
                full_plot_area.x,
                full_plot_area.y,
                full_plot_area.width - cb_width - colorbar::COLORBAR_GAP,
                full_plot_area.height,
            );
            let cb_rect = Rect::new(
                full_plot_area.x + full_plot_area.width - cb_width,
                full_plot_area.y,
                cb_width,
                full_plot_area.height,
            );
            (shrunk, Some(cb_rect))
        } else {
            (full_plot_area, None)
        };

        // Step 4: Draw axes background.
        let bg_path = Path::rect(plot_area);
        renderer.fill_path(
            &bg_path,
            &Paint::new(theme.axes_background),
            Affine::IDENTITY,
        );

        // Step 5: Draw grid (behind data).
        if self.show_grid.unwrap_or(theme.show_grid) {
            self.draw_grid(
                renderer, &plot_area, &xticks, &yticks, xmin, xmax, ymin, ymax, theme,
            );
        }

        // Step 6: Clip to plot area and draw each artist.
        let clip_path = Path::rect(plot_area);
        renderer.push_clip(&clip_path, Affine::IDENTITY);
        for artist in &self.artists {
            self.draw_artist(renderer, artist, &plot_area, xmin, xmax, ymin, ymax, theme);
        }
        renderer.pop_clip();

        // Step 6b: Draw minor ticks for log scales (behind spines, after data).
        let x_minor = if matches!(self.xscale, Scale::Log10) {
            ticks::generate_log_minor_ticks(xmin, xmax)
        } else {
            Vec::new()
        };
        let y_minor = if matches!(self.yscale, Scale::Log10) {
            ticks::generate_log_minor_ticks(ymin, ymax)
        } else {
            Vec::new()
        };

        // Step 7: Draw spines.
        self.draw_spines(renderer, &plot_area, theme);

        // Step 8: Draw ticks and tick labels (including minor ticks).
        self.draw_ticks(
            renderer, &plot_area, &xticks, &yticks, xmin, xmax, ymin, ymax, theme,
        );
        if !x_minor.is_empty() || !y_minor.is_empty() {
            self.draw_minor_ticks(
                renderer, &plot_area, &x_minor, &y_minor, xmin, xmax, ymin, ymax, theme,
            );
        }

        // Step 9: Draw axis labels and title.
        self.draw_labels(renderer, &plot_area, &bounds, theme);

        // Step 9b: Draw text annotations and arrow annotations.
        self.draw_annotations(renderer, &plot_area, xmin, xmax, ymin, ymax, theme);

        // Step 10: Draw legend if enabled.
        if self.show_legend && !suppress_legend {
            self.draw_legend(renderer, &plot_area, theme);
        }

        // Draw colorbar if present.
        if let (Some(ref cb), Some(ref cb_rect)) = (&effective_colorbar, &colorbar_rect) {
            colorbar::draw_colorbar(renderer, cb, cb_rect, theme);
        }
    }

    fn auto_colorbar_from_artists(&self) -> Option<Colorbar> {
        for artist in &self.artists {
            match artist {
                Artist::Heatmap(a) if a.show_colorbar => {
                    return Some(Colorbar::new(
                        a.cmap,
                        a.effective_vmin(),
                        a.effective_vmax(),
                    ));
                }
                Artist::Hexbin(a) if a.show_colorbar => {
                    let result =
                        crate::charts::hexbin::bin_hexagonal(&a.x, &a.y, a.gridsize, a.mincnt);
                    let vmin = result.min_count as f64;
                    let vmax = (result.max_count as f64).max(vmin + 1.0);
                    return Some(Colorbar::new(a.cmap, vmin, vmax));
                }
                _ => {}
            }
        }
        None
    }

    /// Renders this twin axes overlaid on its parent's plot area.
    pub(crate) fn render_twin(
        &self,
        renderer: &mut impl Renderer,
        plot_area: Rect,
        bounds: Rect,
        fig_theme: &Theme,
    ) {
        let theme = self.theme_override.as_ref().unwrap_or(fig_theme);
        let (xmin, xmax, ymin, ymax) = self.compute_data_limits();
        let yticks = self.resolve_yticks(ymin, ymax);
        let xticks = self.resolve_xticks(xmin, xmax);

        let clip_path = Path::rect(plot_area);
        renderer.push_clip(&clip_path, Affine::IDENTITY);
        for artist in &self.artists {
            self.draw_artist(renderer, artist, &plot_area, xmin, xmax, ymin, ymax, theme);
        }
        renderer.pop_clip();

        let side = self.twin_side.unwrap_or(TwinSide::Right);
        match side {
            TwinSide::Right => {
                let paint = Paint::new(theme.spine_color);
                let stroke = Stroke::new(theme.spine_width);
                let mut p = Path::new();
                p.move_to(plot_area.right(), plot_area.y);
                p.line_to(plot_area.right(), plot_area.bottom());
                renderer.stroke_path(&p, &paint, &stroke, Affine::IDENTITY);
                self.draw_ticks_right(renderer, &plot_area, &yticks, ymin, ymax, theme);
                self.draw_ylabel_right(renderer, &plot_area, &bounds, theme);
            }
            TwinSide::Top => {
                let paint = Paint::new(theme.spine_color);
                let stroke = Stroke::new(theme.spine_width);
                let mut p = Path::new();
                p.move_to(plot_area.x, plot_area.y);
                p.line_to(plot_area.right(), plot_area.y);
                renderer.stroke_path(&p, &paint, &stroke, Affine::IDENTITY);
                self.draw_ticks_top(renderer, &plot_area, &xticks, xmin, xmax, theme);
                self.draw_xlabel_top(renderer, &plot_area, theme);
            }
        }

        self.draw_annotations(renderer, &plot_area, xmin, xmax, ymin, ymax, theme);
    }

    /// Returns the computed plot area for this axes given a bounds rectangle.
    pub(crate) fn compute_plot_area(&self, bounds: &Rect) -> Rect {
        let mut layout_config = LayoutConfig::new(bounds.width, bounds.height);
        layout_config.has_title = self.title.is_some();
        layout_config.has_xlabel = self.xlabel.is_some();
        layout_config.has_ylabel = self.ylabel.is_some();
        layout_config.has_legend = self.show_legend;
        let layout_result = layout::compute_layout(&layout_config);
        Rect::new(
            bounds.x + layout_result.plot_area.x,
            bounds.y + layout_result.plot_area.y,
            layout_result.plot_area.width,
            layout_result.plot_area.height,
        )
    }

    /// Collects legend entries from this axes' artists.
    pub fn collect_legend_entries(&self) -> Vec<LegendEntry> {
        self.artists
            .iter()
            .filter_map(|a| {
                let (label, color, swatch) = match a {
                    Artist::Line(a) => (a.label.as_deref(), a.color, SwatchKind::Line),
                    Artist::Scatter(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::Bar(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::Histogram(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::FillBetween(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::Step(a) => (a.label.as_deref(), a.color, SwatchKind::Line),
                    Artist::Stem(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::BoxPlot(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::ErrorBar(a) => (a.label.as_deref(), a.color, SwatchKind::Line),
                    Artist::Heatmap(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::Pie(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::Violin(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::Contour(a) => (
                        a.label.as_deref(),
                        a.color,
                        if a.filled {
                            SwatchKind::Filled
                        } else {
                            SwatchKind::Line
                        },
                    ),
                    Artist::Polar(a) => (
                        a.label.as_deref(),
                        a.color,
                        if a.filled {
                            SwatchKind::Filled
                        } else {
                            SwatchKind::Line
                        },
                    ),
                    Artist::Hexbin(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::Waterfall(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                };
                label.map(|l| LegendEntry {
                    label: l.to_string(),
                    color,
                    swatch,
                })
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Step 1: Data limits
    // -----------------------------------------------------------------------

    /// Computes the data limits from all artists, applying user overrides
    /// and padding. Returns `(xmin, xmax, ymin, ymax)`.
    fn compute_data_limits(&self) -> (f64, f64, f64, f64) {
        let mut x_lo = f64::INFINITY;
        let mut x_hi = f64::NEG_INFINITY;
        let mut y_lo = f64::INFINITY;
        let mut y_hi = f64::NEG_INFINITY;

        for artist in &self.artists {
            match artist {
                Artist::Line(a) => {
                    if let Some((lo, hi)) = a.x.bounds() {
                        x_lo = x_lo.min(lo);
                        x_hi = x_hi.max(hi);
                    }
                    if let Some((lo, hi)) = a.y.bounds() {
                        y_lo = y_lo.min(lo);
                        y_hi = y_hi.max(hi);
                    }
                }
                Artist::Scatter(a) => {
                    if let Some((lo, hi)) = a.x.bounds() {
                        x_lo = x_lo.min(lo);
                        x_hi = x_hi.max(hi);
                    }
                    if let Some((lo, hi)) = a.y.bounds() {
                        y_lo = y_lo.min(lo);
                        y_hi = y_hi.max(hi);
                    }
                }
                Artist::Bar(a) => {
                    let n = a.categories.len() as f64;
                    if a.horizontal {
                        // x-axis is the value axis, y-axis is the category axis.
                        y_lo = 0.0_f64.min(y_lo);
                        y_hi = n.max(y_hi);
                        x_lo = 0.0_f64.min(x_lo);
                        if let Some((lo, hi)) = a.heights.bounds() {
                            x_lo = x_lo.min(lo.min(0.0));
                            x_hi = x_hi.max(hi);
                        }
                    } else {
                        // x-axis is the category axis, y-axis is the value axis.
                        x_lo = 0.0_f64.min(x_lo);
                        x_hi = n.max(x_hi);
                        y_lo = 0.0_f64.min(y_lo);
                        if let Some((lo, hi)) = a.heights.bounds() {
                            y_lo = y_lo.min(lo.min(0.0));
                            y_hi = y_hi.max(hi);
                        }
                    }
                }
                Artist::Histogram(a) => {
                    if let (Some(&first), Some(&last)) = (a.bin_edges.first(), a.bin_edges.last()) {
                        x_lo = x_lo.min(first);
                        x_hi = x_hi.max(last);
                    }
                    y_lo = 0.0_f64.min(y_lo);
                    let max_count = a.counts.iter().fold(0.0f64, |a, &b| a.max(b));
                    y_hi = y_hi.max(max_count);
                }
                Artist::FillBetween(a) => {
                    if let Some((lo, hi)) = a.x.bounds() {
                        x_lo = x_lo.min(lo);
                        x_hi = x_hi.max(hi);
                    }
                    if let Some((lo, hi)) = a.y1.bounds() {
                        y_lo = y_lo.min(lo);
                        y_hi = y_hi.max(hi);
                    }
                    if let Some((lo, hi)) = a.y2.bounds() {
                        y_lo = y_lo.min(lo);
                        y_hi = y_hi.max(hi);
                    }
                }
                Artist::Step(a) => {
                    if let Some((lo, hi)) = a.x.bounds() {
                        x_lo = x_lo.min(lo);
                        x_hi = x_hi.max(hi);
                    }
                    if let Some((lo, hi)) = a.y.bounds() {
                        y_lo = y_lo.min(lo);
                        y_hi = y_hi.max(hi);
                    }
                }
                Artist::Stem(a) => {
                    if let Some((lo, hi)) = a.x.bounds() {
                        x_lo = x_lo.min(lo);
                        x_hi = x_hi.max(hi);
                    }
                    if let Some((lo, hi)) = a.y.bounds() {
                        y_lo = y_lo.min(lo.min(a.baseline));
                        y_hi = y_hi.max(hi.max(a.baseline));
                    }
                }
                Artist::BoxPlot(a) => {
                    let n = a.stats.len() as f64;
                    x_lo = 0.0_f64.min(x_lo);
                    x_hi = n.max(x_hi);
                    for s in &a.stats {
                        y_lo = y_lo.min(s.whisker_low);
                        y_hi = y_hi.max(s.whisker_high);
                        for &o in &s.outliers {
                            y_lo = y_lo.min(o);
                            y_hi = y_hi.max(o);
                        }
                    }
                }
                Artist::ErrorBar(a) => {
                    let (bxlo, bxhi, bylo, byhi) = a.data_bounds();
                    x_lo = x_lo.min(bxlo);
                    x_hi = x_hi.max(bxhi);
                    y_lo = y_lo.min(bylo);
                    y_hi = y_hi.max(byhi);
                }
                Artist::Heatmap(a) => {
                    let (bxlo, bxhi, bylo, byhi) = a.data_bounds();
                    x_lo = x_lo.min(bxlo);
                    x_hi = x_hi.max(bxhi);
                    y_lo = y_lo.min(bylo);
                    y_hi = y_hi.max(byhi);
                }
                Artist::Pie(a) => {
                    // Pie charts define their own coordinate space and
                    // should not participate in normal autoscaling.
                    let (bxlo, bxhi, bylo, byhi) = a.data_bounds();
                    x_lo = x_lo.min(bxlo);
                    x_hi = x_hi.max(bxhi);
                    y_lo = y_lo.min(bylo);
                    y_hi = y_hi.max(byhi);
                }
                Artist::Violin(a) => {
                    let (bxlo, bxhi, bylo, byhi) = a.data_bounds();
                    x_lo = x_lo.min(bxlo);
                    x_hi = x_hi.max(bxhi);
                    y_lo = y_lo.min(bylo);
                    y_hi = y_hi.max(byhi);
                }
                Artist::Contour(a) => {
                    let (bxlo, bxhi, bylo, byhi) = a.data_bounds();
                    x_lo = x_lo.min(bxlo);
                    x_hi = x_hi.max(bxhi);
                    y_lo = y_lo.min(bylo);
                    y_hi = y_hi.max(byhi);
                }
                Artist::Polar(a) => {
                    let (bxlo, bxhi, bylo, byhi) = a.data_bounds();
                    x_lo = x_lo.min(bxlo);
                    x_hi = x_hi.max(bxhi);
                    y_lo = y_lo.min(bylo);
                    y_hi = y_hi.max(byhi);
                }
                Artist::Hexbin(a) => {
                    let (bxlo, bxhi, bylo, byhi) = a.data_bounds();
                    x_lo = x_lo.min(bxlo);
                    x_hi = x_hi.max(bxhi);
                    y_lo = y_lo.min(bylo);
                    y_hi = y_hi.max(byhi);
                }
                Artist::Waterfall(a) => {
                    let (bxlo, bxhi, bylo, byhi) = a.data_bounds();
                    x_lo = x_lo.min(bxlo);
                    x_hi = x_hi.max(bxhi);
                    y_lo = y_lo.min(bylo);
                    y_hi = y_hi.max(byhi);
                }
            }
        }

        // Handle the case where there are no artists or no finite data.
        if !x_lo.is_finite() || !x_hi.is_finite() {
            x_lo = if self.xscale.requires_positive() {
                1.0
            } else {
                0.0
            };
            x_hi = if self.xscale.requires_positive() {
                10.0
            } else {
                1.0
            };
        }
        if !y_lo.is_finite() || !y_hi.is_finite() {
            y_lo = if self.yscale.requires_positive() {
                1.0
            } else {
                0.0
            };
            y_hi = if self.yscale.requires_positive() {
                10.0
            } else {
                1.0
            };
        }

        // For log scales, clamp the lower bound to a positive value.
        if self.xscale.requires_positive() {
            if x_lo <= 0.0 {
                // Use a small fraction of x_hi, or fall back to a sensible default.
                x_lo = if x_hi > 0.0 { x_hi * 1e-4 } else { 1.0 };
            }
            if x_hi <= x_lo {
                x_hi = x_lo * 10.0;
            }
        }
        if self.yscale.requires_positive() {
            if y_lo <= 0.0 {
                y_lo = if y_hi > 0.0 { y_hi * 1e-4 } else { 1.0 };
            }
            if y_hi <= y_lo {
                y_hi = y_lo * 10.0;
            }
        }

        // Handle degenerate ranges (all data at a single point).
        if (x_hi - x_lo).abs() < f64::EPSILON {
            x_lo -= 0.5;
            x_hi += 0.5;
        }
        if (y_hi - y_lo).abs() < f64::EPSILON {
            y_lo -= 0.5;
            y_hi += 0.5;
        }

        // Apply padding (5% on each side).
        // For log scales, apply multiplicative padding instead of additive.
        let (x_pad_lo, x_pad_hi) = if self.xscale.requires_positive() {
            // Multiplicative padding: shrink lo, grow hi by a factor.
            let factor = 1.0 + AUTOSCALE_PAD;
            (x_lo / factor, x_hi * factor)
        } else {
            let pad = (x_hi - x_lo) * AUTOSCALE_PAD;
            (x_lo - pad, x_hi + pad)
        };
        let (y_pad_lo, y_pad_hi) = if self.yscale.requires_positive() {
            let factor = 1.0 + AUTOSCALE_PAD;
            (y_lo / factor, y_hi * factor)
        } else {
            let pad = (y_hi - y_lo) * AUTOSCALE_PAD;
            (y_lo - pad, y_hi + pad)
        };
        x_lo = x_pad_lo;
        x_hi = x_pad_hi;
        y_lo = y_pad_lo;
        y_hi = y_pad_hi;

        // Apply user-set limits, overriding auto-scale.
        if let Some((lo, hi)) = self.xlim {
            x_lo = lo;
            x_hi = hi;
        }
        if let Some((lo, hi)) = self.ylim {
            y_lo = lo;
            y_hi = hi;
        }

        // Apply axis inversion by swapping min/max.
        if self.x_inverted {
            std::mem::swap(&mut x_lo, &mut x_hi);
        }
        if self.y_inverted {
            std::mem::swap(&mut y_lo, &mut y_hi);
        }

        (x_lo, x_hi, y_lo, y_hi)
    }

    // -----------------------------------------------------------------------
    // Step 2 helpers: tick resolution
    // -----------------------------------------------------------------------

    /// Resolves x-axis ticks: uses custom ticks when set, falls back to
    /// auto-generation.
    fn resolve_xticks(&self, xmin: f64, xmax: f64) -> Vec<ticks::Tick> {
        if let Some(ref positions) = self.custom_xticks {
            let labels = self.custom_xticklabels.as_ref();
            positions
                .iter()
                .enumerate()
                .map(|(i, &v)| ticks::Tick {
                    value: v,
                    label: labels
                        .and_then(|l| l.get(i))
                        .cloned()
                        .unwrap_or_else(|| ticks::format_tick_value(v)),
                })
                .collect()
        } else {
            let (lo, hi) = if xmin <= xmax {
                (xmin, xmax)
            } else {
                (xmax, xmin)
            };
            ticks::generate_ticks(lo, hi, DEFAULT_TICK_COUNT, &self.xscale)
        }
    }

    /// Resolves y-axis ticks: uses custom ticks when set, falls back to
    /// auto-generation.
    fn resolve_yticks(&self, ymin: f64, ymax: f64) -> Vec<ticks::Tick> {
        if let Some(ref positions) = self.custom_yticks {
            let labels = self.custom_yticklabels.as_ref();
            positions
                .iter()
                .enumerate()
                .map(|(i, &v)| ticks::Tick {
                    value: v,
                    label: labels
                        .and_then(|l| l.get(i))
                        .cloned()
                        .unwrap_or_else(|| ticks::format_tick_value(v)),
                })
                .collect()
        } else {
            let (lo, hi) = if ymin <= ymax {
                (ymin, ymax)
            } else {
                (ymax, ymin)
            };
            ticks::generate_ticks(lo, hi, DEFAULT_TICK_COUNT, &self.yscale)
        }
    }

    // -----------------------------------------------------------------------
    // Step 5: Grid
    // -----------------------------------------------------------------------

    /// Draws major grid lines behind the data.
    fn draw_grid(
        &self,
        renderer: &mut impl Renderer,
        plot_area: &Rect,
        xticks: &[ticks::Tick],
        yticks: &[ticks::Tick],
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
        theme: &Theme,
    ) {
        // Apply grid alpha override.
        let grid_color = if let Some(alpha) = self.grid_alpha {
            theme.grid_color.with_alpha((alpha * 255.0) as u8)
        } else {
            theme.grid_color
        };
        let paint = Paint::new(grid_color);

        // Apply grid line style override.
        let mut stroke = Stroke::new(theme.grid_width);
        if let Some(style) = self.grid_style {
            stroke = match style {
                LineStyle::Solid => stroke,
                LineStyle::Dashed => stroke.with_dash(DashPattern {
                    dashes: vec![6.0, 4.0],
                    offset: 0.0,
                }),
                LineStyle::Dotted => stroke.with_dash(DashPattern {
                    dashes: vec![2.0, 2.0],
                    offset: 0.0,
                }),
                LineStyle::DashDot => stroke.with_dash(DashPattern {
                    dashes: vec![6.0, 3.0, 2.0, 3.0],
                    offset: 0.0,
                }),
            };
        }

        let draw_x = matches!(self.grid_axis, GridAxis::X | GridAxis::Both);
        let draw_y = matches!(self.grid_axis, GridAxis::Y | GridAxis::Both);

        // Vertical grid lines at each x-tick.
        if draw_x {
            for tick in xticks {
                let pt = self.data_to_pixel(tick.value, ymin, plot_area, xmin, xmax, ymin, ymax);
                let mut path = Path::new();
                path.move_to(pt.x, plot_area.y);
                path.line_to(pt.x, plot_area.bottom());
                renderer.stroke_path(&path, &paint, &stroke, Affine::IDENTITY);
            }
        }

        // Horizontal grid lines at each y-tick.
        if draw_y {
            for tick in yticks {
                let pt = self.data_to_pixel(xmin, tick.value, plot_area, xmin, xmax, ymin, ymax);
                let mut path = Path::new();
                path.move_to(plot_area.x, pt.y);
                path.line_to(plot_area.right(), pt.y);
                renderer.stroke_path(&path, &paint, &stroke, Affine::IDENTITY);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Step 6: Artist drawing
    // -----------------------------------------------------------------------

    /// Dispatches drawing to the appropriate artist-type-specific method.
    fn draw_artist(
        &self,
        renderer: &mut impl Renderer,
        artist: &Artist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
        theme: &Theme,
    ) {
        match artist {
            Artist::Line(a) => self.draw_line(renderer, a, plot_area, xmin, xmax, ymin, ymax),
            Artist::Scatter(a) => {
                self.draw_scatter(renderer, a, plot_area, xmin, xmax, ymin, ymax, theme)
            }
            Artist::Bar(a) => self.draw_bar(renderer, a, plot_area, xmin, xmax, ymin, ymax),
            Artist::Histogram(a) => self.draw_hist(renderer, a, plot_area, xmin, xmax, ymin, ymax),
            Artist::FillBetween(a) => {
                self.draw_fill_between(renderer, a, plot_area, xmin, xmax, ymin, ymax)
            }
            Artist::Step(a) => self.draw_step(renderer, a, plot_area, xmin, xmax, ymin, ymax),
            Artist::Stem(a) => self.draw_stem(renderer, a, plot_area, xmin, xmax, ymin, ymax),
            Artist::BoxPlot(a) => self.draw_boxplot(renderer, a, plot_area, xmin, xmax, ymin, ymax),
            Artist::ErrorBar(a) => {
                self.draw_errorbar(renderer, a, plot_area, xmin, xmax, ymin, ymax)
            }
            Artist::Heatmap(a) => self.draw_heatmap(renderer, a, plot_area, xmin, xmax, ymin, ymax),
            Artist::Pie(a) => self.draw_pie(renderer, a, plot_area, xmin, xmax, ymin, ymax, theme),
            Artist::Violin(a) => {
                self.draw_violin(renderer, a, plot_area, xmin, xmax, ymin, ymax, theme)
            }
            Artist::Contour(a) => self.draw_contour(renderer, a, plot_area, xmin, xmax, ymin, ymax),
            Artist::Polar(a) => {
                self.draw_polar(renderer, a, plot_area, xmin, xmax, ymin, ymax, theme)
            }
            Artist::Hexbin(a) => self.draw_hexbin(renderer, a, plot_area, xmin, xmax, ymin, ymax),
            Artist::Waterfall(a) => {
                self.draw_waterfall(renderer, a, plot_area, xmin, xmax, ymin, ymax)
            }
        }
    }

    /// Draws a line chart: builds a polyline from data points and strokes it.
    ///
    /// When the artist has decimation enabled and the data exceeds the
    /// threshold, the point set is downsampled before path construction.
    fn draw_line(
        &self,
        renderer: &mut impl Renderer,
        artist: &LineArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) {
        if artist.x.is_empty() {
            return;
        }

        // Compute the set of indices to draw (possibly decimated). The mode
        // (auto / off / explicit) is resolved by a single pure function so the
        // result is deterministic for a given input.
        let indices: Vec<usize> = artist
            .decimate
            .resolve_indices(&artist.x.data, &artist.y.data);

        if indices.is_empty() {
            return;
        }

        let mut path = Path::new();
        let first = self.data_to_pixel(
            artist.x.data[indices[0]],
            artist.y.data[indices[0]],
            plot_area,
            xmin,
            xmax,
            ymin,
            ymax,
        );
        path.move_to(first.x, first.y);

        for &i in &indices[1..] {
            let pt = self.data_to_pixel(
                artist.x.data[i],
                artist.y.data[i],
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );
            path.line_to(pt.x, pt.y);
        }

        let color = artist.color.with_alpha((artist.alpha * 255.0) as u8);
        let paint = Paint::new(color);
        let mut stroke = Stroke::new(artist.width);

        // Apply line style dash pattern.
        match artist.style {
            crate::theme::LineStyle::Solid => {}
            crate::theme::LineStyle::Dashed => {
                stroke = stroke.with_dash(DashPattern {
                    dashes: vec![6.0, 4.0],
                    offset: 0.0,
                });
            }
            crate::theme::LineStyle::Dotted => {
                stroke = stroke.with_dash(DashPattern {
                    dashes: vec![2.0, 2.0],
                    offset: 0.0,
                });
            }
            crate::theme::LineStyle::DashDot => {
                stroke = stroke.with_dash(DashPattern {
                    dashes: vec![6.0, 3.0, 2.0, 3.0],
                    offset: 0.0,
                });
            }
        }

        renderer.stroke_path(&path, &paint, &stroke, Affine::IDENTITY);
    }

    /// Draws a scatter plot: fills a marker shape at each data point.
    fn draw_scatter(
        &self,
        renderer: &mut impl Renderer,
        artist: &ScatterArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
        theme: &Theme,
    ) {
        let alpha_byte = (artist.alpha * 255.0) as u8;

        // Pre-compute per-point colors from c/cmap if set.
        let cmap_colors: Option<Vec<Color>> = match (&artist.c, &artist.cmap) {
            (Some(c_vals), Some(cmap)) if !c_vals.is_empty() => Some(cmap.map_values(c_vals)),
            _ => None,
        };

        let default_color = artist.color.with_alpha(alpha_byte);
        let default_paint = Paint::new(default_color);
        let radius = artist.size / 2.0;

        // Resolve the set of point indices to draw (possibly decimated). The
        // mode (auto / off / explicit) is resolved by a single pure function,
        // and per-point styling is indexed by the original index so colors stay
        // synchronized with their points after decimation.
        let indices = artist
            .decimate
            .resolve_indices(&artist.x.data, &artist.y.data);

        for &i in &indices {
            let pt = self.data_to_pixel(
                artist.x.data[i],
                artist.y.data[i],
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );

            // Resolve the paint for this point: c/cmap > colors > color.
            let paint = if let Some(ref cc) = cmap_colors {
                Paint::new(cc[i].with_alpha(alpha_byte))
            } else if let Some(ref cs) = artist.colors {
                Paint::new(cs[i].with_alpha(alpha_byte))
            } else {
                default_paint
            };

            let marker_path = match artist.marker {
                Marker::Circle | Marker::Point => Path::circle(pt, radius),
                Marker::Square => Path::rect(Rect::new(
                    pt.x - radius,
                    pt.y - radius,
                    radius * 2.0,
                    radius * 2.0,
                )),
                Marker::Diamond => {
                    let mut p = Path::new();
                    p.move_to(pt.x, pt.y - radius);
                    p.line_to(pt.x + radius, pt.y);
                    p.line_to(pt.x, pt.y + radius);
                    p.line_to(pt.x - radius, pt.y);
                    p.close();
                    p
                }
                Marker::Triangle => {
                    let mut p = Path::new();
                    let h = radius * 1.1547; // 2/sqrt(3) for equilateral
                    p.move_to(pt.x, pt.y - radius);
                    p.line_to(pt.x + h * 0.5, pt.y + radius * 0.5);
                    p.line_to(pt.x - h * 0.5, pt.y + radius * 0.5);
                    p.close();
                    p
                }
                Marker::Plus => {
                    // Stroked marker: draw two perpendicular lines.
                    let mut p = Path::new();
                    p.move_to(pt.x - radius, pt.y);
                    p.line_to(pt.x + radius, pt.y);
                    p.move_to(pt.x, pt.y - radius);
                    p.line_to(pt.x, pt.y + radius);
                    let stroke = Stroke::new(theme.line_width.max(1.0));
                    renderer.stroke_path(&p, &paint, &stroke, Affine::IDENTITY);
                    continue;
                }
                Marker::Cross => {
                    let mut p = Path::new();
                    let d = radius * 0.707; // radius / sqrt(2)
                    p.move_to(pt.x - d, pt.y - d);
                    p.line_to(pt.x + d, pt.y + d);
                    p.move_to(pt.x + d, pt.y - d);
                    p.line_to(pt.x - d, pt.y + d);
                    let stroke = Stroke::new(theme.line_width.max(1.0));
                    renderer.stroke_path(&p, &paint, &stroke, Affine::IDENTITY);
                    continue;
                }
                Marker::Star => {
                    // 5-pointed star.
                    let mut p = Path::new();
                    let inner = radius * 0.382;
                    for j in 0..10 {
                        let angle =
                            std::f64::consts::FRAC_PI_2 + j as f64 * std::f64::consts::PI / 5.0;
                        let r = if j % 2 == 0 { radius } else { inner };
                        let sx = pt.x + r * angle.cos();
                        let sy = pt.y - r * angle.sin();
                        if j == 0 {
                            p.move_to(sx, sy);
                        } else {
                            p.line_to(sx, sy);
                        }
                    }
                    p.close();
                    p
                }
            };

            renderer.fill_path(&marker_path, &paint, Affine::IDENTITY);
        }
    }

    /// Draws a bar chart: fills rectangles for each category.
    fn draw_bar(
        &self,
        renderer: &mut impl Renderer,
        artist: &BarArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) {
        let n = artist.categories.len();
        let color = artist.color.with_alpha((artist.alpha * 255.0) as u8);
        let paint = Paint::new(color);

        if artist.horizontal {
            // Horizontal bars: categories on y-axis, values on x-axis.
            let cat_range = ymax - ymin;
            let cat_step = cat_range / n as f64;
            let bar_half = cat_step * artist.bar_width * 0.5;

            for i in 0..n {
                let base_center = ymin + (i as f64 + 0.5) * cat_step;
                let cat_center = if let Some(ref off) = artist.offset {
                    base_center + if i < off.len() { off[i] } else { 0.0 }
                } else {
                    base_center
                };
                let value = artist.heights.data[i];
                let base = if let Some(ref bot) = artist.bottom {
                    if i < bot.len() {
                        bot[i]
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                let left_val = base.min(base + value);
                let right_val = base.max(base + value);

                let p_left = self.data_to_pixel(
                    left_val,
                    cat_center - bar_half,
                    plot_area,
                    xmin,
                    xmax,
                    ymin,
                    ymax,
                );
                let p_right = self.data_to_pixel(
                    right_val,
                    cat_center + bar_half,
                    plot_area,
                    xmin,
                    xmax,
                    ymin,
                    ymax,
                );

                let rect = Rect::from_points(p_left, p_right);
                let bar_path = Path::rect(rect);
                renderer.fill_path(&bar_path, &paint, Affine::IDENTITY);
            }
        } else {
            // Vertical bars: categories on x-axis, values on y-axis.
            let cat_range = xmax - xmin;
            let cat_step = cat_range / n as f64;
            let bar_half = cat_step * artist.bar_width * 0.5;

            for i in 0..n {
                let base_center = xmin + (i as f64 + 0.5) * cat_step;
                let cat_center = if let Some(ref off) = artist.offset {
                    base_center + if i < off.len() { off[i] } else { 0.0 }
                } else {
                    base_center
                };
                let value = artist.heights.data[i];
                let base = if let Some(ref bot) = artist.bottom {
                    if i < bot.len() {
                        bot[i]
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                let bottom_val = base.min(base + value);
                let top_val = base.max(base + value);

                let p_bl = self.data_to_pixel(
                    cat_center - bar_half,
                    bottom_val,
                    plot_area,
                    xmin,
                    xmax,
                    ymin,
                    ymax,
                );
                let p_tr = self.data_to_pixel(
                    cat_center + bar_half,
                    top_val,
                    plot_area,
                    xmin,
                    xmax,
                    ymin,
                    ymax,
                );

                let rect = Rect::from_points(p_bl, p_tr);
                let bar_path = Path::rect(rect);
                renderer.fill_path(&bar_path, &paint, Affine::IDENTITY);
            }
        }
    }

    /// Draws a histogram: fills rectangles from bin edges and counts.
    fn draw_hist(
        &self,
        renderer: &mut impl Renderer,
        artist: &HistArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) {
        let color = artist.color.with_alpha((artist.alpha * 255.0) as u8);
        let paint = Paint::new(color);
        let stroke_paint = Paint::new(Color::WHITE);
        let stroke = Stroke::new(0.5);

        for i in 0..artist.counts.len() {
            let left = artist.bin_edges[i];
            let right = artist.bin_edges[i + 1];
            let height = artist.counts[i];

            if height <= 0.0 {
                continue;
            }

            let p_bl = self.data_to_pixel(left, 0.0, plot_area, xmin, xmax, ymin, ymax);
            let p_tr = self.data_to_pixel(right, height, plot_area, xmin, xmax, ymin, ymax);

            let rect = Rect::from_points(p_bl, p_tr);
            let bar_path = Path::rect(rect);
            renderer.fill_path(&bar_path, &paint, Affine::IDENTITY);
            // Thin white outline between adjacent bins for visual separation.
            renderer.stroke_path(&bar_path, &stroke_paint, &stroke, Affine::IDENTITY);
        }
    }

    /// Draws a fill-between region: builds a closed path from y1 forward and
    /// y2 backward, then fills it.
    fn draw_fill_between(
        &self,
        renderer: &mut impl Renderer,
        artist: &FillBetweenArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) {
        if artist.x.is_empty() {
            return;
        }

        let n = artist.x.len();
        let mut path = Path::new();

        // Forward pass along y1.
        let first = self.data_to_pixel(
            artist.x.data[0],
            artist.y1.data[0],
            plot_area,
            xmin,
            xmax,
            ymin,
            ymax,
        );
        path.move_to(first.x, first.y);
        for i in 1..n {
            let pt = self.data_to_pixel(
                artist.x.data[i],
                artist.y1.data[i],
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );
            path.line_to(pt.x, pt.y);
        }

        // Backward pass along y2 (in reverse order).
        for i in (0..n).rev() {
            let pt = self.data_to_pixel(
                artist.x.data[i],
                artist.y2.data[i],
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );
            path.line_to(pt.x, pt.y);
        }
        path.close();

        let color = artist.color.with_alpha((artist.alpha * 255.0) as u8);
        let paint = Paint::new(color);
        renderer.fill_path(&path, &paint, Affine::IDENTITY);
    }

    // -----------------------------------------------------------------------
    // Step 7: Spines
    // -----------------------------------------------------------------------

    /// Draws the axis spines (border lines around the plot area).
    fn draw_spines(&self, renderer: &mut impl Renderer, plot_area: &Rect, theme: &Theme) {
        let paint = Paint::new(theme.spine_color);
        let stroke = Stroke::new(theme.spine_width);

        // Bottom spine.
        if theme.show_bottom_spine {
            let mut p = Path::new();
            p.move_to(plot_area.x, plot_area.bottom());
            p.line_to(plot_area.right(), plot_area.bottom());
            renderer.stroke_path(&p, &paint, &stroke, Affine::IDENTITY);
        }
        // Left spine.
        if theme.show_left_spine {
            let mut p = Path::new();
            p.move_to(plot_area.x, plot_area.y);
            p.line_to(plot_area.x, plot_area.bottom());
            renderer.stroke_path(&p, &paint, &stroke, Affine::IDENTITY);
        }
        // Top spine.
        if theme.show_top_spine {
            let mut p = Path::new();
            p.move_to(plot_area.x, plot_area.y);
            p.line_to(plot_area.right(), plot_area.y);
            renderer.stroke_path(&p, &paint, &stroke, Affine::IDENTITY);
        }
        // Right spine.
        if theme.show_right_spine {
            let mut p = Path::new();
            p.move_to(plot_area.right(), plot_area.y);
            p.line_to(plot_area.right(), plot_area.bottom());
            renderer.stroke_path(&p, &paint, &stroke, Affine::IDENTITY);
        }
    }

    // -----------------------------------------------------------------------
    // Step 8: Ticks and tick labels
    // -----------------------------------------------------------------------

    /// Draws tick marks and their labels along both axes.
    fn draw_ticks(
        &self,
        renderer: &mut impl Renderer,
        plot_area: &Rect,
        xticks: &[ticks::Tick],
        yticks: &[ticks::Tick],
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
        theme: &Theme,
    ) {
        let tick_paint = Paint::new(theme.tick_color);
        let tick_stroke = Stroke::new(1.0);
        let tick_len = theme.tick_length;

        let x_label_style = TextStyle {
            size: theme.tick_label_size,
            color: theme.text_color,
            weight: FontWeight::Normal,
            family: theme.font_family.clone(),
            halign: if self.xtick_rotation.abs() > 1.0 {
                HAlign::Right
            } else {
                HAlign::Center
            },
            valign: VAlign::Top,
        };

        // Tick direction offset: outward goes away from plot, inward goes in.
        let outward = matches!(theme.tick_direction, TickDirection::Outward);

        // Pre-compute x-tick rotation transform builder.
        let x_rot_rad = -self.xtick_rotation.to_radians();

        // --- X-axis ticks (bottom) ---
        for tick in xticks {
            let pt = self.data_to_pixel(tick.value, ymin, plot_area, xmin, xmax, ymin, ymax);
            // Clamp to plot area x-bounds.
            if pt.x < plot_area.x - 0.5 || pt.x > plot_area.right() + 0.5 {
                continue;
            }
            let x = pt.x;
            let y_base = plot_area.bottom();

            // Draw tick mark.
            let (y_start, y_end) = if outward {
                (y_base, y_base + tick_len)
            } else {
                (y_base - tick_len, y_base)
            };
            let mut tp = Path::new();
            tp.move_to(x, y_start);
            tp.line_to(x, y_end);
            renderer.stroke_path(&tp, &tick_paint, &tick_stroke, Affine::IDENTITY);

            // Draw tick label with optional rotation.
            let label_y = if outward {
                y_base + tick_len + 2.0
            } else {
                y_base + 2.0
            };
            let label_pos = Point::new(x, label_y);
            let transform = if self.xtick_rotation.abs() > 0.01 {
                let rotate = Affine::rotate(x_rot_rad);
                let to_origin = Affine::translate(kurbo::Vec2::new(-label_pos.x, -label_pos.y));
                let from_origin = Affine::translate(kurbo::Vec2::new(label_pos.x, label_pos.y));
                from_origin * rotate * to_origin
            } else {
                Affine::IDENTITY
            };
            renderer.draw_text(&tick.label, label_pos, &x_label_style, transform);
        }

        // --- Y-axis ticks (left) ---
        let y_label_style = TextStyle {
            halign: HAlign::Right,
            valign: VAlign::Middle,
            ..x_label_style.clone()
        };

        // Pre-compute y-tick rotation transform builder.
        let y_rot_rad = -self.ytick_rotation.to_radians();

        for tick in yticks {
            let pt = self.data_to_pixel(xmin, tick.value, plot_area, xmin, xmax, ymin, ymax);
            // Clamp to plot area y-bounds.
            if pt.y < plot_area.y - 0.5 || pt.y > plot_area.bottom() + 0.5 {
                continue;
            }
            let y = pt.y;
            let x_base = plot_area.x;

            // Draw tick mark.
            let (x_start, x_end) = if outward {
                (x_base - tick_len, x_base)
            } else {
                (x_base, x_base + tick_len)
            };
            let mut tp = Path::new();
            tp.move_to(x_start, y);
            tp.line_to(x_end, y);
            renderer.stroke_path(&tp, &tick_paint, &tick_stroke, Affine::IDENTITY);

            // Draw tick label with optional rotation.
            let label_x = if outward {
                x_base - tick_len - 3.0
            } else {
                x_base - 3.0
            };
            let label_pos = Point::new(label_x, y);
            let transform = if self.ytick_rotation.abs() > 0.01 {
                let rotate = Affine::rotate(y_rot_rad);
                let to_origin = Affine::translate(kurbo::Vec2::new(-label_pos.x, -label_pos.y));
                let from_origin = Affine::translate(kurbo::Vec2::new(label_pos.x, label_pos.y));
                from_origin * rotate * to_origin
            } else {
                Affine::IDENTITY
            };
            renderer.draw_text(&tick.label, label_pos, &y_label_style, transform);
        }
    }

    /// Draws minor tick marks (without labels) along axes.
    ///
    /// Minor ticks are drawn shorter than major ticks and have no text labels.
    /// They are used primarily on log-scale axes to indicate sub-decade positions
    /// (2, 3, 4, 5, 6, 7, 8, 9 between each power of 10).
    fn draw_minor_ticks(
        &self,
        renderer: &mut impl Renderer,
        plot_area: &Rect,
        x_minor: &[f64],
        y_minor: &[f64],
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
        theme: &Theme,
    ) {
        let tick_paint = Paint::new(theme.tick_color);
        let tick_stroke = Stroke::new(0.5);
        // Minor ticks are half the length of major ticks.
        let tick_len = theme.tick_length * 0.5;
        let outward = matches!(theme.tick_direction, TickDirection::Outward);

        // --- X-axis minor ticks (bottom) ---
        for &val in x_minor {
            let pt = self.data_to_pixel(val, ymin, plot_area, xmin, xmax, ymin, ymax);
            if pt.x < plot_area.x - 0.5 || pt.x > plot_area.right() + 0.5 {
                continue;
            }
            let x = pt.x;
            let y_base = plot_area.bottom();
            let (y_start, y_end) = if outward {
                (y_base, y_base + tick_len)
            } else {
                (y_base - tick_len, y_base)
            };
            let mut tp = Path::new();
            tp.move_to(x, y_start);
            tp.line_to(x, y_end);
            renderer.stroke_path(&tp, &tick_paint, &tick_stroke, Affine::IDENTITY);
        }

        // --- Y-axis minor ticks (left) ---
        for &val in y_minor {
            let pt = self.data_to_pixel(xmin, val, plot_area, xmin, xmax, ymin, ymax);
            if pt.y < plot_area.y - 0.5 || pt.y > plot_area.bottom() + 0.5 {
                continue;
            }
            let y = pt.y;
            let x_base = plot_area.x;
            let (x_start, x_end) = if outward {
                (x_base - tick_len, x_base)
            } else {
                (x_base, x_base + tick_len)
            };
            let mut tp = Path::new();
            tp.move_to(x_start, y);
            tp.line_to(x_end, y);
            renderer.stroke_path(&tp, &tick_paint, &tick_stroke, Affine::IDENTITY);
        }
    }

    /// Draws y-axis tick marks and labels on the right side.
    fn draw_ticks_right(
        &self,
        renderer: &mut impl Renderer,
        plot_area: &Rect,
        yticks: &[ticks::Tick],
        ymin: f64,
        ymax: f64,
        theme: &Theme,
    ) {
        let tick_paint = Paint::new(theme.tick_color);
        let tick_stroke = Stroke::new(1.0);
        let tick_len = theme.tick_length;
        let outward = matches!(theme.tick_direction, TickDirection::Outward);

        let y_label_style = TextStyle {
            size: theme.tick_label_size,
            color: theme.text_color,
            weight: FontWeight::Normal,
            family: theme.font_family.clone(),
            halign: HAlign::Left,
            valign: VAlign::Middle,
        };

        let y_rot_rad = -self.ytick_rotation.to_radians();

        for tick in yticks {
            let pt = self.data_to_pixel(0.0, tick.value, plot_area, 0.0, 1.0, ymin, ymax);
            if pt.y < plot_area.y - 0.5 || pt.y > plot_area.bottom() + 0.5 {
                continue;
            }
            let y = pt.y;
            let x_base = plot_area.right();
            let (x_start, x_end) = if outward {
                (x_base, x_base + tick_len)
            } else {
                (x_base - tick_len, x_base)
            };
            let mut tp = Path::new();
            tp.move_to(x_start, y);
            tp.line_to(x_end, y);
            renderer.stroke_path(&tp, &tick_paint, &tick_stroke, Affine::IDENTITY);

            let label_x = if outward {
                x_base + tick_len + 3.0
            } else {
                x_base + 3.0
            };
            let label_pos = Point::new(label_x, y);
            let transform = if self.ytick_rotation.abs() > 0.01 {
                let rotate = Affine::rotate(y_rot_rad);
                let to_origin = Affine::translate(kurbo::Vec2::new(-label_pos.x, -label_pos.y));
                let from_origin = Affine::translate(kurbo::Vec2::new(label_pos.x, label_pos.y));
                from_origin * rotate * to_origin
            } else {
                Affine::IDENTITY
            };
            renderer.draw_text(&tick.label, label_pos, &y_label_style, transform);
        }
    }

    /// Draws the y-axis label on the right side, rotated 90 degrees clockwise.
    fn draw_ylabel_right(
        &self,
        renderer: &mut impl Renderer,
        plot_area: &Rect,
        bounds: &Rect,
        theme: &Theme,
    ) {
        if let Some(ylabel) = &self.ylabel {
            let style = TextStyle {
                size: theme.axis_label_size,
                color: theme.text_color,
                weight: FontWeight::Normal,
                family: theme.font_family.clone(),
                halign: HAlign::Center,
                valign: VAlign::Top,
            };
            let x = bounds.right() - 4.0;
            let y = plot_area.y + plot_area.height / 2.0;
            let rotate = Affine::rotate(std::f64::consts::FRAC_PI_2);
            let translate_to = Affine::translate(kurbo::Vec2::new(x, y));
            let translate_back = Affine::translate(kurbo::Vec2::new(-x, -y));
            let transform = translate_to * rotate * translate_back;
            renderer.draw_text(ylabel, Point::new(x, y), &style, transform);
        }
    }

    /// Draws x-axis tick marks and labels on the top of the plot area.
    fn draw_ticks_top(
        &self,
        renderer: &mut impl Renderer,
        plot_area: &Rect,
        xticks: &[ticks::Tick],
        xmin: f64,
        xmax: f64,
        theme: &Theme,
    ) {
        let tick_paint = Paint::new(theme.tick_color);
        let tick_stroke = Stroke::new(1.0);
        let tick_len = theme.tick_length;
        let outward = matches!(theme.tick_direction, TickDirection::Outward);

        let x_label_style = TextStyle {
            size: theme.tick_label_size,
            color: theme.text_color,
            weight: FontWeight::Normal,
            family: theme.font_family.clone(),
            halign: if self.xtick_rotation.abs() > 1.0 {
                HAlign::Left
            } else {
                HAlign::Center
            },
            valign: VAlign::Bottom,
        };

        let x_rot_rad = -self.xtick_rotation.to_radians();

        for tick in xticks {
            let pt = self.data_to_pixel(tick.value, 0.0, plot_area, xmin, xmax, 0.0, 1.0);
            if pt.x < plot_area.x - 0.5 || pt.x > plot_area.right() + 0.5 {
                continue;
            }
            let x = pt.x;
            let y_base = plot_area.y;
            let (y_start, y_end) = if outward {
                (y_base - tick_len, y_base)
            } else {
                (y_base, y_base + tick_len)
            };
            let mut tp = Path::new();
            tp.move_to(x, y_start);
            tp.line_to(x, y_end);
            renderer.stroke_path(&tp, &tick_paint, &tick_stroke, Affine::IDENTITY);

            let label_y = if outward {
                y_base - tick_len - 2.0
            } else {
                y_base - 2.0
            };
            let label_pos = Point::new(x, label_y);
            let transform = if self.xtick_rotation.abs() > 0.01 {
                let rotate = Affine::rotate(x_rot_rad);
                let to_origin = Affine::translate(kurbo::Vec2::new(-label_pos.x, -label_pos.y));
                let from_origin = Affine::translate(kurbo::Vec2::new(label_pos.x, label_pos.y));
                from_origin * rotate * to_origin
            } else {
                Affine::IDENTITY
            };
            renderer.draw_text(&tick.label, label_pos, &x_label_style, transform);
        }
    }

    /// Draws the x-axis label on the top side.
    fn draw_xlabel_top(&self, renderer: &mut impl Renderer, plot_area: &Rect, theme: &Theme) {
        if let Some(xlabel) = &self.xlabel {
            let style = TextStyle {
                size: theme.axis_label_size,
                color: theme.text_color,
                weight: FontWeight::Normal,
                family: theme.font_family.clone(),
                halign: HAlign::Center,
                valign: VAlign::Bottom,
            };
            let x = plot_area.x + plot_area.width / 2.0;
            let y = plot_area.y - theme.tick_length - theme.tick_label_size - 8.0;
            renderer.draw_text(xlabel, Point::new(x, y), &style, Affine::IDENTITY);
        }
    }

    // -----------------------------------------------------------------------
    // Step 9: Labels and title
    // -----------------------------------------------------------------------

    /// Draws the x-axis label, y-axis label, and title.
    fn draw_labels(
        &self,
        renderer: &mut impl Renderer,
        plot_area: &Rect,
        bounds: &Rect,
        theme: &Theme,
    ) {
        // Title (centered above plot area).
        if let Some(title) = &self.title {
            let style = TextStyle {
                size: theme.title_size,
                color: theme.text_color,
                weight: theme.title_weight,
                family: theme.font_family.clone(),
                halign: HAlign::Center,
                valign: VAlign::Bottom,
            };
            let x = plot_area.x + plot_area.width / 2.0;
            let y = plot_area.y - 10.0;
            renderer.draw_text(title, Point::new(x, y), &style, Affine::IDENTITY);
        }

        // X-axis label (centered below tick labels).
        if let Some(xlabel) = &self.xlabel {
            let style = TextStyle {
                size: theme.axis_label_size,
                color: theme.text_color,
                weight: FontWeight::Normal,
                family: theme.font_family.clone(),
                halign: HAlign::Center,
                valign: VAlign::Top,
            };
            let x = plot_area.x + plot_area.width / 2.0;
            // Position below the tick labels. Approximate tick label height.
            let y = plot_area.bottom() + theme.tick_length + theme.tick_label_size + 8.0;
            renderer.draw_text(xlabel, Point::new(x, y), &style, Affine::IDENTITY);
        }

        // Y-axis label (centered to the left of tick labels, rotated 90 degrees).
        if let Some(ylabel) = &self.ylabel {
            let style = TextStyle {
                size: theme.axis_label_size,
                color: theme.text_color,
                weight: FontWeight::Normal,
                family: theme.font_family.clone(),
                halign: HAlign::Center,
                valign: VAlign::Bottom,
            };
            let x = bounds.x + 4.0;
            let y = plot_area.y + plot_area.height / 2.0;
            // Rotate -90 degrees around the label position for vertical text.
            let rotate = Affine::rotate(-std::f64::consts::FRAC_PI_2);
            let translate_to = Affine::translate(kurbo::Vec2::new(x, y));
            let translate_back = Affine::translate(kurbo::Vec2::new(-x, -y));
            let transform = translate_to * rotate * translate_back;
            renderer.draw_text(ylabel, Point::new(x, y), &style, transform);
        }
    }

    // -----------------------------------------------------------------------
    // Step 10: Legend
    // -----------------------------------------------------------------------

    /// Draws a box-and-whisker plot: box from Q1 to Q3, median line, whiskers,
    /// caps, and optional outlier dots.
    fn draw_boxplot(
        &self,
        renderer: &mut impl Renderer,
        artist: &BoxPlotArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) {
        let n = artist.stats.len();
        if n == 0 {
            return;
        }

        let fill_color = artist.color.with_alpha((artist.alpha * 255.0) as u8);
        let stroke_color = Color::BLACK.with_alpha((artist.alpha * 255.0) as u8);
        let paint = Paint::new(stroke_color);
        let thin = Stroke::new(1.0);
        let thick = Stroke::new(2.0);
        let hair = Stroke::new(0.5);

        for (i, stats) in artist.stats.iter().enumerate() {
            let cx = i as f64 + 0.5;
            let half = artist.box_width / 2.0;
            let left = cx - half;
            let right = cx + half;

            // --- Box (Q1 to Q3) ---
            let tl = self.data_to_pixel(left, stats.q3, plot_area, xmin, xmax, ymin, ymax);
            let br = self.data_to_pixel(right, stats.q1, plot_area, xmin, xmax, ymin, ymax);
            let box_rect_path = {
                let mut p = Path::new();
                p.move_to(tl.x, tl.y);
                p.line_to(br.x, tl.y);
                p.line_to(br.x, br.y);
                p.line_to(tl.x, br.y);
                p.close();
                p
            };
            renderer.fill_path(&box_rect_path, &Paint::new(fill_color), Affine::IDENTITY);
            renderer.stroke_path(&box_rect_path, &paint, &thin, Affine::IDENTITY);

            // --- Median line ---
            let ml = self.data_to_pixel(left, stats.median, plot_area, xmin, xmax, ymin, ymax);
            let mr = self.data_to_pixel(right, stats.median, plot_area, xmin, xmax, ymin, ymax);
            let mut median_path = Path::new();
            median_path.move_to(ml.x, ml.y);
            median_path.line_to(mr.x, mr.y);
            renderer.stroke_path(&median_path, &paint, &thick, Affine::IDENTITY);

            // --- Lower whisker ---
            let wl_bottom =
                self.data_to_pixel(cx, stats.whisker_low, plot_area, xmin, xmax, ymin, ymax);
            let wl_top = self.data_to_pixel(cx, stats.q1, plot_area, xmin, xmax, ymin, ymax);
            let mut wl_path = Path::new();
            wl_path.move_to(wl_top.x, wl_top.y);
            wl_path.line_to(wl_bottom.x, wl_bottom.y);
            renderer.stroke_path(&wl_path, &paint, &thin, Affine::IDENTITY);

            // Lower cap
            let cap_left = self.data_to_pixel(
                cx - half * 0.5,
                stats.whisker_low,
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );
            let cap_right = self.data_to_pixel(
                cx + half * 0.5,
                stats.whisker_low,
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );
            let mut cap_path = Path::new();
            cap_path.move_to(cap_left.x, cap_left.y);
            cap_path.line_to(cap_right.x, cap_right.y);
            renderer.stroke_path(&cap_path, &paint, &thin, Affine::IDENTITY);

            // --- Upper whisker ---
            let wu_bottom = self.data_to_pixel(cx, stats.q3, plot_area, xmin, xmax, ymin, ymax);
            let wu_top =
                self.data_to_pixel(cx, stats.whisker_high, plot_area, xmin, xmax, ymin, ymax);
            let mut wu_path = Path::new();
            wu_path.move_to(wu_bottom.x, wu_bottom.y);
            wu_path.line_to(wu_top.x, wu_top.y);
            renderer.stroke_path(&wu_path, &paint, &thin, Affine::IDENTITY);

            // Upper cap
            let ucap_left = self.data_to_pixel(
                cx - half * 0.5,
                stats.whisker_high,
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );
            let ucap_right = self.data_to_pixel(
                cx + half * 0.5,
                stats.whisker_high,
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );
            let mut ucap_path = Path::new();
            ucap_path.move_to(ucap_left.x, ucap_left.y);
            ucap_path.line_to(ucap_right.x, ucap_right.y);
            renderer.stroke_path(&ucap_path, &paint, &thin, Affine::IDENTITY);

            // --- Outliers ---
            if artist.show_outliers {
                let r = 3.0;
                for &val in &stats.outliers {
                    let pt = self.data_to_pixel(cx, val, plot_area, xmin, xmax, ymin, ymax);
                    let mut dot = Path::new();
                    for seg in 0..8 {
                        let angle = std::f64::consts::TAU * seg as f64 / 8.0;
                        let dx = r * angle.cos();
                        let dy = r * angle.sin();
                        if seg == 0 {
                            dot.move_to(pt.x + dx, pt.y + dy);
                        } else {
                            dot.line_to(pt.x + dx, pt.y + dy);
                        }
                    }
                    dot.close();
                    renderer.fill_path(&dot, &Paint::new(fill_color), Affine::IDENTITY);
                    renderer.stroke_path(&dot, &paint, &hair, Affine::IDENTITY);
                }
            }
        }
    }

    /// Draws a step (staircase) chart.
    fn draw_step(
        &self,
        renderer: &mut impl Renderer,
        artist: &StepArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) {
        if artist.x.len() < 2 {
            return;
        }
        let color = artist.color.with_alpha((artist.alpha * 255.0) as u8);
        let paint = Paint::new(color);
        let stroke = Stroke::new(artist.width);

        let mut path = Path::new();
        let first = self.data_to_pixel(
            artist.x.data[0],
            artist.y.data[0],
            plot_area,
            xmin,
            xmax,
            ymin,
            ymax,
        );
        path.move_to(first.x, first.y);

        for i in 1..artist.x.len() {
            let prev = self.data_to_pixel(
                artist.x.data[i - 1],
                artist.y.data[i - 1],
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );
            let cur = self.data_to_pixel(
                artist.x.data[i],
                artist.y.data[i],
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );
            match artist.where_step {
                StepWhere::Pre => {
                    path.line_to(prev.x, cur.y);
                    path.line_to(cur.x, cur.y);
                }
                StepWhere::Post => {
                    path.line_to(cur.x, prev.y);
                    path.line_to(cur.x, cur.y);
                }
                StepWhere::Mid => {
                    let mid_x = (prev.x + cur.x) / 2.0;
                    path.line_to(mid_x, prev.y);
                    path.line_to(mid_x, cur.y);
                    path.line_to(cur.x, cur.y);
                }
            }
        }
        renderer.stroke_path(&path, &paint, &stroke, Affine::IDENTITY);
    }

    /// Draws a stem (lollipop) chart.
    fn draw_stem(
        &self,
        renderer: &mut impl Renderer,
        artist: &StemArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) {
        if artist.x.is_empty() {
            return;
        }
        let alpha_byte = (artist.alpha * 255.0) as u8;
        let color = artist.color.with_alpha(alpha_byte);
        let paint = Paint::new(color);
        let stroke = Stroke::new(artist.line_width);
        let radius = artist.marker_size / 2.0;

        // Draw baseline.
        let bl_left = self.data_to_pixel(
            artist.x.data[0],
            artist.baseline,
            plot_area,
            xmin,
            xmax,
            ymin,
            ymax,
        );
        let bl_right = self.data_to_pixel(
            *artist.x.data.last().unwrap(),
            artist.baseline,
            plot_area,
            xmin,
            xmax,
            ymin,
            ymax,
        );
        let mut bl_path = Path::new();
        bl_path.move_to(bl_left.x, bl_left.y);
        bl_path.line_to(bl_right.x, bl_right.y);
        let bl_paint = Paint::new(Color::BLACK.with_alpha(alpha_byte));
        let bl_stroke = Stroke::new(0.8);
        renderer.stroke_path(&bl_path, &bl_paint, &bl_stroke, Affine::IDENTITY);

        // Draw stems and markers.
        for i in 0..artist.x.len() {
            let base = self.data_to_pixel(
                artist.x.data[i],
                artist.baseline,
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );
            let tip = self.data_to_pixel(
                artist.x.data[i],
                artist.y.data[i],
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );
            let mut stem_path = Path::new();
            stem_path.move_to(base.x, base.y);
            stem_path.line_to(tip.x, tip.y);
            renderer.stroke_path(&stem_path, &paint, &stroke, Affine::IDENTITY);
            let marker = Path::circle(tip, radius);
            renderer.fill_path(&marker, &paint, Affine::IDENTITY);
        }
    }

    /// Draws an error bar plot: center line with circle markers, vertical
    /// and/or horizontal error bars with caps.
    fn draw_errorbar(
        &self,
        renderer: &mut impl Renderer,
        artist: &ErrorBarArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) {
        if artist.x.is_empty() {
            return;
        }
        let paint = Paint::new(artist.color);
        let stroke = Stroke::new(artist.line_width);
        let marker_radius = 3.0;

        // Draw center connecting line.
        let mut line_path = Path::new();
        let first = self.data_to_pixel(
            artist.x.data[0],
            artist.y.data[0],
            plot_area,
            xmin,
            xmax,
            ymin,
            ymax,
        );
        line_path.move_to(first.x, first.y);
        for i in 1..artist.x.len() {
            let pt = self.data_to_pixel(
                artist.x.data[i],
                artist.y.data[i],
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );
            line_path.line_to(pt.x, pt.y);
        }
        renderer.stroke_path(&line_path, &paint, &stroke, Affine::IDENTITY);

        // Draw markers and error bars for each point.
        for i in 0..artist.x.len() {
            let xv = artist.x.data[i];
            let yv = artist.y.data[i];
            let center = self.data_to_pixel(xv, yv, plot_area, xmin, xmax, ymin, ymax);

            // Circle marker at the data point.
            let marker = Path::circle(center, marker_radius);
            renderer.fill_path(&marker, &paint, Affine::IDENTITY);

            // Vertical error bars (yerr).
            if let Some(ref yerr) = artist.yerr {
                let (lo, hi) = match yerr {
                    ErrorBarData::Symmetric(e) => (yv - e[i], yv + e[i]),
                    ErrorBarData::Asymmetric { low, high } => (yv - low[i], yv + high[i]),
                };
                let pt_lo = self.data_to_pixel(xv, lo, plot_area, xmin, xmax, ymin, ymax);
                let pt_hi = self.data_to_pixel(xv, hi, plot_area, xmin, xmax, ymin, ymax);

                // Vertical bar.
                let mut bar = Path::new();
                bar.move_to(pt_lo.x, pt_lo.y);
                bar.line_to(pt_hi.x, pt_hi.y);
                renderer.stroke_path(&bar, &paint, &stroke, Affine::IDENTITY);

                // Caps.
                if artist.cap_size > 0.0 {
                    let half_cap = artist.cap_size / 2.0;
                    let mut cap_lo = Path::new();
                    cap_lo.move_to(pt_lo.x - half_cap, pt_lo.y);
                    cap_lo.line_to(pt_lo.x + half_cap, pt_lo.y);
                    renderer.stroke_path(&cap_lo, &paint, &stroke, Affine::IDENTITY);

                    let mut cap_hi = Path::new();
                    cap_hi.move_to(pt_hi.x - half_cap, pt_hi.y);
                    cap_hi.line_to(pt_hi.x + half_cap, pt_hi.y);
                    renderer.stroke_path(&cap_hi, &paint, &stroke, Affine::IDENTITY);
                }
            }

            // Horizontal error bars (xerr).
            if let Some(ref xerr) = artist.xerr {
                let (lo, hi) = match xerr {
                    ErrorBarData::Symmetric(e) => (xv - e[i], xv + e[i]),
                    ErrorBarData::Asymmetric { low, high } => (xv - low[i], xv + high[i]),
                };
                let pt_lo = self.data_to_pixel(lo, yv, plot_area, xmin, xmax, ymin, ymax);
                let pt_hi = self.data_to_pixel(hi, yv, plot_area, xmin, xmax, ymin, ymax);

                // Horizontal bar.
                let mut bar = Path::new();
                bar.move_to(pt_lo.x, pt_lo.y);
                bar.line_to(pt_hi.x, pt_hi.y);
                renderer.stroke_path(&bar, &paint, &stroke, Affine::IDENTITY);

                // Caps.
                if artist.cap_size > 0.0 {
                    let half_cap = artist.cap_size / 2.0;
                    let mut cap_lo = Path::new();
                    cap_lo.move_to(pt_lo.x, pt_lo.y - half_cap);
                    cap_lo.line_to(pt_lo.x, pt_lo.y + half_cap);
                    renderer.stroke_path(&cap_lo, &paint, &stroke, Affine::IDENTITY);

                    let mut cap_hi = Path::new();
                    cap_hi.move_to(pt_hi.x, pt_hi.y - half_cap);
                    cap_hi.line_to(pt_hi.x, pt_hi.y + half_cap);
                    renderer.stroke_path(&cap_hi, &paint, &stroke, Affine::IDENTITY);
                }
            }
        }
    }

    /// Draws a heatmap: fills rectangles for each cell, colored via the
    /// configured colormap. Optionally draws cell value text.
    fn draw_heatmap(
        &self,
        renderer: &mut impl Renderer,
        artist: &HeatmapArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) {
        let nrows = artist.data.len();
        if nrows == 0 {
            return;
        }
        let ncols = artist.data[0].len();
        if ncols == 0 {
            return;
        }

        let vmin = artist.effective_vmin();
        let vmax = artist.effective_vmax();

        let text_style = TextStyle {
            size: 10.0,
            color: Color::BLACK,
            weight: FontWeight::Normal,
            family: None,
            halign: HAlign::Center,
            valign: VAlign::Middle,
        };

        for row in 0..nrows {
            for col in 0..ncols {
                let val = artist.data[row][col];
                let cell_color = artist.cmap.map_value(val, vmin, vmax);

                // Cell rectangle in data space: col..col+1 on x, row..row+1 on y.
                let p_bl =
                    self.data_to_pixel(col as f64, row as f64, plot_area, xmin, xmax, ymin, ymax);
                let p_tr = self.data_to_pixel(
                    (col + 1) as f64,
                    (row + 1) as f64,
                    plot_area,
                    xmin,
                    xmax,
                    ymin,
                    ymax,
                );
                let rect = Rect::from_points(p_bl, p_tr);
                let cell_path = Path::rect(rect);
                renderer.fill_path(&cell_path, &Paint::new(cell_color), Affine::IDENTITY);

                // Optional value text.
                if artist.show_values {
                    let cx = (p_bl.x + p_tr.x) / 2.0;
                    let cy = (p_bl.y + p_tr.y) / 2.0;
                    let label = format!("{val:.1}");
                    renderer.draw_text(&label, Point::new(cx, cy), &text_style, Affine::IDENTITY);
                }
            }
        }
    }

    /// Draws a polar chart: concentric r-grid circles, radial theta-grid lines,
    /// and the data path in polar coordinates.
    fn draw_polar(
        &self,
        renderer: &mut impl Renderer,
        artist: &PolarArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
        theme: &Theme,
    ) {
        let n = artist.r.len().min(artist.theta.len());
        if n == 0 {
            return;
        }

        let r_max = artist.max_finite_r();
        if r_max <= 0.0 || !r_max.is_finite() {
            return;
        }

        // The center of the plot in pixel space. The polar data is centered
        // at the origin in data space (0, 0).
        let center = self.data_to_pixel(0.0, 0.0, plot_area, xmin, xmax, ymin, ymax);

        // Use the smaller dimension to avoid clipping.
        let max_radius_px = (plot_area.width / 2.0).min(plot_area.height / 2.0) * 0.85;

        // --- Draw r-grid: concentric circles ---
        let num_r_rings = 5;
        let r_step = r_max / num_r_rings as f64;
        let grid_color = theme.grid_color;
        let grid_paint = Paint::new(grid_color.with_alpha(100));
        let grid_stroke = Stroke::new(0.5);
        let label_style = TextStyle {
            size: 9.0,
            color: theme.tick_color,
            weight: FontWeight::Normal,
            family: theme.font_family.clone(),
            halign: HAlign::Left,
            valign: VAlign::Middle,
        };

        for i in 1..=num_r_rings {
            let r_val = i as f64 * r_step;
            let r_px = r_val / r_max * max_radius_px;
            let circle = Path::circle(center, r_px);
            renderer.stroke_path(&circle, &grid_paint, &grid_stroke, Affine::IDENTITY);

            // R-axis label at the right side of each circle
            let label_pt = Point::new(center.x + r_px + 3.0, center.y - 2.0);
            let label_text = if r_val == r_val.floor() {
                format!("{:.0}", r_val)
            } else {
                format!("{:.1}", r_val)
            };
            renderer.draw_text(&label_text, label_pt, &label_style, Affine::IDENTITY);
        }

        // --- Draw theta-grid: radial lines at 0, 30, 60, ..., 330 degrees ---
        let angle_label_style = TextStyle {
            size: 10.0,
            color: theme.tick_color,
            weight: FontWeight::Normal,
            family: theme.font_family.clone(),
            halign: HAlign::Center,
            valign: VAlign::Middle,
        };

        for deg in (0..360).step_by(30) {
            let angle = (deg as f64).to_radians();
            let end_x = center.x + max_radius_px * angle.cos();
            let end_y = center.y - max_radius_px * angle.sin();
            let mut line = Path::new();
            line.move_to(center.x, center.y);
            line.line_to(end_x, end_y);
            renderer.stroke_path(&line, &grid_paint, &grid_stroke, Affine::IDENTITY);

            // Angle label at the tip of each radial line
            let label_offset = 14.0;
            let lx = center.x + (max_radius_px + label_offset) * angle.cos();
            let ly = center.y - (max_radius_px + label_offset) * angle.sin();
            let label_text = format!("{}°", deg);
            renderer.draw_text(
                &label_text,
                Point::new(lx, ly),
                &angle_label_style,
                Affine::IDENTITY,
            );
        }

        // --- Draw the data path ---
        // Helper: convert (r, theta) -> pixel coordinates
        let to_px = |r: f64, theta: f64| -> Point {
            let px_r = r / r_max * max_radius_px;
            Point::new(center.x + px_r * theta.cos(), center.y - px_r * theta.sin())
        };

        let mut path = Path::new();
        let mut started = false;

        for i in 0..n {
            let r = artist.r[i];
            let theta = artist.theta[i];
            if !r.is_finite() || !theta.is_finite() || r < 0.0 {
                continue;
            }
            let pt = to_px(r, theta);
            if !started {
                path.move_to(pt.x, pt.y);
                started = true;
            } else {
                path.line_to(pt.x, pt.y);
            }
        }

        if !started {
            return;
        }

        let alpha_byte = (artist.alpha * 255.0) as u8;
        let color = artist.color.with_alpha(alpha_byte);

        if artist.filled {
            // Close the path and fill
            path.close();
            let fill_paint = Paint::new(color);
            renderer.fill_path(&path, &fill_paint, Affine::IDENTITY);
            // Draw the outline
            let stroke_paint = Paint::new(
                artist
                    .color
                    .with_alpha(((artist.alpha.min(1.0) * 0.8 + 0.2) * 255.0) as u8),
            );
            let stroke = Stroke::new(artist.linewidth);
            renderer.stroke_path(&path, &stroke_paint, &stroke, Affine::IDENTITY);
        } else {
            // Stroke the path
            let paint = Paint::new(color);
            let stroke = Stroke::new(artist.linewidth);
            renderer.stroke_path(&path, &paint, &stroke, Affine::IDENTITY);
        }

        // --- Draw markers at each data point ---
        if let Some(marker) = artist.marker {
            let marker_size = 5.0;
            let marker_radius = marker_size / 2.0;
            let marker_paint = Paint::new(color);

            for i in 0..n {
                let r = artist.r[i];
                let theta = artist.theta[i];
                if !r.is_finite() || !theta.is_finite() || r < 0.0 {
                    continue;
                }
                let pt = to_px(r, theta);
                let marker_path = match marker {
                    Marker::Circle | Marker::Point => Path::circle(pt, marker_radius),
                    Marker::Square => Path::rect(Rect::new(
                        pt.x - marker_radius,
                        pt.y - marker_radius,
                        marker_size,
                        marker_size,
                    )),
                    Marker::Diamond => {
                        let mut p = Path::new();
                        p.move_to(pt.x, pt.y - marker_radius);
                        p.line_to(pt.x + marker_radius, pt.y);
                        p.line_to(pt.x, pt.y + marker_radius);
                        p.line_to(pt.x - marker_radius, pt.y);
                        p.close();
                        p
                    }
                    Marker::Triangle => {
                        let mut p = Path::new();
                        let h = marker_radius * 1.1547;
                        p.move_to(pt.x, pt.y - marker_radius);
                        p.line_to(pt.x + h * 0.5, pt.y + marker_radius * 0.5);
                        p.line_to(pt.x - h * 0.5, pt.y + marker_radius * 0.5);
                        p.close();
                        p
                    }
                    Marker::Plus => {
                        let mut p = Path::new();
                        p.move_to(pt.x - marker_radius, pt.y);
                        p.line_to(pt.x + marker_radius, pt.y);
                        p.move_to(pt.x, pt.y - marker_radius);
                        p.line_to(pt.x, pt.y + marker_radius);
                        let ms = Stroke::new(theme.line_width.max(1.0));
                        renderer.stroke_path(&p, &marker_paint, &ms, Affine::IDENTITY);
                        continue;
                    }
                    Marker::Cross => {
                        let mut p = Path::new();
                        let d = marker_radius * 0.707;
                        p.move_to(pt.x - d, pt.y - d);
                        p.line_to(pt.x + d, pt.y + d);
                        p.move_to(pt.x + d, pt.y - d);
                        p.line_to(pt.x - d, pt.y + d);
                        let ms = Stroke::new(theme.line_width.max(1.0));
                        renderer.stroke_path(&p, &marker_paint, &ms, Affine::IDENTITY);
                        continue;
                    }
                    Marker::Star => {
                        let mut p = Path::new();
                        let inner = marker_radius * 0.382;
                        for j in 0..10 {
                            let a =
                                std::f64::consts::FRAC_PI_2 + j as f64 * std::f64::consts::PI / 5.0;
                            let r = if j % 2 == 0 { marker_radius } else { inner };
                            let sx = pt.x + r * a.cos();
                            let sy = pt.y - r * a.sin();
                            if j == 0 {
                                p.move_to(sx, sy);
                            } else {
                                p.line_to(sx, sy);
                            }
                        }
                        p.close();
                        p
                    }
                };
                renderer.fill_path(&marker_path, &marker_paint, Affine::IDENTITY);
            }
        }
    }

    /// Draws a hexbin plot: bins data points into hexagonal cells, maps
    /// counts through the colormap, and renders filled hexagons.
    fn draw_hexbin(
        &self,
        renderer: &mut impl Renderer,
        artist: &HexbinArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) {
        use crate::charts::hexbin::{bin_hexagonal, hex_size_for_gridsize, hexagon_vertices};

        let result = bin_hexagonal(&artist.x, &artist.y, artist.gridsize, artist.mincnt);
        if result.cells.is_empty() {
            return;
        }

        let vmin = result.min_count as f64;
        let vmax = result.max_count as f64;

        // Compute hex size in data space.
        let data_xrange = (xmax - xmin).max(f64::EPSILON);
        let hex_data_size = hex_size_for_gridsize(data_xrange, artist.gridsize);

        let alpha_byte = (artist.alpha * 255.0).round() as u8;

        for &(cx, cy, count) in &result.cells {
            // Map count to color via colormap.
            let mut fill_color = artist.cmap.map_value(count as f64, vmin, vmax);
            fill_color = fill_color.with_alpha(alpha_byte);

            // Compute hex vertices in data space, then convert to pixel space.
            let data_verts = hexagon_vertices(cx, cy, hex_data_size);

            let mut path = Path::new();
            for (i, &(vx, vy)) in data_verts.iter().enumerate() {
                let p = self.data_to_pixel(vx, vy, plot_area, xmin, xmax, ymin, ymax);
                if i == 0 {
                    path.move_to(p.x, p.y);
                } else {
                    path.line_to(p.x, p.y);
                }
            }
            path.close();

            renderer.fill_path(&path, &Paint::new(fill_color), Affine::IDENTITY);

            // Optional edge stroke.
            if let Some(edge_color) = artist.edgecolor {
                let stroke = Stroke::new(0.5);
                renderer.stroke_path(
                    &path,
                    &Paint::new(edge_color.with_alpha(alpha_byte)),
                    &stroke,
                    Affine::IDENTITY,
                );
            }
        }
    }

    /// Draws a waterfall chart: bars showing cumulative positive/negative changes.
    fn draw_waterfall(
        &self,
        renderer: &mut impl Renderer,
        artist: &WaterfallArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) {
        let n = artist.categories.len();
        if n == 0 {
            return;
        }

        let positions = artist.bar_positions();
        let cumsum = artist.cumulative_sums();

        let cat_range = xmax - xmin;
        let cat_step = cat_range / n as f64;
        let bar_half = cat_step * artist.bar_width * 0.5;

        for i in 0..n {
            let (base, top) = positions[i];

            // Select bar color based on type.
            let bar_color = if artist.total_indices.contains(&i) {
                artist.total_color
            } else if artist.values.data[i] >= 0.0 {
                artist.increase_color
            } else {
                artist.decrease_color
            };

            let color = bar_color.with_alpha((artist.alpha * 255.0) as u8);
            let paint = Paint::new(color);

            let cat_center = xmin + (i as f64 + 0.5) * cat_step;
            let bottom_val = base.min(top);
            let top_val = base.max(top);

            let p_bl = self.data_to_pixel(
                cat_center - bar_half,
                bottom_val,
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );
            let p_tr = self.data_to_pixel(
                cat_center + bar_half,
                top_val,
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );

            let rect = Rect::from_points(p_bl, p_tr);
            let bar_path = Path::rect(rect);
            renderer.fill_path(&bar_path, &paint, Affine::IDENTITY);

            // Draw value label on the bar if enabled.
            if artist.show_values {
                let display_val = if artist.total_indices.contains(&i) {
                    cumsum[i]
                } else {
                    artist.values.data[i]
                };
                let label_text = format_waterfall_value(display_val);

                // Position label above positive bars, below negative bars.
                let label_y = if display_val >= 0.0 {
                    top_val
                } else {
                    bottom_val
                };
                let label_pos =
                    self.data_to_pixel(cat_center, label_y, plot_area, xmin, xmax, ymin, ymax);

                let text_style = TextStyle {
                    size: 10.0,
                    color: Color::BLACK,
                    weight: FontWeight::Normal,
                    family: None,
                    halign: HAlign::Center,
                    valign: if display_val >= 0.0 {
                        VAlign::Bottom
                    } else {
                        VAlign::Top
                    },
                };
                let offset_pos = Point::new(
                    label_pos.x,
                    if display_val >= 0.0 {
                        label_pos.y - 3.0
                    } else {
                        label_pos.y + 3.0
                    },
                );
                renderer.draw_text(&label_text, offset_pos, &text_style, Affine::IDENTITY);
            }
        }

        // Draw connector lines between consecutive bars.
        if artist.connector_lines && n > 1 {
            let connector_paint = Paint::new(Color::rgb(0x80, 0x80, 0x80).with_alpha(180));
            let connector_stroke = Stroke::new(0.8);

            for (i, &connector_y) in cumsum.iter().enumerate().take(n - 1) {
                let right_edge = xmin + (i as f64 + 0.5) * cat_step + bar_half;
                let left_edge = xmin + ((i + 1) as f64 + 0.5) * cat_step - bar_half;

                let p_from =
                    self.data_to_pixel(right_edge, connector_y, plot_area, xmin, xmax, ymin, ymax);
                let p_to =
                    self.data_to_pixel(left_edge, connector_y, plot_area, xmin, xmax, ymin, ymax);

                let mut path = Path::new();
                path.move_to(p_from.x, p_from.y);
                path.line_to(p_to.x, p_to.y);
                renderer.stroke_path(&path, &connector_paint, &connector_stroke, Affine::IDENTITY);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Step 9b: Text and arrow annotations
    // -----------------------------------------------------------------------

    /// Draws all text annotations and arrow annotations.
    fn draw_annotations(
        &self,
        renderer: &mut impl Renderer,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
        theme: &Theme,
    ) {
        // --- TextAnnotation items ---
        for ta in &self.texts {
            let pt = self.data_to_pixel(ta.x, ta.y, plot_area, xmin, xmax, ymin, ymax);
            let size = ta.fontsize.unwrap_or(theme.axis_label_size);
            let color = ta.color.unwrap_or(theme.text_color);
            let style = TextStyle {
                size,
                color,
                weight: FontWeight::Normal,
                family: theme.font_family.clone(),
                halign: ta.ha,
                valign: ta.va,
            };
            if ta.rotation.abs() < f64::EPSILON {
                renderer.draw_text(&ta.text, pt, &style, Affine::IDENTITY);
            } else {
                let angle_rad = -ta.rotation.to_radians();
                let rotate = Affine::rotate(angle_rad);
                let translate_to = Affine::translate(kurbo::Vec2::new(pt.x, pt.y));
                let translate_back = Affine::translate(kurbo::Vec2::new(-pt.x, -pt.y));
                let transform = translate_to * rotate * translate_back;
                renderer.draw_text(&ta.text, pt, &style, transform);
            }
        }

        // --- Annotation items (text + optional arrow) ---
        for ann in &self.annotations {
            let text_pt = self.data_to_pixel(
                ann.xytext.0,
                ann.xytext.1,
                plot_area,
                xmin,
                xmax,
                ymin,
                ymax,
            );
            let target_pt =
                self.data_to_pixel(ann.xy.0, ann.xy.1, plot_area, xmin, xmax, ymin, ymax);

            let size = ann.fontsize.unwrap_or(theme.axis_label_size);
            let color = ann.color.unwrap_or(theme.text_color);
            let style = TextStyle {
                size,
                color,
                weight: FontWeight::Normal,
                family: theme.font_family.clone(),
                halign: ann.ha,
                valign: ann.va,
            };
            renderer.draw_text(&ann.text, text_pt, &style, Affine::IDENTITY);

            // Draw arrow if requested.
            if ann.arrowstyle != ArrowStyle::None {
                let arrow_col = ann.arrow_color.unwrap_or(color);
                self.draw_annotation_arrow(
                    renderer,
                    text_pt,
                    target_pt,
                    arrow_col,
                    &ann.arrowstyle,
                );
            }
        }
    }

    /// Draws an arrow line from `from` to `to` with an arrowhead at `to`.
    fn draw_annotation_arrow(
        &self,
        renderer: &mut impl Renderer,
        from: Point,
        to: Point,
        color: Color,
        style: &ArrowStyle,
    ) {
        let paint = Paint::new(color);
        let stroke = Stroke::new(1.0);

        // Direction vector from `from` to `to`.
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-6 {
            return;
        }

        // Unit direction vector.
        let ux = dx / len;
        let uy = dy / len;

        // Draw the line from `from` to `to`.
        let mut line = Path::new();
        line.move_to(from.x, from.y);
        line.line_to(to.x, to.y);
        renderer.stroke_path(&line, &paint, &stroke, Affine::IDENTITY);

        // Draw a filled triangular arrowhead at `to`.
        let head_len = match style {
            ArrowStyle::None => return,
            ArrowStyle::Simple => 8.0,
            ArrowStyle::Fancy => 12.0,
        };
        let head_half_width = match style {
            ArrowStyle::None => return,
            ArrowStyle::Simple => 3.0,
            ArrowStyle::Fancy => 5.0,
        };

        // Perpendicular to the direction.
        let px = -uy;
        let py = ux;

        // Triangle vertices: tip at `to`, base offset by head_len along -direction.
        let base_x = to.x - ux * head_len;
        let base_y = to.y - uy * head_len;

        let left_x = base_x + px * head_half_width;
        let left_y = base_y + py * head_half_width;
        let right_x = base_x - px * head_half_width;
        let right_y = base_y - py * head_half_width;

        let mut arrow = Path::new();
        arrow.move_to(to.x, to.y);
        arrow.line_to(left_x, left_y);
        arrow.line_to(right_x, right_y);
        arrow.close();
        renderer.fill_path(&arrow, &paint, Affine::IDENTITY);
    }

    /// Draws a pie chart: fills arc wedges centered in the plot area.
    ///
    /// Each wedge is built as a closed path from the center to the arc
    /// perimeter, using cubic Bezier segments to approximate circular
    /// arcs (each sub-arc spans at most 90 degrees).
    fn draw_pie(
        &self,
        renderer: &mut impl Renderer,
        artist: &PieArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
        theme: &Theme,
    ) {
        let n = artist.sizes.len();
        if n == 0 {
            return;
        }

        // Normalise sizes to fractions summing to 1.0.
        let total: f64 = artist
            .sizes
            .iter()
            .copied()
            .filter(|v| v.is_finite() && *v > 0.0)
            .sum();
        if total <= 0.0 {
            return;
        }
        let fractions: Vec<f64> = artist
            .sizes
            .iter()
            .map(|&s| {
                if s.is_finite() && s > 0.0 {
                    s / total
                } else {
                    0.0
                }
            })
            .collect();

        // Center of the pie in data space is (0, 0).
        let center_px = self.data_to_pixel(0.0, 0.0, plot_area, xmin, xmax, ymin, ymax);

        // Compute pixel radius: map (radius, 0) and use x-distance.
        let edge_px = self.data_to_pixel(artist.radius, 0.0, plot_area, xmin, xmax, ymin, ymax);
        let radius_px = (edge_px.x - center_px.x).abs();

        let start_rad = artist.start_angle.to_radians();
        let mut current_angle = start_rad;

        let pct_style = TextStyle {
            size: 10.0,
            color: Color::BLACK,
            weight: FontWeight::Normal,
            family: None,
            halign: HAlign::Center,
            valign: VAlign::Middle,
        };
        let label_style = TextStyle {
            size: 11.0,
            color: theme.tick_color,
            weight: FontWeight::Normal,
            family: None,
            halign: HAlign::Center,
            valign: VAlign::Middle,
        };

        for i in 0..n {
            let frac = fractions[i];
            if frac <= 0.0 {
                current_angle += frac * std::f64::consts::TAU;
                continue;
            }

            let sweep = frac * std::f64::consts::TAU;
            let mid_angle = current_angle + sweep / 2.0;

            // Resolve wedge color.
            let wedge_color = if let Some(ref colors) = artist.colors {
                colors[i % colors.len()]
            } else {
                Color::TABLEAU_10[i % 10]
            };

            // Compute explode offset in pixels.
            let explode_frac = artist
                .explode
                .as_ref()
                .map(|e| if i < e.len() { e[i] } else { 0.0 })
                .unwrap_or(0.0);
            let offset_x = explode_frac * radius_px * mid_angle.cos();
            let offset_y = explode_frac * radius_px * (-mid_angle.sin()); // y inverted

            let cx = center_px.x + offset_x;
            let cy = center_px.y + offset_y;

            // Build the wedge path: move to center, line to arc start,
            // approximate arc with cubic Beziers, close back to center.
            let mut path = Path::new();
            path.move_to(cx, cy);

            let arc_start_x = cx + radius_px * current_angle.cos();
            let arc_start_y = cy - radius_px * current_angle.sin();
            path.line_to(arc_start_x, arc_start_y);

            // Split the sweep into sub-arcs of at most 90 degrees.
            let max_sub = std::f64::consts::FRAC_PI_2;
            let num_segments = (sweep / max_sub).ceil() as usize;
            let seg_sweep = sweep / num_segments as f64;
            let mut seg_start = current_angle;

            for _ in 0..num_segments {
                let seg_end = seg_start + seg_sweep;
                // Cubic Bezier approximation for a circular arc.
                let half = seg_sweep / 2.0;
                let alpha = (4.0 / 3.0) * (half / 2.0).tan();

                let p0x = cx + radius_px * seg_start.cos();
                let p0y = cy - radius_px * seg_start.sin();
                let p3x = cx + radius_px * seg_end.cos();
                let p3y = cy - radius_px * seg_end.sin();

                // Tangent direction at start: perpendicular to radial.
                let t0x = -seg_start.sin();
                let t0y = -seg_start.cos(); // y inverted in pixel space
                                            // Tangent direction at end: perpendicular to radial.
                let t1x = -seg_end.sin();
                let t1y = -seg_end.cos(); // y inverted in pixel space

                let cp1x = p0x + alpha * radius_px * t0x;
                let cp1y = p0y + alpha * radius_px * t0y;
                let cp2x = p3x - alpha * radius_px * t1x;
                let cp2y = p3y - alpha * radius_px * t1y;

                let _ = path.curve_to(cp1x, cp1y, cp2x, cp2y, p3x, p3y);
                seg_start = seg_end;
            }

            path.close();

            let paint = Paint::new(wedge_color);
            renderer.fill_path(&path, &paint, Affine::IDENTITY);

            // Thin white outline for visual separation between wedges.
            let outline_paint = Paint::new(Color::WHITE);
            let outline_stroke = Stroke::new(1.5);
            renderer.stroke_path(&path, &outline_paint, &outline_stroke, Affine::IDENTITY);

            // Percentage label at the midpoint of the arc.
            if artist.autopct {
                let pct_r = radius_px * 0.6;
                let pct_x = cx + pct_r * mid_angle.cos();
                let pct_y = cy - pct_r * mid_angle.sin();
                let pct_text = format!("{:.1}%", frac * 100.0);
                renderer.draw_text(
                    &pct_text,
                    Point::new(pct_x, pct_y),
                    &pct_style,
                    Affine::IDENTITY,
                );
            }

            // Wedge labels outside the arc.
            if let Some(ref labels) = artist.labels {
                if i < labels.len() {
                    let label_r = radius_px * 1.15;
                    let lx = cx + label_r * mid_angle.cos();
                    let ly = cy - label_r * mid_angle.sin();
                    renderer.draw_text(
                        &labels[i],
                        Point::new(lx, ly),
                        &label_style,
                        Affine::IDENTITY,
                    );
                }
            }

            current_angle += sweep;
        }
    }

    /// Draws a contour or filled contour plot.
    fn draw_contour(
        &self,
        renderer: &mut impl Renderer,
        artist: &ContourArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) {
        let nx = artist.x.len();
        let ny = artist.y.len();
        if nx < 2 || ny < 2 || artist.z.len() < 2 {
            return;
        }

        let levels = artist.effective_levels();
        let (zmin, zmax) = artist.z_bounds();

        if artist.filled {
            let avgs = artist.cell_averages();
            for (j, row) in avgs.iter().enumerate() {
                for (i, &avg) in row.iter().enumerate() {
                    if !avg.is_finite() {
                        continue;
                    }
                    let cell_color = if let Some(ref colors) = artist.colors {
                        let idx = levels
                            .iter()
                            .position(|&l| avg < l)
                            .unwrap_or(levels.len())
                            .saturating_sub(1);
                        colors[idx % colors.len()]
                    } else {
                        artist.cmap.map_value(avg, zmin, zmax)
                    };
                    let p_bl = self.data_to_pixel(
                        artist.x[i],
                        artist.y[j],
                        plot_area,
                        xmin,
                        xmax,
                        ymin,
                        ymax,
                    );
                    let p_tr = self.data_to_pixel(
                        artist.x[i + 1],
                        artist.y[j + 1],
                        plot_area,
                        xmin,
                        xmax,
                        ymin,
                        ymax,
                    );
                    let rect = Rect::from_points(p_bl, p_tr);
                    let cell_path = Path::rect(rect);
                    renderer.fill_path(&cell_path, &Paint::new(cell_color), Affine::IDENTITY);
                }
            }
        }

        if !artist.filled {
            for (li, &level) in levels.iter().enumerate() {
                let segments = artist.marching_squares(level);
                if segments.is_empty() {
                    continue;
                }
                let line_color = if let Some(ref colors) = artist.colors {
                    colors[li % colors.len()]
                } else {
                    artist.cmap.map_value(level, zmin, zmax)
                };
                let paint = Paint::new(line_color);
                let stroke = Stroke::new(artist.linewidths);
                for (sx0, sy0, sx1, sy1) in &segments {
                    let p0 = self.data_to_pixel(*sx0, *sy0, plot_area, xmin, xmax, ymin, ymax);
                    let p1 = self.data_to_pixel(*sx1, *sy1, plot_area, xmin, xmax, ymin, ymax);
                    let mut path = Path::new();
                    path.move_to(p0.x, p0.y);
                    path.line_to(p1.x, p1.y);
                    renderer.stroke_path(&path, &paint, &stroke, Affine::IDENTITY);
                }
            }
        }
    }

    /// Draws a violin plot.
    fn draw_violin(
        &self,
        renderer: &mut impl Renderer,
        artist: &ViolinArtist,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
        theme: &Theme,
    ) {
        use crate::charts::violin::{gaussian_kde, silverman_bandwidth};

        let fill_color = artist.color.with_alpha((artist.alpha * 255.0) as u8);
        let fill_paint = Paint::new(fill_color);
        let outline_paint = Paint::new(artist.color);
        let outline_stroke = Stroke::new(1.0);

        for (di, data) in artist.datasets.iter().enumerate() {
            let mut sorted: Vec<f64> = data.iter().copied().filter(|v| v.is_finite()).collect();
            if sorted.is_empty() {
                continue;
            }
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let pos = artist
                .positions
                .as_ref()
                .and_then(|p| p.get(di).copied())
                .unwrap_or(di as f64 + 1.0);

            let bw = if artist.bw_method > 0.0 {
                artist.bw_method
            } else {
                silverman_bandwidth(&sorted)
            };

            let data_min = sorted[0];
            let data_max = sorted[sorted.len() - 1];
            let n_eval = 100;
            let eval_points: Vec<f64> = (0..n_eval)
                .map(|i| data_min + (data_max - data_min) * i as f64 / (n_eval - 1) as f64)
                .collect();
            let densities = gaussian_kde(&sorted, bw, &eval_points);

            let max_density = densities.iter().copied().fold(0.0_f64, f64::max);
            if max_density <= 0.0 {
                continue;
            }

            let half_width = artist.widths * 0.5;

            // Build mirrored path
            let mut path = Path::new();
            // Right side (top to bottom in data space)
            let first_y = eval_points[0];
            let first_w = densities[0] / max_density * half_width;
            let fp = self.data_to_pixel(pos + first_w, first_y, plot_area, xmin, xmax, ymin, ymax);
            path.move_to(fp.x, fp.y);
            for i in 1..n_eval {
                let w = densities[i] / max_density * half_width;
                let p =
                    self.data_to_pixel(pos + w, eval_points[i], plot_area, xmin, xmax, ymin, ymax);
                path.line_to(p.x, p.y);
            }
            // Left side (bottom to top)
            for i in (0..n_eval).rev() {
                let w = densities[i] / max_density * half_width;
                let p =
                    self.data_to_pixel(pos - w, eval_points[i], plot_area, xmin, xmax, ymin, ymax);
                path.line_to(p.x, p.y);
            }
            path.close();

            renderer.fill_path(&path, &fill_paint, Affine::IDENTITY);
            renderer.stroke_path(&path, &outline_paint, &outline_stroke, Affine::IDENTITY);

            // Median and quartile lines
            let n = sorted.len();
            if artist.show_median {
                let median = if n % 2 == 0 {
                    (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
                } else {
                    sorted[n / 2]
                };
                let med_density = gaussian_kde(&sorted, bw, &[median])[0];
                let med_w = med_density / max_density * half_width;
                let p1 = self.data_to_pixel(pos - med_w, median, plot_area, xmin, xmax, ymin, ymax);
                let p2 = self.data_to_pixel(pos + med_w, median, plot_area, xmin, xmax, ymin, ymax);
                let mut mp = Path::new();
                mp.move_to(p1.x, p1.y);
                mp.line_to(p2.x, p2.y);
                let med_paint = Paint::new(theme.text_color);
                let med_stroke = Stroke::new(2.0);
                renderer.stroke_path(&mp, &med_paint, &med_stroke, Affine::IDENTITY);
            }

            if artist.show_quartiles && n >= 4 {
                let q1 = sorted[n / 4];
                let q3 = sorted[3 * n / 4];
                for q in [q1, q3] {
                    let q_density = gaussian_kde(&sorted, bw, &[q])[0];
                    let q_w = q_density / max_density * half_width;
                    let p1 = self.data_to_pixel(pos - q_w, q, plot_area, xmin, xmax, ymin, ymax);
                    let p2 = self.data_to_pixel(pos + q_w, q, plot_area, xmin, xmax, ymin, ymax);
                    let mut qp = Path::new();
                    qp.move_to(p1.x, p1.y);
                    qp.line_to(p2.x, p2.y);
                    let q_stroke = Stroke::new(1.0).with_dash(DashPattern {
                        dashes: vec![4.0, 2.0],
                        offset: 0.0,
                    });
                    renderer.stroke_path(
                        &qp,
                        &Paint::new(theme.text_color),
                        &q_stroke,
                        Affine::IDENTITY,
                    );
                }
            }
        }
    }

    fn draw_legend(&self, renderer: &mut impl Renderer, plot_area: &Rect, theme: &Theme) {
        // Collect labeled artists into LegendEntry items, choosing the
        // appropriate swatch kind for each artist type.
        let entries: Vec<LegendEntry> = self
            .artists
            .iter()
            .filter_map(|a| {
                let (label, color, swatch) = match a {
                    Artist::Line(a) => (a.label.as_deref(), a.color, SwatchKind::Line),
                    Artist::Scatter(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::Bar(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::Histogram(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::FillBetween(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::Step(a) => (a.label.as_deref(), a.color, SwatchKind::Line),
                    Artist::Stem(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::BoxPlot(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::ErrorBar(a) => (a.label.as_deref(), a.color, SwatchKind::Line),
                    Artist::Heatmap(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::Pie(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::Violin(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::Contour(a) => (
                        a.label.as_deref(),
                        a.color,
                        if a.filled {
                            SwatchKind::Filled
                        } else {
                            SwatchKind::Line
                        },
                    ),
                    Artist::Polar(a) => (
                        a.label.as_deref(),
                        a.color,
                        if a.filled {
                            SwatchKind::Filled
                        } else {
                            SwatchKind::Line
                        },
                    ),
                    Artist::Hexbin(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                    Artist::Waterfall(a) => (a.label.as_deref(), a.color, SwatchKind::Filled),
                };
                label.map(|l| LegendEntry {
                    label: l.to_string(),
                    color,
                    swatch,
                })
            })
            .collect();

        legend::draw_legend(renderer, &entries, plot_area, self.legend_loc, theme);
    }

    // -----------------------------------------------------------------------
    // Coordinate transform
    // -----------------------------------------------------------------------

    /// Maps a data-space `(x, y)` coordinate to a pixel-space [`Point`] within
    /// the given `plot_area` rectangle.
    ///
    /// The x-axis maps left-to-right and the y-axis maps bottom-to-top (i.e.,
    /// pixel y is inverted relative to data y).
    fn data_to_pixel(
        &self,
        x: f64,
        y: f64,
        plot_area: &Rect,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) -> Point {
        let tx = self.xscale.transform(x, xmin, xmax);
        let ty = self.yscale.transform(y, ymin, ymax);
        Point::new(
            plot_area.x + tx * plot_area.width,
            plot_area.y + (1.0 - ty) * plot_area.height, // y is inverted
        )
    }
}

// ---------------------------------------------------------------------------
// Module-level helpers
// ---------------------------------------------------------------------------

/// Formats a waterfall bar value for display.
///
/// Integer-valued floats are displayed without a decimal point.
/// Decimal values are shown with up to 2 significant decimal places,
/// with trailing zeros stripped.
fn format_waterfall_value(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.2}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_axes_has_defaults() {
        let ax = Axes::new();
        assert!(ax.artists.is_empty());
        assert!(ax.title.is_none());
        assert!(ax.xlabel.is_none());
        assert!(ax.ylabel.is_none());
        assert!(ax.xlim.is_none());
        assert!(ax.ylim.is_none());
        assert!(!ax.show_legend);
        assert_eq!(ax.color_index, 0);
    }

    #[test]
    fn plot_creates_line_artist() {
        let mut ax = Axes::new();
        let result = ax.plot(vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]);
        assert!(result.is_ok());
        assert_eq!(ax.artists.len(), 1);
        assert!(matches!(&ax.artists[0], Artist::Line(_)));
        assert_eq!(ax.color_index, 1);
    }

    #[test]
    fn plot_length_mismatch() {
        let mut ax = Axes::new();
        let result = ax.plot(vec![1.0, 2.0], vec![1.0]);
        assert!(matches!(
            result,
            Err(PlotError::SeriesLengthMismatch {
                expected: 2,
                got: 1
            })
        ));
    }

    #[test]
    fn plot_empty_data() {
        let mut ax = Axes::new();
        let result = ax.plot(Vec::<f64>::new(), Vec::<f64>::new());
        assert!(matches!(result, Err(PlotError::EmptyData)));
    }

    #[test]
    fn scatter_creates_artist() {
        let mut ax = Axes::new();
        let result = ax.scatter(vec![1.0, 2.0], vec![3.0, 4.0]);
        assert!(result.is_ok());
        assert!(matches!(&ax.artists[0], Artist::Scatter(_)));
    }

    #[test]
    fn bar_creates_artist() {
        let mut ax = Axes::new();
        let cats: &[&str] = &["a", "b", "c"];
        let result = ax.bar(cats, vec![10.0, 20.0, 30.0]);
        assert!(result.is_ok());
        match &ax.artists[0] {
            Artist::Bar(a) => {
                assert!(!a.horizontal);
                assert_eq!(a.categories.len(), 3);
            }
            _ => panic!("expected Bar artist"),
        }
    }

    #[test]
    fn barh_creates_horizontal_artist() {
        let mut ax = Axes::new();
        let cats: &[&str] = &["x", "y"];
        let result = ax.barh(cats, vec![5.0, 10.0]);
        assert!(result.is_ok());
        match &ax.artists[0] {
            Artist::Bar(a) => assert!(a.horizontal),
            _ => panic!("expected Bar artist"),
        }
    }

    #[test]
    fn hist_computes_bins() {
        let mut ax = Axes::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = ax.hist(data, 5);
        assert!(result.is_ok());
        match &ax.artists[0] {
            Artist::Histogram(a) => {
                assert_eq!(a.bin_edges.len(), 6); // 5 bins = 6 edges
                assert_eq!(a.counts.len(), 5);
                // Total count should equal number of data points.
                let total: f64 = a.counts.iter().sum();
                assert_eq!(total, 10.0);
            }
            _ => panic!("expected Hist artist"),
        }
    }

    #[test]
    fn hist_single_value() {
        let mut ax = Axes::new();
        let result = ax.hist(vec![5.0, 5.0, 5.0], 3);
        assert!(result.is_ok());
        match &ax.artists[0] {
            Artist::Histogram(a) => {
                let total: f64 = a.counts.iter().sum();
                assert_eq!(total, 3.0);
            }
            _ => panic!("expected Hist artist"),
        }
    }

    #[test]
    fn hist_empty_data() {
        let mut ax = Axes::new();
        let result = ax.hist(Vec::<f64>::new(), 10);
        assert!(matches!(result, Err(PlotError::EmptyData)));
    }

    #[test]
    fn fill_between_creates_artist() {
        let mut ax = Axes::new();
        let result = ax.fill_between(
            vec![1.0, 2.0, 3.0],
            vec![1.0, 2.0, 1.0],
            vec![0.0, 0.0, 0.0],
        );
        assert!(result.is_ok());
        assert!(matches!(&ax.artists[0], Artist::FillBetween(_)));
    }

    #[test]
    fn fill_between_length_mismatch() {
        let mut ax = Axes::new();
        let result = ax.fill_between(vec![1.0, 2.0], vec![1.0], vec![0.0, 0.0]);
        assert!(matches!(
            result,
            Err(PlotError::SeriesLengthMismatch { .. })
        ));
    }

    #[test]
    fn configuration_methods_return_self() {
        let mut ax = Axes::new();
        ax.set_title("Test")
            .set_xlabel("X")
            .set_ylabel("Y")
            .set_xlim(0.0, 10.0)
            .set_ylim(-1.0, 1.0)
            .set_xscale(Scale::Linear)
            .set_yscale(Scale::Log10)
            .grid(true)
            .legend();

        assert_eq!(ax.title.as_deref(), Some("Test"));
        assert_eq!(ax.xlabel.as_deref(), Some("X"));
        assert_eq!(ax.ylabel.as_deref(), Some("Y"));
        assert_eq!(ax.xlim, Some((0.0, 10.0)));
        assert_eq!(ax.ylim, Some((-1.0, 1.0)));
        assert_eq!(ax.show_grid, Some(true));
        assert!(ax.show_legend);
    }

    #[test]
    fn color_cycle_advances() {
        let mut ax = Axes::new();
        for _ in 0..12 {
            ax.plot(vec![1.0, 2.0], vec![1.0, 2.0]).unwrap();
        }
        assert_eq!(ax.color_index, 12);
        // 12th artist wraps around: index 10 % 10 == 0, so color should
        // be the same as the first.
        match (&ax.artists[0], &ax.artists[10]) {
            (Artist::Line(a), Artist::Line(b)) => {
                assert_eq!(a.color, b.color);
            }
            _ => panic!("expected Line artists"),
        }
    }

    #[test]
    fn data_to_pixel_linear() {
        let ax = Axes::new();
        let plot_area = Rect::new(100.0, 50.0, 400.0, 300.0);

        // Bottom-left corner of data space.
        let p = ax.data_to_pixel(0.0, 0.0, &plot_area, 0.0, 10.0, 0.0, 10.0);
        assert!((p.x - 100.0).abs() < 1e-10);
        assert!((p.y - 350.0).abs() < 1e-10); // bottom of plot

        // Top-right corner of data space.
        let p = ax.data_to_pixel(10.0, 10.0, &plot_area, 0.0, 10.0, 0.0, 10.0);
        assert!((p.x - 500.0).abs() < 1e-10);
        assert!((p.y - 50.0).abs() < 1e-10); // top of plot

        // Center.
        let p = ax.data_to_pixel(5.0, 5.0, &plot_area, 0.0, 10.0, 0.0, 10.0);
        assert!((p.x - 300.0).abs() < 1e-10);
        assert!((p.y - 200.0).abs() < 1e-10);
    }

    #[test]
    fn compute_data_limits_no_artists() {
        let ax = Axes::new();
        let (xmin, xmax, ymin, ymax) = ax.compute_data_limits();
        // Should return sensible defaults.
        assert!(xmin < xmax);
        assert!(ymin < ymax);
    }

    #[test]
    fn compute_data_limits_with_user_override() {
        let mut ax = Axes::new();
        ax.set_xlim(-5.0, 5.0).set_ylim(0.0, 100.0);
        let (xmin, xmax, ymin, ymax) = ax.compute_data_limits();
        assert!((xmin - (-5.0)).abs() < f64::EPSILON);
        assert!((xmax - 5.0).abs() < f64::EPSILON);
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn compute_data_limits_from_line_data() {
        let mut ax = Axes::new();
        ax.plot(vec![1.0, 5.0, 10.0], vec![2.0, 8.0, 3.0]).unwrap();
        let (xmin, xmax, ymin, ymax) = ax.compute_data_limits();
        // Should encompass data with padding.
        assert!(xmin < 1.0);
        assert!(xmax > 10.0);
        assert!(ymin < 2.0);
        assert!(ymax > 8.0);
    }
    // -- Step chart tests ---------------------------------------------------

    #[test]
    fn step_creates_artist() {
        let mut ax = Axes::new();
        let result = ax.step(vec![1.0, 2.0, 3.0], vec![1.0, 3.0, 2.0]);
        assert!(result.is_ok());
        assert!(matches!(&ax.artists[0], Artist::Step(_)));
    }

    #[test]
    fn step_length_mismatch() {
        let mut ax = Axes::new();
        let result = ax.step(vec![1.0, 2.0], vec![1.0]);
        assert!(matches!(
            result,
            Err(PlotError::SeriesLengthMismatch { .. })
        ));
    }

    #[test]
    fn step_empty_data() {
        let mut ax = Axes::new();
        let result = ax.step(Vec::<f64>::new(), Vec::<f64>::new());
        assert!(matches!(result, Err(PlotError::EmptyData)));
    }

    #[test]
    fn step_default_where() {
        let mut ax = Axes::new();
        ax.step(vec![1.0, 2.0], vec![1.0, 2.0]).unwrap();
        match &ax.artists[0] {
            Artist::Step(a) => assert!(matches!(a.where_step, StepWhere::Pre)),
            _ => panic!("expected Step"),
        }
    }

    #[test]
    fn step_color_cycle() {
        let mut ax = Axes::new();
        ax.step(vec![1.0, 2.0], vec![1.0, 2.0]).unwrap();
        ax.step(vec![1.0, 2.0], vec![1.0, 2.0]).unwrap();
        let c0 = ax.artists[0].color();
        let c1 = ax.artists[1].color();
        assert_ne!(c0, c1);
    }

    #[test]
    fn step_builder_chaining() {
        let mut ax = Axes::new();
        ax.step(vec![1.0, 2.0, 3.0], vec![1.0, 3.0, 2.0])
            .unwrap()
            .color(Color::TAB_RED)
            .width(3.0)
            .where_step(StepWhere::Post)
            .label("steps")
            .alpha(0.5);
        match &ax.artists[0] {
            Artist::Step(a) => {
                assert_eq!(a.color, Color::TAB_RED);
                assert!((a.width - 3.0).abs() < 1e-12);
                assert!(matches!(a.where_step, StepWhere::Post));
                assert_eq!(a.label.as_deref(), Some("steps"));
                assert!((a.alpha - 0.5).abs() < 1e-12);
            }
            _ => panic!("expected Step"),
        }
    }

    #[test]
    fn step_data_bounds() {
        let mut ax = Axes::new();
        ax.step(vec![1.0, 5.0, 10.0], vec![2.0, 8.0, 3.0]).unwrap();
        let (xmin, xmax, ymin, ymax) = ax.compute_data_limits();
        assert!(xmin < 1.0);
        assert!(xmax > 10.0);
        assert!(ymin < 2.0);
        assert!(ymax > 8.0);
    }

    #[test]
    fn step_legend_label() {
        let mut ax = Axes::new();
        ax.step(vec![1.0, 2.0], vec![1.0, 2.0]).unwrap().label("S");
        assert_eq!(ax.artists[0].label(), Some("S"));
    }

    #[test]
    fn step_default_alpha() {
        let mut ax = Axes::new();
        ax.step(vec![1.0, 2.0], vec![1.0, 2.0]).unwrap();
        match &ax.artists[0] {
            Artist::Step(a) => assert!((a.alpha - 1.0).abs() < 1e-12),
            _ => panic!("expected Step"),
        }
    }

    #[test]
    fn step_default_width() {
        let mut ax = Axes::new();
        ax.step(vec![1.0, 2.0], vec![1.0, 2.0]).unwrap();
        match &ax.artists[0] {
            Artist::Step(a) => assert!((a.width - 1.5).abs() < 1e-12),
            _ => panic!("expected Step"),
        }
    }

    #[test]
    fn step_mid_mode() {
        let mut ax = Axes::new();
        ax.step(vec![1.0, 2.0, 3.0], vec![1.0, 3.0, 2.0])
            .unwrap()
            .where_step(StepWhere::Mid);
        match &ax.artists[0] {
            Artist::Step(a) => assert!(matches!(a.where_step, StepWhere::Mid)),
            _ => panic!("expected Step"),
        }
    }

    // -- Stem chart tests ---------------------------------------------------

    #[test]
    fn stem_creates_artist() {
        let mut ax = Axes::new();
        let result = ax.stem(vec![1.0, 2.0, 3.0], vec![1.0, 3.0, 2.0]);
        assert!(result.is_ok());
        assert!(matches!(&ax.artists[0], Artist::Stem(_)));
    }

    #[test]
    fn stem_length_mismatch() {
        let mut ax = Axes::new();
        let result = ax.stem(vec![1.0, 2.0], vec![1.0]);
        assert!(matches!(
            result,
            Err(PlotError::SeriesLengthMismatch { .. })
        ));
    }

    #[test]
    fn stem_empty_data() {
        let mut ax = Axes::new();
        let result = ax.stem(Vec::<f64>::new(), Vec::<f64>::new());
        assert!(matches!(result, Err(PlotError::EmptyData)));
    }

    #[test]
    fn stem_default_baseline() {
        let mut ax = Axes::new();
        ax.stem(vec![1.0, 2.0], vec![1.0, 2.0]).unwrap();
        match &ax.artists[0] {
            Artist::Stem(a) => assert!((a.baseline - 0.0).abs() < 1e-12),
            _ => panic!("expected Stem"),
        }
    }

    #[test]
    fn stem_default_marker_size() {
        let mut ax = Axes::new();
        ax.stem(vec![1.0, 2.0], vec![1.0, 2.0]).unwrap();
        match &ax.artists[0] {
            Artist::Stem(a) => assert!((a.marker_size - 6.0).abs() < 1e-12),
            _ => panic!("expected Stem"),
        }
    }

    #[test]
    fn stem_builder_chaining() {
        let mut ax = Axes::new();
        ax.stem(vec![1.0, 2.0, 3.0], vec![1.0, 3.0, 2.0])
            .unwrap()
            .color(Color::TAB_GREEN)
            .baseline(1.0)
            .marker_size(8.0)
            .width(2.0)
            .label("stems")
            .alpha(0.7);
        match &ax.artists[0] {
            Artist::Stem(a) => {
                assert_eq!(a.color, Color::TAB_GREEN);
                assert!((a.baseline - 1.0).abs() < 1e-12);
                assert!((a.marker_size - 8.0).abs() < 1e-12);
                assert!((a.line_width - 2.0).abs() < 1e-12);
                assert_eq!(a.label.as_deref(), Some("stems"));
                assert!((a.alpha - 0.7).abs() < 1e-12);
            }
            _ => panic!("expected Stem"),
        }
    }

    #[test]
    fn stem_data_bounds_include_baseline() {
        let mut ax = Axes::new();
        ax.stem(vec![1.0, 5.0], vec![2.0, 8.0])
            .unwrap()
            .baseline(-5.0);
        let (_xmin, _xmax, ymin, _ymax) = ax.compute_data_limits();
        assert!(ymin < -5.0);
    }

    #[test]
    fn stem_legend_label() {
        let mut ax = Axes::new();
        ax.stem(vec![1.0, 2.0], vec![1.0, 2.0]).unwrap().label("L");
        assert_eq!(ax.artists[0].label(), Some("L"));
    }

    #[test]
    fn stem_color_cycle() {
        let mut ax = Axes::new();
        ax.stem(vec![1.0, 2.0], vec![1.0, 2.0]).unwrap();
        ax.stem(vec![1.0, 2.0], vec![1.0, 2.0]).unwrap();
        let c0 = ax.artists[0].color();
        let c1 = ax.artists[1].color();
        assert_ne!(c0, c1);
    }

    #[test]
    fn stem_alpha_default() {
        let mut ax = Axes::new();
        ax.stem(vec![1.0, 2.0], vec![1.0, 2.0]).unwrap();
        match &ax.artists[0] {
            Artist::Stem(a) => assert!((a.alpha - 1.0).abs() < 1e-12),
            _ => panic!("expected Stem"),
        }
    }

    #[test]
    fn stem_negative_baseline() {
        let mut ax = Axes::new();
        ax.stem(vec![1.0, 2.0], vec![1.0, 2.0])
            .unwrap()
            .baseline(-3.0);
        match &ax.artists[0] {
            Artist::Stem(a) => assert!((a.baseline - (-3.0)).abs() < 1e-12),
            _ => panic!("expected Stem"),
        }
    }

    // -- Text annotation tests -----------------------------------------------

    #[test]
    fn text_creates_annotation() {
        let mut ax = Axes::new();
        ax.text(1.0, 2.0, "hello");
        assert_eq!(ax.texts.len(), 1);
        assert_eq!(ax.texts[0].text, "hello");
        assert!((ax.texts[0].x - 1.0).abs() < f64::EPSILON);
        assert!((ax.texts[0].y - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn text_default_alignment() {
        let mut ax = Axes::new();
        ax.text(0.0, 0.0, "test");
        assert_eq!(ax.texts[0].ha, HAlign::Left);
        assert_eq!(ax.texts[0].va, VAlign::Baseline);
        assert!((ax.texts[0].rotation - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn text_builder_chaining() {
        let mut ax = Axes::new();
        ax.text(1.0, 2.0, "styled")
            .fontsize(14.0)
            .color(Color::TAB_RED)
            .ha(HAlign::Center)
            .va(VAlign::Top)
            .rotation(45.0);
        let t = &ax.texts[0];
        assert_eq!(t.fontsize, Some(14.0));
        assert_eq!(t.color, Some(Color::TAB_RED));
        assert_eq!(t.ha, HAlign::Center);
        assert_eq!(t.va, VAlign::Top);
        assert!((t.rotation - 45.0).abs() < f64::EPSILON);
    }

    #[test]
    fn text_does_not_affect_autoscale() {
        let mut ax = Axes::new();
        ax.plot(vec![0.0, 1.0], vec![0.0, 1.0]).unwrap();
        let limits_before = ax.compute_data_limits();
        // Place text far outside the data range.
        ax.text(100.0, 100.0, "far away");
        let limits_after = ax.compute_data_limits();
        assert_eq!(limits_before, limits_after);
    }

    #[test]
    fn multiple_texts() {
        let mut ax = Axes::new();
        ax.text(1.0, 1.0, "first");
        ax.text(2.0, 2.0, "second");
        ax.text(3.0, 3.0, "third");
        assert_eq!(ax.texts.len(), 3);
        assert_eq!(ax.texts[0].text, "first");
        assert_eq!(ax.texts[1].text, "second");
        assert_eq!(ax.texts[2].text, "third");
    }

    // -- Annotation tests (annotate with arrow) ------------------------------

    #[test]
    fn annotate_creates_annotation() {
        let mut ax = Axes::new();
        ax.annotate("peak", (1.0, 2.0), (3.0, 4.0));
        assert_eq!(ax.annotations.len(), 1);
        assert_eq!(ax.annotations[0].text, "peak");
        assert_eq!(ax.annotations[0].xy, (1.0, 2.0));
        assert_eq!(ax.annotations[0].xytext, (3.0, 4.0));
    }

    #[test]
    fn annotate_default_no_arrow() {
        let mut ax = Axes::new();
        ax.annotate("label", (0.0, 0.0), (1.0, 1.0));
        assert_eq!(ax.annotations[0].arrowstyle, ArrowStyle::None);
    }

    #[test]
    fn annotate_default_alignment() {
        let mut ax = Axes::new();
        ax.annotate("label", (0.0, 0.0), (1.0, 1.0));
        assert_eq!(ax.annotations[0].ha, HAlign::Center);
        assert_eq!(ax.annotations[0].va, VAlign::Bottom);
    }

    #[test]
    fn annotate_with_arrow() {
        let mut ax = Axes::new();
        ax.annotate("peak", (1.0, 1.0), (2.0, 2.0))
            .arrowstyle(ArrowStyle::Simple);
        assert_eq!(ax.annotations[0].arrowstyle, ArrowStyle::Simple);
    }

    #[test]
    fn annotate_with_fancy_arrow() {
        let mut ax = Axes::new();
        ax.annotate("label", (0.0, 0.0), (1.0, 1.0))
            .arrowstyle(ArrowStyle::Fancy);
        assert_eq!(ax.annotations[0].arrowstyle, ArrowStyle::Fancy);
    }

    #[test]
    fn annotate_builder_chaining() {
        let mut ax = Axes::new();
        ax.annotate("note", (1.0, 2.0), (3.0, 4.0))
            .fontsize(12.0)
            .color(Color::TAB_BLUE)
            .ha(HAlign::Right)
            .va(VAlign::Top)
            .arrowstyle(ArrowStyle::Fancy)
            .arrow_color(Color::TAB_RED);
        let a = &ax.annotations[0];
        assert_eq!(a.fontsize, Some(12.0));
        assert_eq!(a.color, Some(Color::TAB_BLUE));
        assert_eq!(a.ha, HAlign::Right);
        assert_eq!(a.va, VAlign::Top);
        assert_eq!(a.arrowstyle, ArrowStyle::Fancy);
        assert_eq!(a.arrow_color, Some(Color::TAB_RED));
    }

    #[test]
    fn annotate_does_not_affect_autoscale() {
        let mut ax = Axes::new();
        ax.plot(vec![0.0, 1.0], vec![0.0, 1.0]).unwrap();
        let limits_before = ax.compute_data_limits();
        // Annotate far outside the data range.
        ax.annotate("far", (100.0, 100.0), (200.0, 200.0));
        let limits_after = ax.compute_data_limits();
        assert_eq!(limits_before, limits_after);
    }

    #[test]
    fn multiple_annotations() {
        let mut ax = Axes::new();
        ax.annotate("a", (0.0, 0.0), (1.0, 1.0));
        ax.annotate("b", (2.0, 2.0), (3.0, 3.0));
        assert_eq!(ax.annotations.len(), 2);
    }

    #[test]
    fn text_at_plot_boundary() {
        let mut ax = Axes::new();
        ax.plot(vec![0.0, 10.0], vec![0.0, 10.0]).unwrap();
        // Place text at the boundary of the data range.
        ax.text(0.0, 0.0, "origin");
        ax.text(10.0, 10.0, "corner");
        assert_eq!(ax.texts.len(), 2);
    }

    #[test]
    fn overlapping_annotations() {
        let mut ax = Axes::new();
        // Multiple annotations at the same position -- should not panic.
        ax.annotate("one", (5.0, 5.0), (6.0, 6.0));
        ax.annotate("two", (5.0, 5.0), (6.0, 6.0));
        ax.text(6.0, 6.0, "three");
        assert_eq!(ax.annotations.len(), 2);
        assert_eq!(ax.texts.len(), 1);
    }

    #[test]
    fn new_axes_has_empty_annotations() {
        let ax = Axes::new();
        assert!(ax.texts.is_empty());
        assert!(ax.annotations.is_empty());
    }

    #[test]
    fn text_pixel_placement() {
        // Verify that text annotations are positioned using data_to_pixel.
        let ax = Axes::new();
        let plot_area = Rect::new(100.0, 50.0, 400.0, 300.0);
        // Center of data space (5, 5) with limits (0, 10, 0, 10).
        let pt = ax.data_to_pixel(5.0, 5.0, &plot_area, 0.0, 10.0, 0.0, 10.0);
        assert!((pt.x - 300.0).abs() < 1e-10);
        assert!((pt.y - 200.0).abs() < 1e-10);
    }

    #[test]
    fn annotation_pixel_placement() {
        // Verify pixel positions for both xy and xytext.
        let ax = Axes::new();
        let plot_area = Rect::new(0.0, 0.0, 100.0, 100.0);
        let target = ax.data_to_pixel(0.0, 0.0, &plot_area, 0.0, 10.0, 0.0, 10.0);
        let text_pos = ax.data_to_pixel(5.0, 5.0, &plot_area, 0.0, 10.0, 0.0, 10.0);
        // target at (0, 0) data -> pixel (0, 100) in a y-inverted 100x100 area.
        assert!((target.x - 0.0).abs() < 1e-10);
        assert!((target.y - 100.0).abs() < 1e-10);
        // text at (5, 5) data -> pixel (50, 50).
        assert!((text_pos.x - 50.0).abs() < 1e-10);
        assert!((text_pos.y - 50.0).abs() < 1e-10);
    }

    // -- Axis control tests ------------------------------------------------

    #[test]
    fn set_xlim_overrides_autoscale() {
        let mut ax = Axes::new();
        ax.plot(vec![0.0, 10.0], vec![0.0, 10.0]).unwrap();
        ax.set_xlim(2.0, 8.0);
        let (xmin, xmax, _, _) = ax.compute_data_limits();
        assert!((xmin - 2.0).abs() < f64::EPSILON);
        assert!((xmax - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn set_ylim_overrides_autoscale() {
        let mut ax = Axes::new();
        ax.plot(vec![0.0, 10.0], vec![0.0, 10.0]).unwrap();
        ax.set_ylim(-5.0, 15.0);
        let (_, _, ymin, ymax) = ax.compute_data_limits();
        assert!((ymin - (-5.0)).abs() < f64::EPSILON);
        assert!((ymax - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn set_xlim_with_min_greater_than_max() {
        let mut ax = Axes::new();
        ax.set_xlim(10.0, 2.0);
        let (xmin, xmax, _, _) = ax.compute_data_limits();
        // User limits are stored as-is; inversion happens separately.
        assert!((xmin - 10.0).abs() < f64::EPSILON);
        assert!((xmax - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn invert_xaxis_swaps_limits() {
        let mut ax = Axes::new();
        ax.plot(vec![0.0, 10.0], vec![0.0, 10.0]).unwrap();
        ax.set_xlim(0.0, 10.0);
        ax.invert_xaxis();
        let (xmin, xmax, _, _) = ax.compute_data_limits();
        // After inversion, min and max are swapped.
        assert!((xmin - 10.0).abs() < f64::EPSILON);
        assert!((xmax - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn invert_yaxis_swaps_limits() {
        let mut ax = Axes::new();
        ax.set_ylim(0.0, 100.0);
        ax.invert_yaxis();
        let (_, _, ymin, ymax) = ax.compute_data_limits();
        assert!((ymin - 100.0).abs() < f64::EPSILON);
        assert!((ymax - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn custom_xticks_appear_in_output() {
        let mut ax = Axes::new();
        ax.set_xticks(&[1.0, 2.0, 3.0]);
        let ticks = ax.resolve_xticks(0.0, 10.0);
        assert_eq!(ticks.len(), 3);
        assert!((ticks[0].value - 1.0).abs() < f64::EPSILON);
        assert!((ticks[1].value - 2.0).abs() < f64::EPSILON);
        assert!((ticks[2].value - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn custom_yticks_appear_in_output() {
        let mut ax = Axes::new();
        ax.set_yticks(&[0.0, 50.0, 100.0]);
        let ticks = ax.resolve_yticks(0.0, 100.0);
        assert_eq!(ticks.len(), 3);
        assert!((ticks[0].value - 0.0).abs() < f64::EPSILON);
        assert!((ticks[1].value - 50.0).abs() < f64::EPSILON);
        assert!((ticks[2].value - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn custom_tick_labels_override_default_format() {
        let mut ax = Axes::new();
        ax.set_xticks(&[0.0, 3.125, 6.25]);
        ax.set_xticklabels(&["0", "pi", "2pi"]);
        let ticks = ax.resolve_xticks(0.0, 7.0);
        assert_eq!(ticks.len(), 3);
        assert_eq!(ticks[0].label, "0");
        assert_eq!(ticks[1].label, "pi");
        assert_eq!(ticks[2].label, "2pi");
    }

    #[test]
    fn custom_tick_labels_partial_match() {
        // Fewer labels than ticks: missing labels fall back to format_tick.
        let mut ax = Axes::new();
        ax.set_xticks(&[1.0, 2.0, 3.0]);
        ax.set_xticklabels(&["one"]);
        let ticks = ax.resolve_xticks(0.0, 5.0);
        assert_eq!(ticks[0].label, "one");
        assert_eq!(ticks[1].label, "2");
        assert_eq!(ticks[2].label, "3");
    }

    #[test]
    fn empty_custom_ticks() {
        let mut ax = Axes::new();
        ax.set_xticks(&[]);
        let ticks = ax.resolve_xticks(0.0, 10.0);
        assert!(ticks.is_empty());
    }

    #[test]
    fn grid_visibility_toggle() {
        let mut ax = Axes::new();
        assert!(ax.show_grid.is_none());
        ax.grid(true);
        assert_eq!(ax.show_grid, Some(true));
        ax.grid(false);
        assert_eq!(ax.show_grid, Some(false));
    }

    #[test]
    fn grid_axis_setting() {
        let mut ax = Axes::new();
        assert_eq!(ax.grid_axis, GridAxis::Both);
        ax.grid_axis("x");
        assert_eq!(ax.grid_axis, GridAxis::X);
        ax.grid_axis("y");
        assert_eq!(ax.grid_axis, GridAxis::Y);
        ax.grid_axis("both");
        assert_eq!(ax.grid_axis, GridAxis::Both);
    }

    #[test]
    fn grid_alpha_setting() {
        let mut ax = Axes::new();
        ax.grid_alpha(0.5);
        assert!((ax.grid_alpha.unwrap() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn grid_alpha_clamps() {
        let mut ax = Axes::new();
        ax.grid_alpha(2.0);
        assert!((ax.grid_alpha.unwrap() - 1.0).abs() < f64::EPSILON);
        ax.grid_alpha(-0.5);
        assert!((ax.grid_alpha.unwrap() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn grid_style_setting() {
        let mut ax = Axes::new();
        ax.grid_style(crate::theme::LineStyle::Dashed);
        assert_eq!(ax.grid_style, Some(crate::theme::LineStyle::Dashed));
    }

    #[test]
    fn tick_rotation_setting() {
        let mut ax = Axes::new();
        assert!((ax.xtick_rotation - 0.0).abs() < f64::EPSILON);
        ax.tick_params_x_rotation(45.0);
        assert!((ax.xtick_rotation - 45.0).abs() < f64::EPSILON);
        ax.tick_params_y_rotation(-30.0);
        assert!((ax.ytick_rotation - (-30.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn axis_control_chaining() {
        let mut ax = Axes::new();
        ax.set_xlim(0.0, 10.0)
            .set_ylim(-1.0, 1.0)
            .invert_xaxis()
            .grid(true)
            .grid_axis("y")
            .grid_alpha(0.3)
            .grid_style(crate::theme::LineStyle::Dotted)
            .set_xticks(&[0.0, 5.0, 10.0])
            .set_xticklabels(&["start", "mid", "end"])
            .tick_params_x_rotation(90.0);

        assert_eq!(ax.xlim, Some((0.0, 10.0)));
        assert_eq!(ax.ylim, Some((-1.0, 1.0)));
        assert!(ax.x_inverted);
        assert_eq!(ax.show_grid, Some(true));
        assert_eq!(ax.grid_axis, GridAxis::Y);
        assert!((ax.grid_alpha.unwrap() - 0.3).abs() < f64::EPSILON);
        assert_eq!(ax.grid_style, Some(crate::theme::LineStyle::Dotted));
        assert_eq!(ax.custom_xticks.as_ref().unwrap().len(), 3);
        assert_eq!(ax.custom_xticklabels.as_ref().unwrap().len(), 3);
        assert!((ax.xtick_rotation - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn new_axes_has_axis_control_defaults() {
        let ax = Axes::new();
        assert_eq!(ax.grid_axis, GridAxis::Both);
        assert!(ax.grid_alpha.is_none());
        assert!(ax.grid_style.is_none());
        assert!(!ax.x_inverted);
        assert!(!ax.y_inverted);
        assert!(ax.custom_xticks.is_none());
        assert!(ax.custom_yticks.is_none());
        assert!(ax.custom_xticklabels.is_none());
        assert!(ax.custom_yticklabels.is_none());
        assert!((ax.xtick_rotation - 0.0).abs() < f64::EPSILON);
        assert!((ax.ytick_rotation - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn autoscale_not_broken_without_user_limits() {
        let mut ax = Axes::new();
        ax.plot(vec![1.0, 5.0, 10.0], vec![2.0, 8.0, 3.0]).unwrap();
        let (xmin, xmax, ymin, ymax) = ax.compute_data_limits();
        // Autoscale should still pad around data.
        assert!(xmin < 1.0);
        assert!(xmax > 10.0);
        assert!(ymin < 2.0);
        assert!(ymax > 8.0);
    }

    #[test]
    fn resolve_xticks_auto_when_not_set() {
        let ax = Axes::new();
        let ticks = ax.resolve_xticks(0.0, 10.0);
        // Should produce auto-generated ticks (non-empty).
        assert!(!ticks.is_empty());
    }

    #[test]
    fn resolve_yticks_auto_when_not_set() {
        let ax = Axes::new();
        let ticks = ax.resolve_yticks(0.0, 100.0);
        assert!(!ticks.is_empty());
    }

    // -- Log scale tests ---------------------------------------------------

    #[test]
    fn data_to_pixel_log10() {
        let mut ax = Axes::new();
        ax.set_xscale(Scale::Log10);
        let plot_area = Rect::new(100.0, 50.0, 400.0, 300.0);

        // 1.0 maps to the left edge in [1, 1000] log range.
        let p = ax.data_to_pixel(1.0, 0.0, &plot_area, 1.0, 1000.0, 0.0, 10.0);
        assert!(
            (p.x - 100.0).abs() < 1e-6,
            "log10(1)=0 should be left edge, got {}",
            p.x
        );

        // 1000 maps to the right edge.
        let p = ax.data_to_pixel(1000.0, 0.0, &plot_area, 1.0, 1000.0, 0.0, 10.0);
        assert!(
            (p.x - 500.0).abs() < 1e-6,
            "log10(1000)=3 should be right edge, got {}",
            p.x
        );

        // 10 is 1/3 of the way (log10(10)=1 out of 3 decades).
        let p = ax.data_to_pixel(10.0, 0.0, &plot_area, 1.0, 1000.0, 0.0, 10.0);
        let expected_x = 100.0 + 400.0 / 3.0;
        assert!(
            (p.x - expected_x).abs() < 1e-6,
            "log10(10)=1/3 of range, expected {}, got {}",
            expected_x,
            p.x
        );

        // 100 is 2/3 of the way.
        let p = ax.data_to_pixel(100.0, 0.0, &plot_area, 1.0, 1000.0, 0.0, 10.0);
        let expected_x = 100.0 + 400.0 * 2.0 / 3.0;
        assert!(
            (p.x - expected_x).abs() < 1e-6,
            "log10(100)=2/3 of range, expected {}, got {}",
            expected_x,
            p.x
        );
    }

    #[test]
    fn data_to_pixel_log10_y() {
        let mut ax = Axes::new();
        ax.set_yscale(Scale::Log10);
        let plot_area = Rect::new(0.0, 0.0, 400.0, 300.0);

        // ymin=1, ymax=100 (2 decades). y=10 is the midpoint.
        let p = ax.data_to_pixel(0.0, 10.0, &plot_area, 0.0, 10.0, 1.0, 100.0);
        // ty = (log10(10) - log10(1)) / (log10(100) - log10(1)) = 1/2 = 0.5
        // pixel_y = 0 + (1 - 0.5) * 300 = 150
        assert!(
            (p.y - 150.0).abs() < 1e-6,
            "log10(10) should be vertical center, got {}",
            p.y
        );
    }

    #[test]
    fn compute_data_limits_log_clamps_positive() {
        let mut ax = Axes::new();
        ax.set_xscale(Scale::Log10);
        ax.set_yscale(Scale::Log10);
        ax.plot(vec![0.1, 1.0, 10.0], vec![1.0, 10.0, 100.0])
            .unwrap();
        let (xmin, xmax, ymin, ymax) = ax.compute_data_limits();
        assert!(xmin > 0.0, "log x-min must be positive, got {}", xmin);
        assert!(xmax > xmin, "log x-max must be > x-min");
        assert!(ymin > 0.0, "log y-min must be positive, got {}", ymin);
        assert!(ymax > ymin, "log y-max must be > y-min");
    }

    #[test]
    fn compute_data_limits_log_with_zeros() {
        // Data includes zero, which is invalid for log10 -- should clamp.
        let mut ax = Axes::new();
        ax.set_xscale(Scale::Log10);
        ax.plot(vec![0.0, 1.0, 10.0], vec![1.0, 2.0, 3.0]).unwrap();
        let (xmin, xmax, _ymin, _ymax) = ax.compute_data_limits();
        assert!(
            xmin > 0.0,
            "log x-min should be clamped positive, got {}",
            xmin
        );
        assert!(xmax > xmin);
    }

    #[test]
    fn data_to_pixel_symlog() {
        let mut ax = Axes::new();
        ax.set_xscale(Scale::SymLog { linthresh: 1.0 });
        let plot_area = Rect::new(0.0, 0.0, 400.0, 300.0);

        // For a symmetric range [-100, 100], zero should be at the center.
        let p = ax.data_to_pixel(0.0, 0.0, &plot_area, -100.0, 100.0, 0.0, 1.0);
        assert!(
            (p.x - 200.0).abs() < 1e-6,
            "symlog(0) should be center for symmetric range, got {}",
            p.x
        );
    }

    #[test]
    fn set_scale_methods_return_self() {
        let mut ax = Axes::new();
        ax.set_xscale(Scale::Log10)
            .set_yscale(Scale::Log10)
            .set_title("Log plot");
        assert!(matches!(ax.xscale, Scale::Log10));
        assert!(matches!(ax.yscale, Scale::Log10));
        assert_eq!(ax.title.as_deref(), Some("Log plot"));
    }

    #[test]
    fn compute_data_limits_log_no_artists() {
        let mut ax = Axes::new();
        ax.set_xscale(Scale::Log10);
        ax.set_yscale(Scale::Log10);
        let (xmin, xmax, ymin, ymax) = ax.compute_data_limits();
        assert!(xmin > 0.0, "default log x-min must be positive");
        assert!(xmax > xmin);
        assert!(ymin > 0.0, "default log y-min must be positive");
        assert!(ymax > ymin);
    }

    #[test]
    fn data_to_pixel_log10_very_large_range() {
        let mut ax = Axes::new();
        ax.set_xscale(Scale::Log10);
        let plot_area = Rect::new(0.0, 0.0, 1000.0, 100.0);

        // 10 decades: 1e-5 to 1e5
        let p_lo = ax.data_to_pixel(1e-5, 0.0, &plot_area, 1e-5, 1e5, 0.0, 1.0);
        let p_hi = ax.data_to_pixel(1e5, 0.0, &plot_area, 1e-5, 1e5, 0.0, 1.0);
        assert!((p_lo.x - 0.0).abs() < 1e-6);
        assert!((p_hi.x - 1000.0).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // Twin axes tests
    // -----------------------------------------------------------------------

    #[test]
    fn new_axes_has_no_twin_fields() {
        let ax = Axes::new();
        assert!(!ax.is_twin());
        assert_eq!(ax.twin_side(), None);
    }

    // -----------------------------------------------------------------------
    // Colorbar tests
    // -----------------------------------------------------------------------

    #[test]
    fn colorbar_attaches_to_axes() {
        let mut ax = Axes::new();
        let cb = ax.colorbar(crate::colormap::Colormap::Viridis, 0.0, 1.0);
        cb.set_label("Test");
        assert!(ax.colorbar.is_some());
        assert_eq!(ax.colorbar.as_ref().unwrap().label.as_deref(), Some("Test"));
    }

    #[test]
    fn colorbar_replaces_previous() {
        let mut ax = Axes::new();
        ax.colorbar(crate::colormap::Colormap::Viridis, 0.0, 1.0);
        ax.colorbar(crate::colormap::Colormap::Plasma, -10.0, 10.0);
        let cb = ax.colorbar.as_ref().unwrap();
        assert_eq!(cb.cmap, crate::colormap::Colormap::Plasma);
        assert!((cb.vmin - (-10.0)).abs() < f64::EPSILON);
        assert!((cb.vmax - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn heatmap_auto_colorbar() {
        let mut ax = Axes::new();
        ax.heatmap(vec![vec![1.0, 2.0], vec![3.0, 4.0]])
            .unwrap()
            .colorbar(true);
        let auto_cb = ax.auto_colorbar_from_artists();
        assert!(auto_cb.is_some());
        let cb = auto_cb.unwrap();
        assert!((cb.vmin - 1.0).abs() < f64::EPSILON);
        assert!((cb.vmax - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn heatmap_no_auto_colorbar_by_default() {
        let mut ax = Axes::new();
        ax.heatmap(vec![vec![1.0, 2.0]]).unwrap();
        let auto_cb = ax.auto_colorbar_from_artists();
        assert!(auto_cb.is_none());
    }

    #[test]
    fn new_axes_has_no_colorbar() {
        let ax = Axes::new();
        assert!(ax.colorbar.is_none());
    }
}
