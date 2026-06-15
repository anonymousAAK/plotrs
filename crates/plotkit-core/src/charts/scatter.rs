//! Scatter chart builder methods.
//!
//! This module extends [`ScatterArtist`] with a fluent API for configuring
//! scatter plot properties. Since [`Axes::scatter`] returns
//! `Result<&mut ScatterArtist>`, these builder methods can be chained
//! directly on the return value:
//!
//! ```ignore
//! ax.scatter(&x, &y)?
//!     .color(Color::TAB_ORANGE)
//!     .marker(Marker::Diamond)
//!     .size(8.0)
//!     .label("Observations")
//!     .alpha(0.7);
//! ```

use crate::artist::ScatterArtist;
use crate::colormap::Colormap;
use crate::decimate::{DecimateMethod, DecimateMode};
use crate::primitives::Color;
use crate::theme::Marker;

impl ScatterArtist {
    /// Sets the marker color.
    ///
    /// Applies the given [`Color`] to every marker rendered by this artist,
    /// unless per-point colors have been set via [`colors`](Self::colors).
    ///
    /// # Arguments
    ///
    /// * `color` - The [`Color`] to fill each marker with.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.color(Color::TAB_RED);
    /// ```
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    /// Sets the marker shape.
    ///
    /// The [`Marker`] enum defines the available shapes (circle, square,
    /// triangle, diamond, plus, cross, star, point). The default is
    /// [`Marker::Circle`].
    ///
    /// # Arguments
    ///
    /// * `marker` - The [`Marker`] variant to use for every data point.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.marker(Marker::Triangle);
    /// ```
    pub fn marker(&mut self, marker: Marker) -> &mut Self {
        self.marker = marker;
        self
    }

