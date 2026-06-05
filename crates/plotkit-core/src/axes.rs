//! The Axes container -- holds chart data and renders a single subplot.
//!
//! `Axes` is the central workhorse of the plotting library. It stores a list of
//! [`Artist`] values (one per chart call), manages auto-scaling, tick generation,
//! and drives the full ten-step render pipeline that produces the final visual
//! output within a subplot rectangle.

use crate::artist::*;
use crate::error::{PlotError, Result};
use crate::layout::{self, LayoutConfig};
use crate::legend::{self, LegendEntry, SwatchKind};
use crate::primitives::*;
use crate::renderer::Renderer;
use crate::scale::Scale;
use crate::series::{IntoCategories, IntoSeries};
use crate::theme::{Loc, Marker, Theme, TickDirection};
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
    /// Whether the legend should be drawn.
    pub(crate) show_legend: bool,
    /// Where to place the legend.
    pub(crate) legend_loc: Loc,
    /// Per-axes theme override. `None` means use the figure theme.
    pub(crate) theme_override: Option<Theme>,
    /// Tracks the current position in the color cycle so that each successive
    /// artist receives a distinct color automatically.
    color_index: usize,
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
            show_legend: false,
            legend_loc: Loc::Best,
            theme_override: None,
            color_index: 0,
        }
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
}

// ---------------------------------------------------------------------------
// Configuration methods (builder-style, return &mut Self)
// ---------------------------------------------------------------------------

impl Axes {
    /// Sets the title displayed above the axes area.
    pub fn set_title(&mut self, title: &str) -> &mut Self {
        self.title = Some(title.to_string());
        self
    }

    /// Sets the x-axis label.
    pub fn set_xlabel(&mut self, label: &str) -> &mut Self {
        self.xlabel = Some(label.to_string());
        self
    }

