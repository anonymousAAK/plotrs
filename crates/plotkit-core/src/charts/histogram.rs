//! Histogram chart builder methods and binning utilities.
//!
//! This module extends [`HistArtist`] with a fluent API for configuring
//! histogram properties, and provides the [`compute_bins`] utility function
//! for computing equal-width bin edges and counts from raw data.
//!
//! Since [`Axes::hist`] returns `Result<&mut HistArtist>`, the builder
//! methods can be chained directly on the return value:
//!
//! ```ignore
//! ax.hist(&data, 20)?
//!     .color(Color::TAB_BLUE)
//!     .label("Distribution")
//!     .alpha(0.7)
//!     .density(true);
//! ```
//!
//! The [`compute_bins`] function is typically called internally when
//! constructing a [`HistArtist`], but it is public so that users can
//! pre-compute bin edges and counts for custom workflows.
//!
//! [`Axes::hist`]: crate::axes::Axes::hist

use crate::artist::HistArtist;
use crate::primitives::Color;

impl HistArtist {
    /// Sets the bar fill color for every bin in the histogram.
    ///
    /// Accepts any [`Color`] value, which can be constructed from RGB
    /// components, hex strings, or named color constants.
    ///
    /// # Arguments
    ///
    /// * `color` - The [`Color`] to fill each histogram bar with.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.color(Color::TAB_BLUE);
    /// ```
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    /// Sets the legend label for this histogram.
    ///
    /// When a label is set, the histogram will appear in the legend if one
    /// is displayed on the axes. Pass an empty string or omit this call to
    /// exclude the histogram from the legend. Calling this method again
    /// overwrites any previously set label.
    ///
    /// # Arguments
    ///
    /// * `label` - A string slice that will be stored as the legend entry.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.label("Scores");
    /// ```
    pub fn label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }

    /// Sets the opacity (0.0 = fully transparent, 1.0 = fully opaque).
    ///
    /// The value is clamped to the `[0.0, 1.0]` range. The default opacity
    /// is determined by the active theme (typically `0.7` for histograms so
    /// that overlapping distributions remain visible).
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

    /// Controls whether the histogram displays probability density instead
    /// of raw counts.
    ///
    /// When `density` is `true`, the `counts` vector is normalized so that
    /// the total area under the histogram integrates to 1.0. Each bin's
    /// value becomes `count / (total * bin_width)`. This is useful for
    /// comparing distributions with different sample sizes or overlaying a
    /// probability density function.
    ///
    /// When `density` is `false` (the default), the `counts` vector stores
    /// raw frequency counts.
    ///
    /// # Arguments
    ///
    /// * `density` - If `true`, normalize the histogram to unit area.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.density(true); // show probability density
    /// ```
    pub fn density(&mut self, density: bool) -> &mut Self {
        self.density = density;
        if density {
            self.recompute_density();
        }
        self
    }

    /// Normalizes the `counts` vector so that the total area under the
    /// histogram equals 1.0.
    ///
    /// Each bin value is divided by `total_count * bin_width`, where
    /// `total_count` is the sum of all counts and `bin_width` is the width
    /// of the corresponding bin. This method is called automatically by
    /// [`density`](Self::density) when density mode is enabled.
    fn recompute_density(&mut self) {
        let total: f64 = self.counts.iter().sum();
        if total > 0.0 && self.bin_edges.len() > 1 {
            for (i, count) in self.counts.iter_mut().enumerate() {
                let bin_width = self.bin_edges[i + 1] - self.bin_edges[i];
                *count /= total * bin_width;
            }
        }
    }
}

