//! Box plot builder methods.
//!
//! Provides a fluent builder API for configuring [`BoxPlotArtist`] instances.
//! Each method returns `&mut Self`, allowing calls to be chained together
//! for concise, readable chart construction.

use crate::artist::BoxPlotArtist;
use crate::primitives::Color;

/// Summary statistics for a single group in a box plot.
///
/// Holds the quartiles, whisker endpoints, and outliers computed from
/// raw data. The whisker limits are determined by the interquartile
/// range multiplied by a configurable factor (default 1.5).
#[derive(Debug, Clone)]
pub struct BoxStats {
    /// First quartile (25th percentile).
    pub q1: f64,
    /// Median (50th percentile).
    pub median: f64,
    /// Third quartile (75th percentile).
    pub q3: f64,
    /// Lower whisker endpoint (lowest datum within Q1 - factor * IQR).
    pub whisker_low: f64,
    /// Upper whisker endpoint (highest datum within Q3 + factor * IQR).
    pub whisker_high: f64,
    /// Data points falling outside the whisker range.
    pub outliers: Vec<f64>,
}

/// Computes the linear-interpolation percentile of a sorted slice.
///
/// Uses the same interpolation method as NumPy's default (`linear`).
/// `p` must be in the range [0.0, 1.0].
fn percentile(sorted: &[f64], p: f64) -> f64 {
    assert!(!sorted.is_empty(), "percentile requires non-empty data");
    if sorted.len() == 1 {
        return sorted[0];
    }
    let idx = p * (sorted.len() - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = lo + 1;
    let frac = idx - lo as f64;
    if hi >= sorted.len() {
        sorted[sorted.len() - 1]
    } else {
        sorted[lo] * (1.0 - frac) + sorted[hi] * frac
    }
}

/// Computes box plot statistics for a single dataset.
///
/// The `whisker_factor` controls how far the whiskers extend beyond Q1 and Q3,
/// as a multiple of the interquartile range (IQR). The standard value is 1.5.
/// Points outside the whisker range are classified as outliers.
pub fn compute_stats(data: &[f64], whisker_factor: f64) -> BoxStats {
    let mut sorted: Vec<f64> = data.iter().copied().filter(|v| v.is_finite()).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if sorted.is_empty() {
        return BoxStats {
            q1: 0.0,
            median: 0.0,
            q3: 0.0,
            whisker_low: 0.0,
            whisker_high: 0.0,
            outliers: vec![],
        };
    }

    let q1 = percentile(&sorted, 0.25);
    let median = percentile(&sorted, 0.5);
    let q3 = percentile(&sorted, 0.75);
    let iqr = q3 - q1;

    let fence_low = q1 - whisker_factor * iqr;
    let fence_high = q3 + whisker_factor * iqr;

    // Whisker endpoints are the most extreme data points within the fences.
    let whisker_low = sorted
        .iter()
        .copied()
        .find(|&v| v >= fence_low)
        .unwrap_or(q1);
    let whisker_high = sorted
        .iter()
        .rev()
        .copied()
        .find(|&v| v <= fence_high)
        .unwrap_or(q3);

    let outliers: Vec<f64> = sorted
        .iter()
        .copied()
        .filter(|&v| v < whisker_low || v > whisker_high)
        .collect();

    BoxStats {
        q1,
        median,
        q3,
        whisker_low,
        whisker_high,
        outliers,
    }
}

impl BoxPlotArtist {
    /// Sets the box fill color.
    ///
    /// Applies the given [`Color`] to every box rendered by this artist.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    /// Sets the legend label.
    ///
    /// When a legend is displayed on the figure, this label will appear
    /// next to the color swatch for this box plot.
    pub fn label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }

    /// Sets the opacity.
    ///
    /// The value is clamped to the range [0.0, 1.0], where 0.0 is fully
    /// transparent and 1.0 is fully opaque.
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Sets the box width as a fraction of the category spacing.
    ///
    /// Smaller values produce thinner boxes with more whitespace between them.
    /// The value is clamped to [0.1, 1.0].
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.box_width = width.clamp(0.1, 1.0);
        self
    }

    /// Controls whether outlier points are drawn.
    ///
    /// When `true` (the default), data points beyond the whiskers are rendered
    /// as individual dots. When `false`, outliers are hidden.
    pub fn show_outliers(&mut self, show: bool) -> &mut Self {
        self.show_outliers = show;
        self
    }

    /// Sets the whisker extent factor.
    ///
    /// Whiskers extend from Q1 and Q3 by this factor multiplied by the
    /// interquartile range (IQR). The standard value is 1.5. Changing this
    /// value recomputes the box statistics from the stored raw data.
    pub fn whisker_factor(&mut self, factor: f64) -> &mut Self {
        self.whisker_iq_factor = factor;
        // Recompute stats with the new factor.
        self.stats = self
            .raw_data
            .iter()
            .map(|d| compute_stats(d, factor))
            .collect();
        self
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_single_value() {
        assert!((percentile(&[5.0], 0.5) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn percentile_two_values() {
        let data = [2.0, 8.0];
        assert!((percentile(&data, 0.0) - 2.0).abs() < f64::EPSILON);
        assert!((percentile(&data, 1.0) - 8.0).abs() < f64::EPSILON);
        assert!((percentile(&data, 0.5) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn percentile_interpolation() {
        let data = [1.0, 2.0, 3.0, 4.0];
        assert!((percentile(&data, 0.25) - 1.75).abs() < 1e-10);
        assert!((percentile(&data, 0.5) - 2.5).abs() < 1e-10);
        assert!((percentile(&data, 0.75) - 3.25).abs() < 1e-10);
    }

    #[test]
    fn compute_stats_basic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let stats = compute_stats(&data, 1.5);
        assert!((stats.median - 5.5).abs() < 1e-10);
        assert!((stats.q1 - 3.25).abs() < 1e-10);
        assert!((stats.q3 - 7.75).abs() < 1e-10);
        assert!(stats.outliers.is_empty());
    }

    #[test]
    fn compute_stats_with_outliers() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 100.0];
        let stats = compute_stats(&data, 1.5);
        assert!(!stats.outliers.is_empty());
        assert!(stats.outliers.contains(&100.0));
    }

    #[test]
    fn compute_stats_empty_data() {
        let stats = compute_stats(&[], 1.5);
        assert!((stats.q1 - 0.0).abs() < f64::EPSILON);
        assert!((stats.median - 0.0).abs() < f64::EPSILON);
        assert!((stats.q3 - 0.0).abs() < f64::EPSILON);
        assert!(stats.outliers.is_empty());
    }

    #[test]
    fn compute_stats_single_value() {
        let stats = compute_stats(&[42.0], 1.5);
        assert!((stats.q1 - 42.0).abs() < f64::EPSILON);
        assert!((stats.median - 42.0).abs() < f64::EPSILON);
        assert!((stats.q3 - 42.0).abs() < f64::EPSILON);
        assert!(stats.outliers.is_empty());
    }

    #[test]
    fn compute_stats_two_values() {
        let stats = compute_stats(&[3.0, 7.0], 1.5);
        assert!((stats.median - 5.0).abs() < 1e-10);
        assert!((stats.q1 - 4.0).abs() < 1e-10);
        assert!((stats.q3 - 6.0).abs() < 1e-10);
    }

    #[test]
    fn compute_stats_nan_filtered() {
        let data = vec![f64::NAN, 1.0, 2.0, 3.0, f64::NAN];
        let stats = compute_stats(&data, 1.5);
        assert!((stats.median - 2.0).abs() < 1e-10);
    }

    #[test]
    fn compute_stats_whisker_endpoints() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let stats = compute_stats(&data, 1.5);
        assert!((stats.whisker_low - 1.0).abs() < 1e-10);
        assert!((stats.whisker_high - 10.0).abs() < 1e-10);
    }

    #[test]
    fn compute_stats_custom_whisker_factor() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 100.0];
        let stats_narrow = compute_stats(&data, 0.5);
        let stats_wide = compute_stats(&data, 3.0);
        assert!(stats_narrow.outliers.len() >= stats_wide.outliers.len());
    }

    #[test]
    fn builder_color() {
        let mut artist = sample_boxplot_artist();
        artist.color(Color::TAB_RED);
        assert_eq!(artist.color, Color::TAB_RED);
    }

    #[test]
    fn builder_label() {
        let mut artist = sample_boxplot_artist();
        artist.label("my boxplot");
        assert_eq!(artist.label.as_deref(), Some("my boxplot"));
    }

    #[test]
    fn builder_alpha() {
        let mut artist = sample_boxplot_artist();
        artist.alpha(0.5);
        assert!((artist.alpha - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_alpha_clamped() {
        let mut artist = sample_boxplot_artist();
        artist.alpha(2.0);
        assert!((artist.alpha - 1.0).abs() < f64::EPSILON);
        artist.alpha(-1.0);
        assert!((artist.alpha - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_width() {
        let mut artist = sample_boxplot_artist();
        artist.width(0.5);
        assert!((artist.box_width - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_width_clamped() {
        let mut artist = sample_boxplot_artist();
        artist.width(0.01);
        assert!((artist.box_width - 0.1).abs() < f64::EPSILON);
        artist.width(5.0);
        assert!((artist.box_width - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_show_outliers() {
        let mut artist = sample_boxplot_artist();
        artist.show_outliers(false);
        assert!(!artist.show_outliers);
    }

    #[test]
    fn builder_whisker_factor_recomputes() {
        let mut artist = sample_boxplot_artist();
        let old_stats = artist.stats.clone();
        artist.whisker_factor(0.5);
        assert!(
            artist.stats[0].whisker_high <= old_stats[0].whisker_high
                || artist.stats[0].whisker_low >= old_stats[0].whisker_low
                || artist.stats[0].outliers.len() >= old_stats[0].outliers.len()
        );
    }

    #[test]
    fn data_bounds_single_group() {
        let artist = sample_boxplot_artist();
        let (xmin, xmax, ymin, ymax) = artist.data_bounds();
        assert!((xmin - (-0.5)).abs() < f64::EPSILON);
        assert!((xmax - 0.5).abs() < f64::EPSILON);
        assert!(ymin <= artist.stats[0].whisker_low);
        assert!(ymax >= artist.stats[0].whisker_high);
    }

    #[test]
    fn data_bounds_multiple_groups() {
        let raw = vec![vec![1.0, 2.0, 3.0], vec![10.0, 20.0, 30.0]];
        let stats: Vec<BoxStats> = raw.iter().map(|d| compute_stats(d, 1.5)).collect();
        let artist = BoxPlotArtist {
            stats,
            labels: vec!["A".to_string(), "B".to_string()],
            color: Color::TAB_BLUE,
            label: None,
            alpha: 1.0,
            box_width: 0.5,
            show_outliers: true,
            whisker_iq_factor: 1.5,
            raw_data: raw,
        };
        let (xmin, xmax, ymin, ymax) = artist.data_bounds();
        assert!((xmin - (-0.5)).abs() < f64::EPSILON);
        assert!((xmax - 1.5).abs() < f64::EPSILON);
        assert!(ymin <= 1.0);
        assert!(ymax >= 30.0);
    }

    /// Helper to create a sample BoxPlotArtist for builder tests.
    fn sample_boxplot_artist() -> BoxPlotArtist {
        let raw = vec![vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]];
        let stats = vec![compute_stats(&raw[0], 1.5)];
        BoxPlotArtist {
            stats,
            labels: vec!["Group 1".to_string()],
            color: Color::TAB_BLUE,
            label: None,
            alpha: 1.0,
            box_width: 0.5,
            show_outliers: true,
            whisker_iq_factor: 1.5,
            raw_data: raw,
        }
    }
}
