//! Fill-between chart builder methods.
//!
//! This module extends [`FillBetweenArtist`] with a fluent API for configuring
//! the visual properties of a filled region between two curves. Since
//! [`Axes::fill_between`] returns `Result<&mut FillBetweenArtist>`, these
//! builder methods can be chained directly on the return value:
//!
//! ```ignore
//! ax.fill_between(&x, &y1, &y2)?
//!     .color(Color::rgb(78, 121, 167))
//!     .label("Confidence interval")
//!     .alpha(0.3);
//! ```

use crate::artist::FillBetweenArtist;
use crate::primitives::Color;

impl FillBetweenArtist {
    /// Sets the fill color.
    ///
    /// Applies the given [`Color`] to the entire shaded region between the
    /// two boundary curves. Any previously set color is replaced.
    ///
    /// # Arguments
    ///
    /// * `color` - The [`Color`] to fill the region with.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.color(Color::rgb(78, 121, 167));
    /// ```
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    /// Sets the legend label.
    ///
    /// When a legend is displayed on the figure, this label will appear
    /// next to the color swatch for this fill region. Passing a new value
    /// overwrites any previously set label.
    ///
    /// # Arguments
    ///
    /// * `label` - A string slice that will be stored as the legend entry.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.label("Confidence interval");
    /// ```
    pub fn label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }

    /// Sets the opacity (0.0 = fully transparent, 1.0 = fully opaque).
    ///
    /// The value is clamped to the `[0.0, 1.0]` range. The default opacity
    /// is `1.0`. Lower values are useful for layering multiple fill regions
    /// so that underlying data remains visible.
    ///
    /// # Arguments
    ///
    /// * `alpha` - The desired opacity level.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.alpha(0.3); // 30% opaque, letting background show through
    /// ```
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }
}