/// Computes equal-width bin edges and counts for a histogram.
///
/// Given a slice of data values and a desired number of bins, this function
/// determines the bin edges and counts the number of data points that fall
/// into each bin. Non-finite values (`NaN`, `+Inf`, `-Inf`) are silently
/// ignored.
///
/// # Bin placement
///
/// Bins are equal-width and span the range `[min, max]` of the finite
/// values, where `min` and `max` are the smallest and largest finite values
/// in `data`. The i-th bin covers the half-open interval
/// `[edge[i], edge[i+1])`, except for the last bin which is closed on both
/// sides `[edge[n-1], edge[n]]` to include the maximum value.
///
/// # Single-value case
///
/// When all finite values are identical (i.e. `max == min`), the range is
/// expanded to `[min - 0.5, max + 0.5]` so that the single value falls
/// within the bin and the histogram has a visible width.
///
/// # Returns
///
/// A tuple `(edges, counts)` where:
///
/// * `edges` is a `Vec<f64>` of length `num_bins + 1` containing the sorted
///   bin edges.
/// * `counts` is a `Vec<f64>` of length `num_bins` containing the number of
///   data points in each bin.
///
/// If `data` contains no finite values or `num_bins` is zero, both vectors
/// are returned empty.
///
/// # Examples
///
/// ```
/// use plotkit_core::charts::histogram::compute_bins;
///
/// let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let (edges, counts) = compute_bins(&data, 5);
///
/// assert_eq!(edges.len(), 6);    // 5 bins + 1
/// assert_eq!(counts.len(), 5);   // one count per bin
///
/// // Every value lands in exactly one bin.
/// let total: f64 = counts.iter().sum();
/// assert_eq!(total, 5.0);
/// ```
pub fn compute_bins(data: &[f64], num_bins: usize) -> (Vec<f64>, Vec<f64>) {
    let finite: Vec<f64> = data.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.is_empty() || num_bins == 0 {
        return (vec![], vec![]);
    }

    let min = finite.iter().copied().fold(f64::INFINITY, f64::min);
    let max = finite.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    // Handle single-value case: expand range so the histogram has visible width.
    let (min, max) = if (max - min).abs() < f64::EPSILON {
        (min - 0.5, max + 0.5)
    } else {
        (min, max)
    };

    let bin_width = (max - min) / num_bins as f64;
    let edges: Vec<f64> = (0..=num_bins)
        .map(|i| min + i as f64 * bin_width)
        .collect();

    // Count values in each bin.
    let mut counts = vec![0.0f64; num_bins];
    for &val in &finite {
        let bin = ((val - min) / bin_width).floor() as usize;
        // Clamp to the last bin so that the maximum value (which lands exactly
        // on the right edge) is included in the final bin.
        let bin = bin.min(num_bins - 1);
        counts[bin] += 1.0;
    }

    (edges, counts)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::Series;

    /// Tolerance for floating-point comparisons.
    const TOL: f64 = 1e-12;

    /// Returns true if `a` and `b` are within `TOL` of each other.
    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < TOL
    }

    // -----------------------------------------------------------------------
    // compute_bins — basic behavior
    // -----------------------------------------------------------------------

    #[test]
    fn basic_five_values_five_bins() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (edges, counts) = compute_bins(&data, 5);

        assert_eq!(edges.len(), 6);
        assert_eq!(counts.len(), 5);

        // Total count should equal the number of data points.
        let total: f64 = counts.iter().sum();
        assert!(approx_eq(total, 5.0));

        // First edge should be the minimum value.
        assert!(approx_eq(edges[0], 1.0));
        // Last edge should be the maximum value.
        assert!(approx_eq(edges[5], 5.0));
    }

    #[test]
    fn all_values_in_one_bin() {
        let data = vec![1.0, 1.5, 1.8, 1.9, 2.0];
        let (edges, counts) = compute_bins(&data, 1);

        assert_eq!(edges.len(), 2);
        assert_eq!(counts.len(), 1);
        assert!(approx_eq(counts[0], 5.0));
        assert!(approx_eq(edges[0], 1.0));
        assert!(approx_eq(edges[1], 2.0));
    }

    #[test]
    fn even_distribution_across_bins() {
        // 10 values evenly spaced from 0 to 9, placed into 5 bins.
        let data: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let (edges, counts) = compute_bins(&data, 5);

        assert_eq!(edges.len(), 6);
        assert_eq!(counts.len(), 5);

        // With values 0..9 and 5 equal-width bins of width 1.8:
        // every value should land in exactly one bin.
        let total: f64 = counts.iter().sum();
        assert!(approx_eq(total, 10.0));
    }

    // -----------------------------------------------------------------------
    // compute_bins — edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn empty_data_returns_empty() {
        let (edges, counts) = compute_bins(&[], 10);
        assert!(edges.is_empty());
        assert!(counts.is_empty());
    }

    #[test]
    fn zero_bins_returns_empty() {
        let data = vec![1.0, 2.0, 3.0];
        let (edges, counts) = compute_bins(&data, 0);
        assert!(edges.is_empty());
        assert!(counts.is_empty());
    }

    #[test]
    fn all_nan_returns_empty() {
        let data = vec![f64::NAN, f64::NAN, f64::NAN];
        let (edges, counts) = compute_bins(&data, 5);
        assert!(edges.is_empty());
        assert!(counts.is_empty());
    }

    #[test]
    fn non_finite_values_are_ignored() {
        let data = vec![f64::NAN, 1.0, f64::INFINITY, 2.0, f64::NEG_INFINITY, 3.0];
        let (edges, counts) = compute_bins(&data, 3);

        assert_eq!(edges.len(), 4);
        assert_eq!(counts.len(), 3);

        // Only the three finite values (1.0, 2.0, 3.0) should be counted.
        let total: f64 = counts.iter().sum();
        assert!(approx_eq(total, 3.0));
    }

    #[test]
    fn single_value_expands_range() {
        let data = vec![5.0, 5.0, 5.0];
        let (edges, counts) = compute_bins(&data, 2);

        assert_eq!(edges.len(), 3);
        assert_eq!(counts.len(), 2);

        // Range should be expanded to [4.5, 5.5].
        assert!(approx_eq(edges[0], 4.5));
        assert!(approx_eq(edges[2], 5.5));

        // All values should be counted.
        let total: f64 = counts.iter().sum();
        assert!(approx_eq(total, 3.0));
    }

    #[test]
    fn single_data_point_single_bin() {
        let data = vec![42.0];
        let (edges, counts) = compute_bins(&data, 1);

        assert_eq!(edges.len(), 2);
        assert_eq!(counts.len(), 1);
        assert!(approx_eq(edges[0], 41.5));
        assert!(approx_eq(edges[1], 42.5));
        assert!(approx_eq(counts[0], 1.0));
    }

    #[test]
    fn maximum_value_lands_in_last_bin() {
        // The maximum value sits exactly on the right edge of the last bin.
        // It must be included in the last bin, not lost.
        let data = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let (_, counts) = compute_bins(&data, 4);

        let total: f64 = counts.iter().sum();
        assert!(approx_eq(total, 5.0));

        // Specifically, 4.0 (the max) should be in the last bin.
        assert!(counts[3] >= 1.0);
    }

    // -----------------------------------------------------------------------
    // compute_bins — structural invariants
    // -----------------------------------------------------------------------

    #[test]
    fn edges_are_monotonically_increasing() {
        let data: Vec<f64> = (0..100).map(|i| (i as f64) * 0.37 - 10.0).collect();
        let (edges, _) = compute_bins(&data, 15);

        for window in edges.windows(2) {
            assert!(
                window[1] > window[0],
                "edges not monotonically increasing: {} >= {}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn bins_are_equal_width() {
        let data = vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0];
        let (edges, _) = compute_bins(&data, 5);

        let expected_width = (50.0 - 0.0) / 5.0;
        for window in edges.windows(2) {
            let width = window[1] - window[0];
            assert!(
                approx_eq(width, expected_width),
                "bin width {} differs from expected {}",
                width,
                expected_width
            );
        }
    }

    #[test]
    fn total_count_equals_finite_data_length() {
        let data = vec![
            1.0, 2.0, 3.0, 4.0, 5.0,
            f64::NAN, f64::INFINITY, f64::NEG_INFINITY,
        ];
        let (_, counts) = compute_bins(&data, 3);

        let total: f64 = counts.iter().sum();
        assert!(approx_eq(total, 5.0));
    }

    #[test]
    fn large_number_of_bins() {
        let data: Vec<f64> = (0..1000).map(|i| i as f64).collect();
        let (edges, counts) = compute_bins(&data, 500);

        assert_eq!(edges.len(), 501);
        assert_eq!(counts.len(), 500);

        let total: f64 = counts.iter().sum();
        assert!(approx_eq(total, 1000.0));
    }

    #[test]
    fn negative_values() {
        let data = vec![-10.0, -5.0, -3.0, -1.0, 0.0];
        let (edges, counts) = compute_bins(&data, 2);

        assert_eq!(edges.len(), 3);
        assert_eq!(counts.len(), 2);

        assert!(approx_eq(edges[0], -10.0));
        assert!(approx_eq(edges[2], 0.0));

        let total: f64 = counts.iter().sum();
        assert!(approx_eq(total, 5.0));
    }

    #[test]
    fn mixed_positive_and_negative() {
        let data = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
        let (edges, counts) = compute_bins(&data, 4);

        assert_eq!(edges.len(), 5);
        assert_eq!(counts.len(), 4);

        assert!(approx_eq(edges[0], -2.0));
        assert!(approx_eq(edges[4], 2.0));

        let total: f64 = counts.iter().sum();
        assert!(approx_eq(total, 5.0));
    }

    // -----------------------------------------------------------------------
    // HistArtist builder methods
    // -----------------------------------------------------------------------

    /// Helper: build a minimal `HistArtist` for builder method tests.
    fn sample_hist() -> HistArtist {
        HistArtist {
            data: Series::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
            bins: 3,
            bin_edges: vec![1.0, 3.0, 5.0, 7.0],
            counts: vec![2.0, 2.0, 2.0],
            color: Color::TAB_BLUE,
            label: None,
            alpha: 1.0,
            density: false,
        }
    }

    #[test]
    fn builder_color() {
        let mut h = sample_hist();
        h.color(Color::TAB_RED);
        assert_eq!(h.color, Color::TAB_RED);
    }

    #[test]
    fn builder_label() {
        let mut h = sample_hist();
        assert!(h.label.is_none());
        h.label("Distribution");
        assert_eq!(h.label.as_deref(), Some("Distribution"));
    }

    #[test]
    fn builder_label_overwrite() {
        let mut h = sample_hist();
        h.label("first");
        h.label("second");
        assert_eq!(h.label.as_deref(), Some("second"));
    }

    #[test]
    fn builder_alpha_clamps_to_range() {
        let mut h = sample_hist();

        h.alpha(0.5);
        assert!(approx_eq(h.alpha, 0.5));

        h.alpha(-1.0);
        assert!(approx_eq(h.alpha, 0.0));

        h.alpha(2.0);
        assert!(approx_eq(h.alpha, 1.0));
    }

    #[test]
    fn builder_alpha_boundaries() {
        let mut h = sample_hist();

        h.alpha(0.0);
        assert!(approx_eq(h.alpha, 0.0));

        h.alpha(1.0);
        assert!(approx_eq(h.alpha, 1.0));
    }

    #[test]
    fn builder_density_normalizes_counts() {
        let mut h = sample_hist();
        // counts = [2.0, 2.0, 2.0], bin_edges = [1.0, 3.0, 5.0, 7.0]
        // total = 6.0, each bin_width = 2.0
        // density[i] = count[i] / (total * bin_width) = 2.0 / (6.0 * 2.0) = 1/6
        h.density(true);

        assert!(h.density);
        let expected = 2.0 / (6.0 * 2.0);
        for &c in &h.counts {
            assert!(
                approx_eq(c, expected),
                "expected density {expected}, got {c}"
            );
        }
    }

    #[test]
    fn builder_density_false_does_not_modify_counts() {
        let mut h = sample_hist();
        let original_counts = h.counts.clone();
        h.density(false);
        assert!(!h.density);
        assert_eq!(h.counts, original_counts);
    }

    #[test]
    fn builder_density_with_zero_total() {
        let mut h = HistArtist {
            data: Series::new(vec![]),
            bins: 2,
            bin_edges: vec![0.0, 1.0, 2.0],
            counts: vec![0.0, 0.0],
            color: Color::BLACK,
            label: None,
            alpha: 1.0,
            density: false,
        };
        // Should not panic or produce NaN when total is zero.
        h.density(true);
        assert!(h.counts.iter().all(|c| c.is_finite()));
    }

    #[test]
    fn builder_density_area_integrates_to_one() {
        // Use compute_bins to get realistic counts, then enable density.
        let data: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
        let (edges, counts) = compute_bins(&data, 10);
        let mut h = HistArtist {
            data: Series::new(data),
            bins: 10,
            bin_edges: edges,
            counts,
            color: Color::TAB_BLUE,
            label: None,
            alpha: 1.0,
            density: false,
        };

        h.density(true);

        // The total area (sum of density * bin_width) should be 1.0.
        let area: f64 = h
            .counts
            .iter()
            .enumerate()
            .map(|(i, &d)| d * (h.bin_edges[i + 1] - h.bin_edges[i]))
            .sum();
        assert!(
            (area - 1.0).abs() < 1e-10,
            "density area should be 1.0, got {area}"
        );
    }

    #[test]
    fn builder_chaining() {
        let mut h = sample_hist();
        h.color(Color::TAB_GREEN)
            .label("Test")
            .alpha(0.8);

        assert_eq!(h.color, Color::TAB_GREEN);
        assert_eq!(h.label.as_deref(), Some("Test"));
        assert!(approx_eq(h.alpha, 0.8));
    }
}