    /// Sets the marker size in pixels.
    ///
    /// This controls the diameter of each marker glyph. Larger values
    /// produce more prominent data points; smaller values suit dense
    /// scatter plots.
    ///
    /// # Arguments
    ///
    /// * `size` - The marker diameter in device-independent pixels.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.size(10.0);
    /// ```
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = size;
        self
    }

    /// Sets the legend label for this scatter series.
    ///
    /// When a label is set, the scatter series will appear in the legend
    /// if one is displayed on the axes. Pass an empty string or omit this
    /// call to exclude the series from the legend.
    ///
    /// # Arguments
    ///
    /// * `label` - A string slice that will be stored as the legend entry.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.label("Measurements");
    /// ```
    pub fn label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }

    /// Sets the opacity (0.0 = fully transparent, 1.0 = fully opaque).
    ///
    /// The value is clamped to the `[0.0, 1.0]` range. The default opacity
    /// is determined by the active theme (typically `0.8`).
    ///
    /// # Arguments
    ///
    /// * `alpha` - The desired opacity level.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.alpha(0.5); // 50% transparent
    /// ```
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Sets per-point colors, overriding the single uniform color.
    ///
    /// When set, each data point is rendered with its corresponding color
    /// from the vector. The length of `colors` must equal the number of
    /// data points (`x.len()` and `y.len()`). This is commonly used to
    /// map a third variable to a colormap.
    ///
    /// Calling [`color`](Self::color) after this method does not clear the
    /// per-point colors; the per-point colors take precedence during
    /// rendering.
    ///
    /// # Arguments
    ///
    /// * `colors` - A vector of [`Color`] values, one per data point.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.colors(vec![Color::TAB_BLUE, Color::TAB_RED, Color::TAB_GREEN]);
    /// ```
    pub fn colors(&mut self, colors: Vec<Color>) -> &mut Self {
        self.colors = Some(colors);
        self
    }

    /// Sets per-point scalar values for colormap-driven coloring.
    ///
    /// When combined with [`cmap`](Self::cmap), each scalar value is mapped
    /// through the colormap to produce a per-point color. The length of `c`
    /// must equal the number of data points. This takes precedence over
    /// both the uniform [`color`](Self::color) and [`colors`](Self::colors).
    ///
    /// # Arguments
    ///
    /// * `c` - A vector of scalar values, one per data point.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.c(vec![0.0, 0.5, 1.0]).cmap(Colormap::Viridis);
    /// ```
    pub fn c(&mut self, c: Vec<f64>) -> &mut Self {
        self.c = Some(c);
        self
    }

    /// Sets the colormap used to map `c` values to colors.
    ///
    /// Must be used together with [`c`](Self::c) to have any effect. When
    /// both are set, the scatter plot renders each point with a color
    /// determined by mapping its `c` value through the given colormap.
    ///
    /// # Arguments
    ///
    /// * `cmap` - The [`Colormap`] variant to use.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.c(values).cmap(Colormap::Plasma);
    /// ```
    pub fn cmap(&mut self, cmap: Colormap) -> &mut Self {
        self.cmap = Some(cmap);
        self
    }

    /// Enables LTTB decimation with the given explicit point threshold.
    ///
    /// When the data series length exceeds `threshold`, the rendering pipeline
    /// downsamples the points using the Largest Triangle Three Buckets
    /// algorithm before drawing. Per-point styling (`colors`, `c`) stays
    /// synchronized with the surviving points.
    ///
    /// This overrides the default [`DecimateMode::Auto`] behavior with an
    /// explicit threshold. To disable decimation entirely, use
    /// [`no_decimate`](Self::no_decimate).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// ax.scatter(&x, &y)?.decimate(2000);
    /// ```
    pub fn decimate(&mut self, threshold: usize) -> &mut Self {
        self.decimate = DecimateMode::Explicit(threshold, DecimateMethod::Lttb);
        self
    }

    /// Enables decimation with a specific method and explicit point threshold.
    ///
    /// Available methods:
    /// - [`DecimateMethod::Lttb`] — best visual fidelity (default)
    /// - [`DecimateMethod::MinMax`] — fastest, preserves peaks/troughs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// ax.scatter(&x, &y)?.decimate_with(2000, DecimateMethod::MinMax);
    /// ```
    pub fn decimate_with(&mut self, threshold: usize, method: DecimateMethod) -> &mut Self {
        self.decimate = DecimateMode::Explicit(threshold, method);
        self
    }

    /// Disables decimation entirely, drawing every point.
    ///
    /// By default large series are auto-decimated via [`DecimateMode::Auto`].
    /// Call this when every marker must be rendered regardless of series size.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// ax.scatter(&x, &y)?.no_decimate();
    /// ```
    pub fn no_decimate(&mut self) -> &mut Self {
        self.decimate = DecimateMode::Off;
        self
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::axes::Axes;
    use crate::decimate::{DecimateMethod, DecimateMode, DEFAULT_DECIMATE_THRESHOLD};

    fn make_axes() -> Axes {
        Axes::new()
    }

    #[test]
    fn scatter_defaults_to_auto_decimate() {
        let mut ax = make_axes();
        let artist = ax.scatter(&[0.0, 1.0, 2.0], &[0.0, 1.0, 2.0]).unwrap();
        assert_eq!(artist.decimate, DecimateMode::Auto);
    }

    #[test]
    fn scatter_decimate_sets_explicit_lttb() {
        let mut ax = make_axes();
        let artist = ax.scatter(&[0.0, 1.0], &[0.0, 1.0]).unwrap();
        artist.decimate(2000);
        assert_eq!(
            artist.decimate,
            DecimateMode::Explicit(2000, DecimateMethod::Lttb)
        );
    }

    #[test]
    fn scatter_decimate_with_sets_explicit_method() {
        let mut ax = make_axes();
        let artist = ax.scatter(&[0.0, 1.0], &[0.0, 1.0]).unwrap();
        artist.decimate_with(750, DecimateMethod::MinMax);
        assert_eq!(
            artist.decimate,
            DecimateMode::Explicit(750, DecimateMethod::MinMax)
        );
    }

    #[test]
    fn scatter_no_decimate_disables() {
        let mut ax = make_axes();
        let artist = ax.scatter(&[0.0, 1.0], &[0.0, 1.0]).unwrap();
        artist.no_decimate();
        assert_eq!(artist.decimate, DecimateMode::Off);
    }

    #[test]
    fn scatter_auto_kicks_in_above_threshold() {
        let n = DEFAULT_DECIMATE_THRESHOLD + 2500;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| (v * 0.02).cos()).collect();
        let indices = DecimateMode::Auto.resolve_indices(&x, &y);
        assert_eq!(indices.len(), DEFAULT_DECIMATE_THRESHOLD);
        assert_eq!(*indices.first().unwrap(), 0);
        assert_eq!(*indices.last().unwrap(), n - 1);
    }

    #[test]
    fn scatter_auto_no_op_below_threshold() {
        let n = 100;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let y = x.clone();
        let indices = DecimateMode::Auto.resolve_indices(&x, &y);
        assert_eq!(indices.len(), n);
        assert_eq!(indices, (0..n).collect::<Vec<_>>());
    }

    #[test]
    fn scatter_no_decimate_keeps_all_points() {
        let n = DEFAULT_DECIMATE_THRESHOLD * 3;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let y = x.clone();
        let indices = DecimateMode::Off.resolve_indices(&x, &y);
        assert_eq!(indices.len(), n);
        assert_eq!(*indices.first().unwrap(), 0);
        assert_eq!(*indices.last().unwrap(), n - 1);
    }

    #[test]
    fn scatter_explicit_minmax_overrides_auto() {
        let n = 8000;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| (v * 0.03).sin()).collect();
        let indices =
            DecimateMode::Explicit(300, DecimateMethod::MinMax).resolve_indices(&x, &y);
        assert!(indices.len() <= 300);
        assert_eq!(*indices.first().unwrap(), 0);
        assert_eq!(*indices.last().unwrap(), n - 1);
    }
}
