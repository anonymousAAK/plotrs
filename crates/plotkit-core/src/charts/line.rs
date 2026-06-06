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
use crate::decimate::DecimateMethod;
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

    /// Enables LTTB decimation with the given point threshold.
    ///
    /// When the data series length exceeds `threshold`, the rendering
    /// pipeline downsamples the data using the Largest Triangle Three
    /// Buckets algorithm before drawing. This dramatically improves
    /// rendering performance for large datasets (100k+ points) with
    /// negligible visual impact.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// ax.plot(&x, &y)?.decimate(1000);
    /// ```
    pub fn decimate(&mut self, threshold: usize) -> &mut Self {
        self.decimate = Some((threshold, DecimateMethod::Lttb));
        self
    }

    /// Enables decimation with a specific method and point threshold.
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
        self.decimate = Some((threshold, method));
        self
    }
}
