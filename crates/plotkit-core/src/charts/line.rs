//! Line chart builder methods.
//!
//! This module extends [`LineArtist`] with a fluent API for configuring
//! line chart properties. Since [`Axes::plot`] returns `Result<&mut LineArtist>`,
//! these builder methods can be chained directly on the return value:
//!
//! ```ignore
//! ax.plot(&x, &y)?
//!     .color(Color::rgb(0.2, 0.4, 0.8))
//!     .width(2.0)
//!     .style(LineStyle::Dashed)
//!     .label("Series A")
//!     .alpha(0.9);
//! ```

use crate::artist::LineArtist;
use crate::decimate::{DecimateMethod, DecimateMode};
use crate::primitives::Color;
use crate::theme::LineStyle;

impl LineArtist {
    /// Sets the line color.
    ///
    /// Accepts any [`Color`] value, which can be constructed from RGB components,
    /// hex strings, or named color constants.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.color(Color::rgb(1.0, 0.0, 0.0)); // red
    /// ```
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    /// Sets the line width in pixels.
    ///
    /// A width of `1.0` is the default hairline width. Values below `1.0` may
    /// produce sub-pixel rendering depending on the backend.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.width(2.5);
    /// ```
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width;
        self
    }

    /// Sets the line style (solid, dashed, dotted, dash-dot).
    ///
    /// The [`LineStyle`] enum defines the available stroke patterns. The default
    /// is [`LineStyle::Solid`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.style(LineStyle::Dashed);
    /// ```
    pub fn style(&mut self, style: LineStyle) -> &mut Self {
        self.style = style;
        self
    }

    /// Sets the legend label for this line.
    ///
    /// When a label is set, the line will appear in the legend if one is
    /// displayed on the axes. Pass an empty string or omit this call to
    /// exclude the line from the legend.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.label("Temperature");
    /// ```
    pub fn label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }

    /// Sets the opacity (0.0 = fully transparent, 1.0 = fully opaque).
    ///
    /// The value is clamped to the `[0.0, 1.0]` range. The default opacity
    /// is `1.0`.
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

    /// Enables LTTB decimation with the given explicit point threshold.
    ///
    /// When the data series length exceeds `threshold`, the rendering
    /// pipeline downsamples the data using the Largest Triangle Three
    /// Buckets algorithm before drawing. This dramatically improves
    /// rendering performance for large datasets (100k+ points) with
    /// negligible visual impact.
    ///
    /// This overrides the default [`DecimateMode::Auto`] behavior with an
    /// explicit threshold. To disable decimation entirely, use
    /// [`no_decimate`](Self::no_decimate).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// ax.plot(&x, &y)?.decimate(1000);
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
    /// ax.plot(&x, &y)?.decimate_with(1000, DecimateMethod::MinMax);
    /// ```
    pub fn decimate_with(&mut self, threshold: usize, method: DecimateMethod) -> &mut Self {
        self.decimate = DecimateMode::Explicit(threshold, method);
        self
    }

    /// Disables decimation entirely, drawing every point.
    ///
    /// By default large series are auto-decimated via
    /// [`DecimateMode::Auto`]. Call this when exact point-for-point rendering
    /// is required regardless of series size.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// ax.plot(&x, &y)?.no_decimate();
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
    fn line_defaults_to_auto_decimate() {
        let mut ax = make_axes();
        let artist = ax.plot([0.0, 1.0, 2.0], [0.0, 1.0, 2.0]).unwrap();
        assert_eq!(artist.decimate, DecimateMode::Auto);
    }

    #[test]
    fn line_decimate_sets_explicit_lttb() {
        let mut ax = make_axes();
        let artist = ax.plot([0.0, 1.0], [0.0, 1.0]).unwrap();
        artist.decimate(1000);
        assert_eq!(
            artist.decimate,
            DecimateMode::Explicit(1000, DecimateMethod::Lttb)
        );
    }

    #[test]
    fn line_decimate_with_sets_explicit_method() {
        let mut ax = make_axes();
        let artist = ax.plot([0.0, 1.0], [0.0, 1.0]).unwrap();
        artist.decimate_with(500, DecimateMethod::MinMax);
        assert_eq!(
            artist.decimate,
            DecimateMode::Explicit(500, DecimateMethod::MinMax)
        );
    }

    #[test]
    fn line_no_decimate_disables() {
        let mut ax = make_axes();
        let artist = ax.plot([0.0, 1.0], [0.0, 1.0]).unwrap();
        artist.no_decimate();
        assert_eq!(artist.decimate, DecimateMode::Off);
    }

    #[test]
    fn line_auto_kicks_in_above_threshold() {
        let n = DEFAULT_DECIMATE_THRESHOLD + 1000;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| (v * 0.01).sin()).collect();
        let indices = DecimateMode::Auto.resolve_indices(&x, &y);
        assert_eq!(indices.len(), DEFAULT_DECIMATE_THRESHOLD);
        assert_eq!(*indices.first().unwrap(), 0);
        assert_eq!(*indices.last().unwrap(), n - 1);
    }

    #[test]
    fn line_auto_no_op_at_or_below_threshold() {
        let n = DEFAULT_DECIMATE_THRESHOLD;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let y = x.clone();
        let indices = DecimateMode::Auto.resolve_indices(&x, &y);
        assert_eq!(indices.len(), n);
        assert_eq!(indices.first().copied(), Some(0));
        assert_eq!(indices.last().copied(), Some(n - 1));
    }

    #[test]
    fn line_no_decimate_draws_all_points_even_when_huge() {
        let n = DEFAULT_DECIMATE_THRESHOLD * 2;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let y = x.clone();
        let indices = DecimateMode::Off.resolve_indices(&x, &y);
        assert_eq!(indices.len(), n);
    }

    #[test]
    fn line_explicit_overrides_auto_threshold() {
        // 6000 points: auto would cut to 5000, but explicit 100 wins.
        let n = 6000;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| (v * 0.05).sin()).collect();
        let indices = DecimateMode::Explicit(100, DecimateMethod::Lttb).resolve_indices(&x, &y);
        assert_eq!(indices.len(), 100);
        assert_eq!(*indices.first().unwrap(), 0);
        assert_eq!(*indices.last().unwrap(), n - 1);
    }
}