    /// Sets the y-axis label.
    pub fn set_ylabel(&mut self, label: &str) -> &mut Self {
        self.ylabel = Some(label.to_string());
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
        let theme = self.theme_override.as_ref().unwrap_or(fig_theme);

        // Step 1: Compute data limits.
        let (xmin, xmax, ymin, ymax) = self.compute_data_limits();

        // Step 2: Generate ticks.
        let xticks = ticks::generate_ticks(xmin, xmax, DEFAULT_TICK_COUNT, &self.xscale);
        let yticks = ticks::generate_ticks(ymin, ymax, DEFAULT_TICK_COUNT, &self.yscale);

        // Step 3: Compute layout (reserve space for labels, ticks, title).
        let mut layout_config = LayoutConfig::new(bounds.width, bounds.height);
        layout_config.has_title = self.title.is_some();
        layout_config.has_xlabel = self.xlabel.is_some();
        layout_config.has_ylabel = self.ylabel.is_some();
        layout_config.has_legend = self.show_legend;





        let layout_result = layout::compute_layout(&layout_config);

        // Offset the computed plot area by the bounds position.
        let plot_area = Rect::new(
            bounds.x + layout_result.plot_area.x,
            bounds.y + layout_result.plot_area.y,
            layout_result.plot_area.width,
            layout_result.plot_area.height,
        );

        // Step 4: Draw axes background.
        let bg_path = Path::rect(plot_area);
        renderer.fill_path(&bg_path, &Paint::new(theme.axes_background), Affine::IDENTITY);

        // Step 5: Draw grid (behind data).
        if self.show_grid.unwrap_or(theme.show_grid) {
            self.draw_grid(renderer, &plot_area, &xticks, &yticks, xmin, xmax, ymin, ymax, theme);
        }

        // Step 6: Clip to plot area and draw each artist.
        let clip_path = Path::rect(plot_area);
        renderer.push_clip(&clip_path, Affine::IDENTITY);
        for artist in &self.artists {
            self.draw_artist(renderer, artist, &plot_area, xmin, xmax, ymin, ymax, theme);
        }
        renderer.pop_clip();

        // Step 7: Draw spines.
        self.draw_spines(renderer, &plot_area, theme);

        // Step 8: Draw ticks and tick labels.
        self.draw_ticks(renderer, &plot_area, &xticks, &yticks, xmin, xmax, ymin, ymax, theme);

        // Step 9: Draw axis labels and title.
        self.draw_labels(renderer, &plot_area, &bounds, theme);

        // Step 10: Draw legend if enabled.
        if self.show_legend {
            self.draw_legend(renderer, &plot_area, theme);
        }
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
            }
        }

        // Handle the case where there are no artists or no finite data.
        if !x_lo.is_finite() || !x_hi.is_finite() {
            x_lo = 0.0;
            x_hi = 1.0;
        }
        if !y_lo.is_finite() || !y_hi.is_finite() {
            y_lo = 0.0;
            y_hi = 1.0;
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
        let x_pad = (x_hi - x_lo) * AUTOSCALE_PAD;
        let y_pad = (y_hi - y_lo) * AUTOSCALE_PAD;
        x_lo -= x_pad;
        x_hi += x_pad;
        y_lo -= y_pad;
        y_hi += y_pad;

        // Apply user-set limits, overriding auto-scale.
        if let Some((lo, hi)) = self.xlim {
            x_lo = lo;
            x_hi = hi;
        }
        if let Some((lo, hi)) = self.ylim {
            y_lo = lo;
            y_hi = hi;
        }

        (x_lo, x_hi, y_lo, y_hi)
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
        let paint = Paint::new(theme.grid_color);
        let stroke = Stroke::new(theme.grid_width);

        // Vertical grid lines at each x-tick.
        for tick in xticks {
            let pt = self.data_to_pixel(tick.value, ymin, plot_area, xmin, xmax, ymin, ymax);
            let mut path = Path::new();
            path.move_to(pt.x, plot_area.y);
            path.line_to(pt.x, plot_area.bottom());
            renderer.stroke_path(&path, &paint, &stroke, Affine::IDENTITY);
        }

        // Horizontal grid lines at each y-tick.
        for tick in yticks {
            let pt = self.data_to_pixel(xmin, tick.value, plot_area, xmin, xmax, ymin, ymax);
            let mut path = Path::new();
            path.move_to(plot_area.x, pt.y);
            path.line_to(plot_area.right(), pt.y);
            renderer.stroke_path(&path, &paint, &stroke, Affine::IDENTITY);
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
            Artist::Scatter(a) => self.draw_scatter(renderer, a, plot_area, xmin, xmax, ymin, ymax, theme),
            Artist::Bar(a) => self.draw_bar(renderer, a, plot_area, xmin, xmax, ymin, ymax),
            Artist::Histogram(a) => self.draw_hist(renderer, a, plot_area, xmin, xmax, ymin, ymax),
            Artist::FillBetween(a) => self.draw_fill_between(renderer, a, plot_area, xmin, xmax, ymin, ymax),
        }
    }

    /// Draws a line chart: builds a polyline from data points and strokes it.
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

        let mut path = Path::new();
        let first = self.data_to_pixel(
            artist.x.data[0],
            artist.y.data[0],
            plot_area,
            xmin, xmax, ymin, ymax,
        );
        path.move_to(first.x, first.y);

        for i in 1..artist.x.len() {
            let pt = self.data_to_pixel(
                artist.x.data[i],
                artist.y.data[i],
                plot_area,
                xmin, xmax, ymin, ymax,
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
        let color = artist.color.with_alpha((artist.alpha * 255.0) as u8);
        let paint = Paint::new(color);
        let radius = artist.size / 2.0;

        for i in 0..artist.x.len() {
            let pt = self.data_to_pixel(
                artist.x.data[i],
                artist.y.data[i],
                plot_area,
                xmin, xmax, ymin, ymax,
            );

            let marker_path = match artist.marker {
                Marker::Circle | Marker::Point => Path::circle(pt, radius),
                Marker::Square => {
                    Path::rect(Rect::new(pt.x - radius, pt.y - radius, radius * 2.0, radius * 2.0))
                }
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
                        let angle = std::f64::consts::FRAC_PI_2
                            + j as f64 * std::f64::consts::PI / 5.0;
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
                let cat_center = ymin + (i as f64 + 0.5) * cat_step;
                let value = artist.heights.data[i];

                let left_val = 0.0_f64.min(value);
                let right_val = 0.0_f64.max(value);

                let p_left = self.data_to_pixel(left_val, cat_center - bar_half, plot_area, xmin, xmax, ymin, ymax);
                let p_right = self.data_to_pixel(right_val, cat_center + bar_half, plot_area, xmin, xmax, ymin, ymax);

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
                let cat_center = xmin + (i as f64 + 0.5) * cat_step;
                let value = artist.heights.data[i];

                let bottom_val = 0.0_f64.min(value);
                let top_val = 0.0_f64.max(value);

                let p_bl = self.data_to_pixel(cat_center - bar_half, bottom_val, plot_area, xmin, xmax, ymin, ymax);
                let p_tr = self.data_to_pixel(cat_center + bar_half, top_val, plot_area, xmin, xmax, ymin, ymax);

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
            xmin, xmax, ymin, ymax,
        );
        path.move_to(first.x, first.y);
        for i in 1..n {
            let pt = self.data_to_pixel(
                artist.x.data[i],
                artist.y1.data[i],
                plot_area,
                xmin, xmax, ymin, ymax,
            );
            path.line_to(pt.x, pt.y);
        }

        // Backward pass along y2 (in reverse order).
        for i in (0..n).rev() {
            let pt = self.data_to_pixel(
                artist.x.data[i],
                artist.y2.data[i],
                plot_area,
                xmin, xmax, ymin, ymax,
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
    fn draw_spines(
        &self,
        renderer: &mut impl Renderer,
        plot_area: &Rect,
        theme: &Theme,
    ) {
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

        let label_style = TextStyle {
            size: theme.tick_label_size,
            color: theme.text_color,
            weight: FontWeight::Normal,
            family: theme.font_family.clone(),
            halign: HAlign::Center,
            valign: VAlign::Top,
        };

        // Tick direction offset: outward goes away from plot, inward goes in.
        let outward = matches!(theme.tick_direction, TickDirection::Outward);

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

            // Draw tick label.
            let label_y = if outward {
                y_base + tick_len + 2.0
            } else {
                y_base + 2.0
            };
            renderer.draw_text(
                &tick.label,
                Point::new(x, label_y),
                &label_style,
                Affine::IDENTITY,
            );
        }

        // --- Y-axis ticks (left) ---
        let y_label_style = TextStyle {
            halign: HAlign::Right,
            valign: VAlign::Middle,
            ..label_style.clone()
        };

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

            // Draw tick label.
            let label_x = if outward {
                x_base - tick_len - 3.0
            } else {
                x_base - 3.0
            };
            renderer.draw_text(
                &tick.label,
                Point::new(label_x, y),
                &y_label_style,
                Affine::IDENTITY,
            );
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

    /// Draws the legend box showing labeled artists.
    ///
    /// Builds [`LegendEntry`] items from the axes' artists and delegates to
    /// [`legend::draw_legend`] for measurement, positioning, and rendering.
    fn draw_legend(
        &self,
        renderer: &mut impl Renderer,
        plot_area: &Rect,
        theme: &Theme,
    ) {
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
                };
                label.map(|l| LegendEntry { label: l.to_string(), color, swatch })
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
            Err(PlotError::SeriesLengthMismatch { expected: 2, got: 1 })
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
        assert!(matches!(result, Err(PlotError::SeriesLengthMismatch { .. })));
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
}
