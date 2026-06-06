//! Violin plot builder methods and kernel density estimation.
//!
//! Provides a fluent builder API for configuring [`ViolinArtist`] instances,
//! along with a Gaussian kernel density estimation (KDE) function used to
//! compute the shape of each violin.

use crate::artist::ViolinArtist;
use crate::primitives::Color;

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

/// Computes the standard deviation of a sorted slice of finite values.
///
/// Returns 0.0 for slices with fewer than 2 elements.
fn std_dev(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if n < 2 {
        return 0.0;
    }
    let mean: f64 = sorted.iter().sum::<f64>() / n as f64;
    let variance = sorted.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    variance.sqrt()
}

/// Computes the bandwidth for kernel density estimation using Silverman's rule.
///
/// The bandwidth is calculated as:
/// `h = 0.9 * min(std, IQR / 1.34) * n^(-1/5)`
///
/// Falls back to a small positive value when the data has zero spread
/// (e.g. all identical values or a single data point).
pub fn silverman_bandwidth(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if n < 2 {
        return 1.0;
    }
    let sd = std_dev(sorted);
    let q1 = percentile(sorted, 0.25);
    let q3 = percentile(sorted, 0.75);
    let iqr = q3 - q1;

    let spread = if sd > 0.0 && iqr > 0.0 {
        sd.min(iqr / 1.34)
    } else if sd > 0.0 {
        sd
    } else if iqr > 0.0 {
        iqr / 1.34
    } else {
        // All values identical -- use a small positive bandwidth.
        return 1.0;
    };

    0.9 * spread * (n as f64).powf(-0.2)
}

/// Evaluates a Gaussian kernel density estimate at a set of points.
///
/// For each point in `eval_points`, the density is estimated by summing
/// Gaussian kernels centered on each data value, scaled by `bandwidth`.
///
/// # Arguments
/// * `data` - The source data values.
/// * `bandwidth` - The smoothing bandwidth (standard deviation of each kernel).
/// * `eval_points` - The x-values at which to evaluate the density.
///
/// # Returns
/// A vector of density values, one per eval point.
pub fn gaussian_kde(data: &[f64], bandwidth: f64, eval_points: &[f64]) -> Vec<f64> {
    let n = data.len() as f64;
    let inv_bw = 1.0 / bandwidth;
    let norm = 1.0 / (n * bandwidth * (2.0 * std::f64::consts::PI).sqrt());

    eval_points
        .iter()
        .map(|&x| {
            let sum: f64 = data
                .iter()
                .map(|&xi| {
                    let u = (x - xi) * inv_bw;
                    (-0.5 * u * u).exp()
                })
                .sum();
            sum * norm
        })
        .collect()
}

impl ViolinArtist {
    /// Sets the fill color.
    ///
    /// Applies the given [`Color`] to every violin rendered by this artist.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    /// Sets the legend label.
    ///
    /// When a legend is displayed on the figure, this label will appear
    /// next to the color swatch for this violin plot.
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

    /// Sets the maximum width of each violin shape.
    ///
    /// The value is clamped to [0.1, 2.0]. This controls the visual width
    /// of the widest part of the violin.
    pub fn widths(&mut self, widths: f64) -> &mut Self {
        self.widths = widths.clamp(0.1, 2.0);
        self
    }

    /// Sets the x-positions for each violin.
    ///
    /// When `None` (the default), violins are placed at 1.0, 2.0, 3.0, etc.
    pub fn positions(&mut self, positions: Vec<f64>) -> &mut Self {
        self.positions = Some(positions);
        self
    }

    /// Controls whether the median line is drawn.
    ///
    /// When `true` (the default), a horizontal line is drawn at the median
    /// of each dataset inside the violin shape.
    pub fn show_median(&mut self, show: bool) -> &mut Self {
        self.show_median = show;
        self
    }

    /// Controls whether Q1/Q3 quartile lines are drawn.
    ///
    /// When `true` (the default), thin horizontal lines are drawn at the
    /// first and third quartiles inside the violin shape.
    pub fn show_quartiles(&mut self, show: bool) -> &mut Self {
        self.show_quartiles = show;
        self
    }

    /// Sets the KDE bandwidth manually.
    ///
    /// Overrides the automatically computed Silverman's rule bandwidth.
    /// A value of 0.0 or negative resets to automatic bandwidth selection.
    pub fn bw_method(&mut self, bw: f64) -> &mut Self {
        self.bw_method = bw;
        self
    }

    /// Returns the x-position for a given violin index.
    ///
    /// Uses custom positions if set, otherwise defaults to 1-based indexing.
    pub fn position_for(&self, index: usize) -> f64 {
        self.positions
            .as_ref()
            .and_then(|p| p.get(index).copied())
            .unwrap_or(index as f64 + 1.0)
    }

