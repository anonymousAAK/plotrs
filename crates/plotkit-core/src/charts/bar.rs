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
}
