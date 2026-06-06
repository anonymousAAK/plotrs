//! Waterfall chart builder methods.
//!
//! Provides a fluent builder API for configuring [`WaterfallArtist`] instances.
//! Each method returns `&mut Self`, allowing calls to be chained together
//! for concise, readable chart construction.

use crate::artist::WaterfallArtist;
use crate::primitives::Color;

impl WaterfallArtist {
    /// Sets the color for bars representing positive (increasing) changes.
    ///
    /// # Arguments
    ///
    /// * `color` - The [`Color`] to fill increase bars with.
    pub fn increase_color(&mut self, color: Color) -> &mut Self {
        self.increase_color = color;
        self
    }

    /// Sets the color for bars representing negative (decreasing) changes.
    ///
    /// # Arguments
    ///
    /// * `color` - The [`Color`] to fill decrease bars with.
    pub fn decrease_color(&mut self, color: Color) -> &mut Self {
        self.decrease_color = color;
        self
    }

    /// Sets the color for total bars (bars drawn from zero).
    ///
    /// # Arguments
    ///
    /// * `color` - The [`Color`] to fill total bars with.
    pub fn total_color(&mut self, color: Color) -> &mut Self {
        self.total_color = color;
        self
    }

    /// Enables or disables connector lines between consecutive bar tops.
    ///
    /// When enabled (the default), thin horizontal lines are drawn from the
    /// end of each bar to the start of the next, visually connecting the
    /// running total across the chart.
    ///
    /// # Arguments
    ///
    /// * `enabled` - `true` to draw connector lines, `false` to hide them.
    pub fn connector_lines(&mut self, enabled: bool) -> &mut Self {
        self.connector_lines = enabled;
        self
    }

    /// Enables or disables value labels displayed on each bar.
    ///
    /// When enabled, each bar shows its change value (or total value for
    /// total bars) as a text label positioned at the bar's top edge.
    ///
    /// # Arguments
    ///
    /// * `enabled` - `true` to show value labels, `false` to hide them.
    pub fn show_values(&mut self, enabled: bool) -> &mut Self {
        self.show_values = enabled;
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

    /// Sets the legend label.
    ///
    /// When a legend is displayed on the figure, this label will appear
    /// next to the color swatch for this waterfall series. Passing a new
    /// value overwrites any previously set label.
    ///
    /// # Arguments
    ///
    /// * `label` - A string slice that will be stored as the legend entry.
    pub fn label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }

    /// Marks the given bar indices as "total" bars.
    ///
    /// Total bars are drawn from zero to the current cumulative sum, rather
    /// than showing the incremental change. Typically the first and/or last
    /// bar in a waterfall chart are marked as totals.
    ///
    /// # Arguments
    ///
    /// * `indices` - A slice of zero-based category indices to treat as totals.
    pub fn total(&mut self, indices: &[usize]) -> &mut Self {
        self.total_indices = indices.to_vec();
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

    /// Computes the running cumulative sum from the change values.
    ///
    /// For total bars, the cumulative sum equals the bar's value directly.
    /// For non-total bars, each value is added to the running total.
    ///
    /// Returns a vector of cumulative sums, one per category.
    pub fn cumulative_sums(&self) -> Vec<f64> {
        let n = self.values.len();
        let mut cumsum = Vec::with_capacity(n);
        let mut running = 0.0;
        for i in 0..n {
            if self.total_indices.contains(&i) {
                running = self.values.data[i];
            } else {
                running += self.values.data[i];
            }
            cumsum.push(running);
        }
        cumsum
    }

    /// Returns the bar base and top positions for each category.
    ///
    /// For non-total bars: base is the previous cumulative sum, top is the
    /// current cumulative sum (base + change).
    ///
    /// For total bars: base is 0, top is the cumulative sum value.
    ///
    /// Returns a vector of `(base, top)` tuples.
    pub fn bar_positions(&self) -> Vec<(f64, f64)> {
        let cumsum = self.cumulative_sums();
        let n = cumsum.len();
        let mut positions = Vec::with_capacity(n);

        for i in 0..n {
            if self.total_indices.contains(&i) {
                positions.push((0.0, cumsum[i]));
            } else {
                let base = if i == 0 { 0.0 } else { cumsum[i - 1] };
                positions.push((base, cumsum[i]));
            }
        }
        positions
    }

    /// Returns the connector line positions as pairs of y-values.
    ///
    /// Each connector runs from the top of bar `i` to the start of bar `i+1`.
    /// Returns a vector of `(from_y, to_x_index)` tuples where `from_y` is
    /// the y-value the connector line runs at.
    pub fn connector_positions(&self) -> Vec<(usize, f64)> {
        if !self.connector_lines {
            return Vec::new();
        }
        let cumsum = self.cumulative_sums();
        let n = cumsum.len();
        if n < 2 {
            return Vec::new();
        }
        let mut connectors = Vec::with_capacity(n - 1);
        for (i, &val) in cumsum.iter().enumerate().take(n - 1) {
            connectors.push((i, val));
        }
        connectors
    }
}
