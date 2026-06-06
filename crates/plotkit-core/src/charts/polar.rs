//! Polar plot builder methods.
//!
//! Provides a fluent builder API for configuring [`PolarArtist`] instances.
//! Polar plots represent data in a polar coordinate system where each point
//! is defined by an angle (theta, in radians) and a radial distance (r).
//!
//! Two modes are supported:
//! - **Line mode** (`filled = false`): draws a polyline connecting the data
//!   points in polar space.
//! - **Filled mode** (`filled = true`): closes the polar path and fills it,
//!   producing a radar/area chart.
//!
//! # Examples
//!
//! ```ignore
//! ax.polar_plot(&theta, &r)?
//!     .color(Color::TAB_BLUE)
//!     .linewidth(2.0)
//!     .label("Wind speed");
//!
//! ax.polar_fill(&theta, &r)?
//!     .color(Color::TAB_ORANGE)
//!     .alpha(0.3)
//!     .label("Coverage");
//! ```

use crate::artist::PolarArtist;
use crate::primitives::Color;
use crate::theme::Marker;

impl PolarArtist {
    /// Sets the line/fill color.
    ///
    /// Accepts any [`Color`] value.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    /// Sets the legend label for this polar series.
    ///
    /// When a label is set, this series will appear in the legend if one is
    /// displayed on the axes.
    pub fn label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }

    /// Sets the opacity (0.0 = fully transparent, 1.0 = fully opaque).
    ///
    /// The value is clamped to the `[0.0, 1.0]` range.
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Sets the stroke width in pixels for the polar line.
    ///
    /// A width of `1.5` is the default. Values below `1.0` may produce
    /// sub-pixel rendering depending on the backend.
    pub fn linewidth(&mut self, width: f64) -> &mut Self {
        self.linewidth = width;
        self
    }

    /// Controls whether the polar path is closed and filled.
    ///
    /// When `true`, the path is closed (connecting the last point back to
    /// the first) and the interior is filled with the configured color and
    /// alpha. When `false` (default for `polar_plot`), only the line is drawn.
    pub fn filled(&mut self, filled: bool) -> &mut Self {
        self.filled = filled;
        self
    }

    /// Sets the marker shape drawn at each data point.
    ///
    /// By default, no markers are drawn (`None`). Set to `Some(Marker::Circle)`
    /// or another variant to show markers at each vertex.
    pub fn marker(&mut self, marker: Marker) -> &mut Self {
        self.marker = Some(marker);
        self
    }

    /// Computes the data-space bounding box `(xmin, xmax, ymin, ymax)` in
    /// Cartesian coordinates derived from the polar data.
    ///
    /// The polar data is converted to Cartesian coordinates (x = r*cos(theta),
    /// y = r*sin(theta)) and the bounding box of those points is returned.
    /// Falls back to `(-1.0, 1.0, -1.0, 1.0)` when no finite data exists.
    pub fn data_bounds(&self) -> (f64, f64, f64, f64) {
        if self.r.is_empty() || self.theta.is_empty() {
            return (-1.0, 1.0, -1.0, 1.0);
        }

        let r_max = self.max_finite_r();
        if r_max <= 0.0 || !r_max.is_finite() {
            return (-1.0, 1.0, -1.0, 1.0);
        }

        // For a polar plot, the data space is a circle of radius r_max
        // centered at the origin. We add a small margin.
        let extent = r_max * 1.1;
        (-extent, extent, -extent, extent)
    }

    /// Returns the maximum finite, positive radial value.
    ///
    /// Returns 0.0 if no finite positive values exist.
    pub fn max_finite_r(&self) -> f64 {
        self.r
            .iter()
            .copied()
            .filter(|v| v.is_finite() && *v >= 0.0)
            .fold(0.0_f64, f64::max)
    }

    /// Converts polar coordinates (r, theta) to Cartesian coordinates (x, y).
    ///
    /// The angle `theta` is in radians, measured counter-clockwise from the
    /// positive x-axis (3 o'clock position).
    pub fn polar_to_cartesian(r: f64, theta: f64) -> (f64, f64) {
        (r * theta.cos(), r * theta.sin())
    }

    /// Returns an iterator of (x, y) Cartesian points derived from the polar data.
    ///
    /// Non-finite r or theta values are filtered out.
    pub fn cartesian_points(&self) -> Vec<(f64, f64)> {
        let n = self.r.len().min(self.theta.len());
        (0..n)
            .filter(|&i| self.r[i].is_finite() && self.theta[i].is_finite() && self.r[i] >= 0.0)
            .map(|i| Self::polar_to_cartesian(self.r[i], self.theta[i]))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI, TAU};

    /// Helper: build a sample PolarArtist for testing.
    fn sample_polar() -> PolarArtist {
        PolarArtist {
            theta: vec![0.0, FRAC_PI_2, PI, 3.0 * FRAC_PI_2],
            r: vec![1.0, 2.0, 1.5, 0.5],
            color: Color::TAB_BLUE,
            label: None,
            alpha: 1.0,
            linewidth: 1.5,
            filled: false,
            marker: None,
        }
    }

    // -- Builder methods ---------------------------------------------------

    #[test]
    fn builder_color() {
        let mut a = sample_polar();
        a.color(Color::TAB_RED);
        assert_eq!(a.color, Color::TAB_RED);
    }

    #[test]
    fn builder_label() {
        let mut a = sample_polar();
        a.label("wind");
        assert_eq!(a.label, Some("wind".to_string()));
    }

    #[test]
    fn builder_alpha() {
        let mut a = sample_polar();
        a.alpha(0.5);
        assert!((a.alpha - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_alpha_clamped() {
        let mut a = sample_polar();
        a.alpha(1.5);
        assert!((a.alpha - 1.0).abs() < f64::EPSILON);
        a.alpha(-0.5);
        assert!(a.alpha.abs() < f64::EPSILON);
    }

    #[test]
    fn builder_linewidth() {
        let mut a = sample_polar();
        a.linewidth(3.0);
        assert!((a.linewidth - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_filled() {
        let mut a = sample_polar();
        assert!(!a.filled);
        a.filled(true);
        assert!(a.filled);
    }

    #[test]
    fn builder_marker() {
        let mut a = sample_polar();
        assert!(a.marker.is_none());
        a.marker(Marker::Circle);
        assert_eq!(a.marker, Some(Marker::Circle));
    }

    // -- Data bounds -------------------------------------------------------

    #[test]
    fn data_bounds_basic() {
        let a = sample_polar();
        let (xmin, xmax, ymin, ymax) = a.data_bounds();
        // r_max = 2.0, extent = 2.0 * 1.1 = 2.2
        assert!((xmin - (-2.2)).abs() < 1e-10);
        assert!((xmax - 2.2).abs() < 1e-10);
        assert!((ymin - (-2.2)).abs() < 1e-10);
        assert!((ymax - 2.2).abs() < 1e-10);
    }

    #[test]
    fn data_bounds_empty() {
        let a = PolarArtist {
            theta: vec![],
            r: vec![],
            color: Color::TAB_BLUE,
            label: None,
            alpha: 1.0,
            linewidth: 1.5,
            filled: false,
            marker: None,
        };
        assert_eq!(a.data_bounds(), (-1.0, 1.0, -1.0, 1.0));
    }

    #[test]
    fn data_bounds_all_nan() {
        let a = PolarArtist {
            theta: vec![0.0, 1.0],
            r: vec![f64::NAN, f64::NAN],
            color: Color::TAB_BLUE,
            label: None,
            alpha: 1.0,
            linewidth: 1.5,
            filled: false,
            marker: None,
        };
        assert_eq!(a.data_bounds(), (-1.0, 1.0, -1.0, 1.0));
    }

    // -- Polar to Cartesian ------------------------------------------------

    #[test]
    fn polar_to_cartesian_at_zero() {
        let (x, y) = PolarArtist::polar_to_cartesian(1.0, 0.0);
        assert!((x - 1.0).abs() < 1e-10);
        assert!(y.abs() < 1e-10);
    }

    #[test]
    fn polar_to_cartesian_at_90_deg() {
        let (x, y) = PolarArtist::polar_to_cartesian(2.0, FRAC_PI_2);
        assert!(x.abs() < 1e-10);
        assert!((y - 2.0).abs() < 1e-10);
    }

    #[test]
    fn polar_to_cartesian_at_pi() {
        let (x, y) = PolarArtist::polar_to_cartesian(1.0, PI);
        assert!((x - (-1.0)).abs() < 1e-10);
        assert!(y.abs() < 1e-10);
    }

    #[test]
    fn polar_to_cartesian_at_270_deg() {
        let (x, y) = PolarArtist::polar_to_cartesian(3.0, 3.0 * FRAC_PI_2);
        assert!(x.abs() < 1e-10);
        assert!((y - (-3.0)).abs() < 1e-10);
    }

    // -- Cartesian points --------------------------------------------------

    #[test]
    fn cartesian_points_basic() {
        let a = PolarArtist {
            theta: vec![0.0, FRAC_PI_2],
            r: vec![1.0, 2.0],
            color: Color::TAB_BLUE,
            label: None,
            alpha: 1.0,
            linewidth: 1.5,
            filled: false,
            marker: None,
        };
        let pts = a.cartesian_points();
        assert_eq!(pts.len(), 2);
        assert!((pts[0].0 - 1.0).abs() < 1e-10);
        assert!(pts[0].1.abs() < 1e-10);
        assert!(pts[1].0.abs() < 1e-10);
        assert!((pts[1].1 - 2.0).abs() < 1e-10);
    }

    #[test]
    fn cartesian_points_nan_filtered() {
        let a = PolarArtist {
            theta: vec![0.0, f64::NAN, PI],
            r: vec![1.0, 2.0, f64::NAN],
            color: Color::TAB_BLUE,
            label: None,
            alpha: 1.0,
            linewidth: 1.5,
            filled: false,
            marker: None,
        };
        let pts = a.cartesian_points();
        // Only the first point survives: (1.0, 0.0)
        assert_eq!(pts.len(), 1);
        assert!((pts[0].0 - 1.0).abs() < 1e-10);
    }

    // -- Angle ranges ------------------------------------------------------

    #[test]
    fn negative_angles() {
        let (x, y) = PolarArtist::polar_to_cartesian(1.0, -FRAC_PI_2);
        assert!(x.abs() < 1e-10);
        assert!((y - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn angles_greater_than_two_pi() {
        // 2*PI + PI/2 should equal PI/2 modulo 2*PI
        let (x1, y1) = PolarArtist::polar_to_cartesian(1.0, TAU + FRAC_PI_2);
        let (x2, y2) = PolarArtist::polar_to_cartesian(1.0, FRAC_PI_2);
        assert!((x1 - x2).abs() < 1e-10);
        assert!((y1 - y2).abs() < 1e-10);
    }

    #[test]
    fn max_finite_r_ignores_nan_and_negative() {
        let a = PolarArtist {
            theta: vec![0.0, 1.0, 2.0, 3.0],
            r: vec![f64::NAN, -1.0, 5.0, 3.0],
            color: Color::TAB_BLUE,
            label: None,
            alpha: 1.0,
            linewidth: 1.5,
            filled: false,
            marker: None,
        };
        assert!((a.max_finite_r() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn data_bounds_with_single_point() {
        let a = PolarArtist {
            theta: vec![0.0],
            r: vec![3.0],
            color: Color::TAB_BLUE,
            label: None,
            alpha: 1.0,
            linewidth: 1.5,
            filled: false,
            marker: None,
        };
        let (xmin, xmax, ymin, ymax) = a.data_bounds();
        // r_max = 3.0, extent = 3.3
        assert!((xmin - (-3.3)).abs() < 1e-10);
        assert!((xmax - 3.3).abs() < 1e-10);
        assert!((ymin - (-3.3)).abs() < 1e-10);
        assert!((ymax - 3.3).abs() < 1e-10);
    }

    #[test]
    fn builder_chaining_returns_self() {
        let mut a = sample_polar();
        let _ = a
            .color(Color::TAB_GREEN)
            .label("test")
            .alpha(0.7)
            .linewidth(2.5)
            .filled(true)
            .marker(Marker::Diamond);
        assert_eq!(a.color, Color::TAB_GREEN);
        assert_eq!(a.label, Some("test".to_string()));
        assert!((a.alpha - 0.7).abs() < f64::EPSILON);
        assert!((a.linewidth - 2.5).abs() < f64::EPSILON);
        assert!(a.filled);
        assert_eq!(a.marker, Some(Marker::Diamond));
    }
}