    /// Computes the data-space bounding box `(xmin, xmax, ymin, ymax)`.
    ///
    /// The x-bounds span from the leftmost position minus half the max width
    /// to the rightmost position plus half the max width. The y-bounds span
    /// from the minimum data value to the maximum data value across all
    /// datasets. Falls back to `(0.0, 1.0, 0.0, 1.0)` when there are no
    /// datasets.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        if self.datasets.is_empty() {
            return (0.0, 1.0, 0.0, 1.0);
        }

        let half_w = self.widths / 2.0;
        let mut xmin = f64::INFINITY;
        let mut xmax = f64::NEG_INFINITY;
        let mut ymin = f64::INFINITY;
        let mut ymax = f64::NEG_INFINITY;

        for (i, dataset) in self.datasets.iter().enumerate() {
            let pos = self.position_for(i);
            xmin = xmin.min(pos - half_w);
            xmax = xmax.max(pos + half_w);
            for &v in dataset {
                if v.is_finite() {
                    ymin = ymin.min(v);
                    ymax = ymax.max(v);
                }
            }
        }

        if !xmin.is_finite() {
            xmin = 0.0;
        }
        if !xmax.is_finite() {
            xmax = 1.0;
        }
        if !ymin.is_finite() {
            ymin = 0.0;
        }
        if !ymax.is_finite() {
            ymax = 1.0;
        }

        (xmin, xmax, ymin, ymax)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a sample ViolinArtist for tests.
    fn sample_violin() -> ViolinArtist {
        ViolinArtist {
            datasets: vec![vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]],
            positions: None,
            widths: 0.7,
            show_median: true,
            show_quartiles: true,
            color: Color::TAB_BLUE,
            alpha: 0.7,
            label: None,
            bw_method: 0.0,
        }
    }

    #[test]
    fn silverman_bandwidth_basic() {
        let sorted = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let bw = silverman_bandwidth(&sorted);
        assert!(bw > 0.0, "bandwidth must be positive");
        assert!(bw < 10.0, "bandwidth should be reasonable");
    }

    #[test]
    fn silverman_bandwidth_single_value() {
        let bw = silverman_bandwidth(&[42.0]);
        assert!((bw - 1.0).abs() < f64::EPSILON, "single value should return fallback bandwidth");
    }

    #[test]
    fn silverman_bandwidth_identical_values() {
        let sorted = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        let bw = silverman_bandwidth(&sorted);
        assert!((bw - 1.0).abs() < f64::EPSILON, "identical values should return fallback bandwidth");
    }

    #[test]
    fn gaussian_kde_basic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let bw = 1.0;
        let eval_points: Vec<f64> = (0..11).map(|i| i as f64 * 0.5).collect();
        let density = gaussian_kde(&data, bw, &eval_points);
        assert_eq!(density.len(), eval_points.len());
        // All density values should be non-negative.
        for &d in &density {
            assert!(d >= 0.0, "density must be non-negative");
        }
    }

    #[test]
    fn gaussian_kde_integrates_roughly_to_one() {
        let data: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
        let bw = silverman_bandwidth(&data);
        // Evaluate on a fine grid covering the data range with margin.
        let n_points = 1000;
        let lo = -2.0;
        let hi = 12.0;
        let step = (hi - lo) / n_points as f64;
        let eval_points: Vec<f64> = (0..=n_points).map(|i| lo + i as f64 * step).collect();
        let density = gaussian_kde(&data, bw, &eval_points);
        let integral: f64 = density.iter().sum::<f64>() * step;
        assert!(
            (integral - 1.0).abs() < 0.1,
            "KDE integral should be approximately 1.0, got {integral}"
        );
    }

    #[test]
    fn gaussian_kde_single_point() {
        let data = vec![5.0];
        let bw = 1.0;
        let density = gaussian_kde(&data, bw, &[5.0]);
        // Peak should be at the data point.
        assert!(density[0] > 0.0);
    }

    #[test]
    fn gaussian_kde_symmetry() {
        let data = vec![0.0];
        let bw = 1.0;
        let d_left = gaussian_kde(&data, bw, &[-1.0]);
        let d_right = gaussian_kde(&data, bw, &[1.0]);
        assert!(
            (d_left[0] - d_right[0]).abs() < 1e-10,
            "KDE should be symmetric around single data point"
        );
    }

    #[test]
    fn data_bounds_single_dataset() {
        let artist = sample_violin();
        let (xmin, xmax, ymin, ymax) = artist.data_bounds();
        // Default position for index 0 is 1.0, half width is 0.35.
        assert!((xmin - 0.65).abs() < f64::EPSILON);
        assert!((xmax - 1.35).abs() < f64::EPSILON);
        assert!((ymin - 1.0).abs() < f64::EPSILON);
        assert!((ymax - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn data_bounds_multiple_datasets() {
        let artist = ViolinArtist {
            datasets: vec![
                vec![1.0, 2.0, 3.0],
                vec![10.0, 20.0, 30.0],
            ],
            positions: None,
            widths: 0.7,
            show_median: true,
            show_quartiles: true,
            color: Color::TAB_BLUE,
            alpha: 0.7,
            label: None,
            bw_method: 0.0,
        };
        let (xmin, xmax, ymin, ymax) = artist.data_bounds();
        // Position 0 -> 1.0, position 1 -> 2.0
        assert!((xmin - 0.65).abs() < f64::EPSILON);
        assert!((xmax - 2.35).abs() < f64::EPSILON);
        assert!((ymin - 1.0).abs() < f64::EPSILON);
        assert!((ymax - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn data_bounds_custom_positions() {
        let artist = ViolinArtist {
            datasets: vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            positions: Some(vec![5.0, 10.0]),
            widths: 1.0,
            show_median: true,
            show_quartiles: true,
            color: Color::TAB_BLUE,
            alpha: 0.7,
            label: None,
            bw_method: 0.0,
        };
        let (xmin, xmax, _ymin, _ymax) = artist.data_bounds();
        assert!((xmin - 4.5).abs() < f64::EPSILON);
        assert!((xmax - 10.5).abs() < f64::EPSILON);
    }

    #[test]
    fn data_bounds_empty() {
        let artist = ViolinArtist {
            datasets: vec![],
            positions: None,
            widths: 0.7,
            show_median: true,
            show_quartiles: true,
            color: Color::TAB_BLUE,
            alpha: 0.7,
            label: None,
            bw_method: 0.0,
        };
        assert_eq!(artist.data_bounds(), (0.0, 1.0, 0.0, 1.0));
    }

    #[test]
    fn data_bounds_nan_filtered() {
        let artist = ViolinArtist {
            datasets: vec![vec![f64::NAN, 1.0, 5.0, f64::NAN]],
            positions: None,
            widths: 0.7,
            show_median: true,
            show_quartiles: true,
            color: Color::TAB_BLUE,
            alpha: 0.7,
            label: None,
            bw_method: 0.0,
        };
        let (_xmin, _xmax, ymin, ymax) = artist.data_bounds();
        assert!((ymin - 1.0).abs() < f64::EPSILON);
        assert!((ymax - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_color() {
        let mut artist = sample_violin();
        artist.color(Color::TAB_RED);
        assert_eq!(artist.color, Color::TAB_RED);
    }

    #[test]
    fn builder_label() {
        let mut artist = sample_violin();
        artist.label("my violin");
        assert_eq!(artist.label.as_deref(), Some("my violin"));
    }

    #[test]
    fn builder_alpha() {
        let mut artist = sample_violin();
        artist.alpha(0.5);
        assert!((artist.alpha - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_alpha_clamped() {
        let mut artist = sample_violin();
        artist.alpha(2.0);
        assert!((artist.alpha - 1.0).abs() < f64::EPSILON);
        artist.alpha(-1.0);
        assert!((artist.alpha - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_widths() {
        let mut artist = sample_violin();
        artist.widths(0.5);
        assert!((artist.widths - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_widths_clamped() {
        let mut artist = sample_violin();
        artist.widths(0.01);
        assert!((artist.widths - 0.1).abs() < f64::EPSILON);
        artist.widths(5.0);
        assert!((artist.widths - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_show_median() {
        let mut artist = sample_violin();
        artist.show_median(false);
        assert!(!artist.show_median);
    }

    #[test]
    fn builder_show_quartiles() {
        let mut artist = sample_violin();
        artist.show_quartiles(false);
        assert!(!artist.show_quartiles);
    }

    #[test]
    fn builder_bw_method() {
        let mut artist = sample_violin();
        artist.bw_method(0.5);
        assert!((artist.bw_method - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_positions() {
        let mut artist = sample_violin();
        artist.positions(vec![2.0, 4.0, 6.0]);
        assert_eq!(artist.positions, Some(vec![2.0, 4.0, 6.0]));
    }

    #[test]
    fn position_for_default() {
        let artist = sample_violin();
        assert!((artist.position_for(0) - 1.0).abs() < f64::EPSILON);
        assert!((artist.position_for(2) - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn position_for_custom() {
        let mut artist = sample_violin();
        artist.positions(vec![10.0, 20.0]);
        assert!((artist.position_for(0) - 10.0).abs() < f64::EPSILON);
        assert!((artist.position_for(1) - 20.0).abs() < f64::EPSILON);
        // Beyond custom positions, falls back to default.
        assert!((artist.position_for(2) - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn percentile_basic() {
        let data = [1.0, 2.0, 3.0, 4.0];
        assert!((percentile(&data, 0.5) - 2.5).abs() < 1e-10);
    }

    #[test]
    fn std_dev_basic() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let sd = std_dev(&data);
        assert!((sd - 2.0).abs() < 1e-10);
    }

    #[test]
    fn std_dev_single() {
        assert!((std_dev(&[5.0]) - 0.0).abs() < f64::EPSILON);
    }
}
