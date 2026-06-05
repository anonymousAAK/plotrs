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
}
