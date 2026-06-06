//! Bar chart builder methods.
//!
//! Provides a fluent builder API for configuring [`BarArtist`] instances.
//! Each method returns `&mut Self`, allowing calls to be chained together
//! for concise, readable chart construction.

use crate::artist::BarArtist;
use crate::primitives::Color;

impl BarArtist {
    /// Sets the bar color.
    ///
    /// Applies the given [`Color`] to every bar rendered by this artist.
    ///
    /// # Arguments
    ///
    /// * `color` - The [`Color`] to fill each bar with.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    /// Sets the legend label.
    ///
    /// When a legend is displayed on the figure, this label will appear
    /// next to the color swatch for this bar series. Passing a new value
    /// overwrites any previously set label.
    ///
    /// # Arguments
    ///
    /// * `label` - A string slice that will be stored as the legend entry.
    pub fn label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }

    /// Sets the opacity.
    ///
    /// The value is clamped to the range `[0.0, 1.0]`, where `0.0` is fully
    /// transparent and `1.0` is fully opaque.
    ///
    /// # Arguments
    ///
    /// * `alpha` - The desired opacity level.
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Sets the bar width as a fraction of the category spacing (0.0 to 1.0).
    ///
    /// Smaller values produce thinner bars with more whitespace between them,
    /// while larger values make the bars wider. The value is clamped to the
    /// range `[0.1, 1.0]` so that bars are never invisibly thin nor overlap
    /// their neighbours.
    ///
    /// # Arguments
    ///
    /// * `width` - The fraction of available category space each bar should occupy.
    pub fn bar_width(&mut self, width: f64) -> &mut Self {
        self.bar_width = width.clamp(0.1, 1.0);
        self
    }

    /// Sets the bottom offset for each bar (for stacking).
    ///
    /// When stacking multiple bar series, set `bottom` for each subsequent
    /// series to the cumulative heights of the series below it. Each bar
    /// then starts at `bottom[i]` instead of `0.0` and extends to
    /// `bottom[i] + height[i]`.
    ///
    /// # Arguments
    ///
    /// * `bottom` - A vector of base offsets, one per bar. Must have the same
    ///   length as the heights vector.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Stack series B on top of series A:
    /// ax.bar(cats, &heights_a)?.label("A");
    /// ax.bar(cats, &heights_b)?.bottom(heights_a.clone()).label("B");
    /// ```
    pub fn bottom(&mut self, bottom: Vec<f64>) -> &mut Self {
        self.bottom = Some(bottom);
        self
    }

    /// Sets the per-bar position offset for grouped (side-by-side) bars.
    ///
    /// Each bar's category center is shifted by `offset[i]` units along the
    /// category axis. This is useful for placing multiple bar series next to
    /// each other within the same categories.
    ///
    /// For most grouped-bar use cases, prefer [`Axes::bar_group`] which
    /// computes offsets automatically.
    ///
    /// # Arguments
    ///
    /// * `offset` - A vector of position offsets, one per bar.
    pub fn offset(&mut self, offset: Vec<f64>) -> &mut Self {
        self.offset = Some(offset);
        self
    }
}
