//! Error bar chart builder methods.
//!
//! This module extends [`ErrorBarArtist`] with a fluent API for configuring
//! error bar properties. Since [`Axes::errorbar`] returns
//! `Result<ErrorBarArtist>`, these builder methods can be chained
//! directly on the return value:
//!
//! ```ignore
//! let eb = ax.errorbar(&x, &y)?
//!     .yerr_symmetric(&errs)
//!     .cap_size(5.0)
//!     .line_width(1.5)
//!     .color(Color::TAB_RED)
//!     .label("Measurements");
//! ax.add_errorbar(eb);
//! ```
//!
//! [`Axes::errorbar`]: crate::axes::Axes::errorbar

use crate::artist::ErrorBarArtist;
use crate::primitives::Color;

impl ErrorBarArtist {
    /// Sets the color for the center line, error bars, and caps.
    ///
    /// Accepts any [`Color`] value, which can be constructed from RGB
    /// components, hex strings, or named color constants.
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

    /// Sets the legend label for this error bar plot.
    ///
    /// When a label is set, the error bar plot will appear in the legend if
    /// one is displayed on the axes. Pass an empty string or omit this call
    /// to exclude the plot from the legend.
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

    /// Sets the cap size in pixels for the error bar ends.
    ///
    /// Caps are the small horizontal (or vertical) lines drawn at the tips
    /// of the error bars. A value of `0.0` disables caps entirely. The
    /// default is `4.0` pixels.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.cap_size(6.0);
    /// ```
    pub fn cap_size(&mut self, size: f64) -> &mut Self {
        self.cap_size = size.max(0.0);
        self
    }

    /// Sets the stroke width of the error bar lines and caps.
    ///
    /// Also affects the center connecting line. A width of `1.0` is the
    /// default hairline width.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.line_width(2.0);
    /// ```
    pub fn line_width(&mut self, width: f64) -> &mut Self {
        self.line_width = width.max(0.0);
        self
    }

    /// Sets symmetric y-error values.
    ///
    /// Each error value `e` produces an error bar from `y - e` to `y + e`
    /// at the corresponding data point. The length of `errs` must equal the
    /// number of data points.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.yerr_symmetric(&vec![0.5, 0.3, 0.4]);
    /// ```
    pub fn yerr_symmetric(mut self, errs: &[f64]) -> Self {
        self.yerr = Some(crate::artist::ErrorBarData::Symmetric(errs.to_vec()));
        self
    }

    /// Sets asymmetric y-error values.
    ///
    /// Each point gets a separate low and high error. The error bar spans
    /// from `y - low[i]` to `y + high[i]`. Both slices must have the same
    /// length as the data series.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.yerr_asymmetric(&low_errs, &high_errs);
    /// ```
    pub fn yerr_asymmetric(mut self, low: &[f64], high: &[f64]) -> Self {
        self.yerr = Some(crate::artist::ErrorBarData::Asymmetric {
            low: low.to_vec(),
            high: high.to_vec(),
        });
        self
    }

    /// Sets symmetric x-error values.
    ///
    /// Each error value `e` produces a horizontal error bar from `x - e`
    /// to `x + e` at the corresponding data point. The length of `errs`
    /// must equal the number of data points.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.xerr_symmetric(&vec![0.2, 0.1, 0.15]);
    /// ```
    pub fn xerr_symmetric(mut self, errs: &[f64]) -> Self {
        self.xerr = Some(crate::artist::ErrorBarData::Symmetric(errs.to_vec()));
        self
    }

    /// Sets asymmetric x-error values.
    ///
    /// Each point gets a separate low and high horizontal error. The error
    /// bar spans from `x - low[i]` to `x + high[i]`. Both slices must
    /// have the same length as the data series.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// artist.xerr_asymmetric(&low_errs, &high_errs);
    /// ```
    pub fn xerr_asymmetric(mut self, low: &[f64], high: &[f64]) -> Self {
        self.xerr = Some(crate::artist::ErrorBarData::Asymmetric {
            low: low.to_vec(),
            high: high.to_vec(),
        });
        self
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artist::ErrorBarData;
    use crate::series::Series;

    /// Tolerance for floating-point comparisons.
    const TOL: f64 = 1e-12;

    /// Returns true if `a` and `b` are within `TOL` of each other.
    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < TOL
    }

    /// Helper: build a minimal `ErrorBarArtist` for testing.
    fn sample_errorbar() -> ErrorBarArtist {
        ErrorBarArtist {
            x: Series::new(vec![1.0, 2.0, 3.0]),
            y: Series::new(vec![10.0, 20.0, 30.0]),
            xerr: None,
            yerr: None,
            color: Color::TAB_BLUE,
            label: None,
            cap_size: 4.0,
            line_width: 1.0,
        }
    }

    #[test]
    fn builder_color() {
        let mut a = sample_errorbar();
        a.color(Color::TAB_RED);
        assert_eq!(a.color, Color::TAB_RED);
    }

    #[test]
    fn builder_label() {
        let mut a = sample_errorbar();
        assert!(a.label.is_none());
        a.label("Measurements");
        assert_eq!(a.label.as_deref(), Some("Measurements"));
    }

    #[test]
    fn builder_label_overwrite() {
        let mut a = sample_errorbar();
        a.label("first");
        a.label("second");
        assert_eq!(a.label.as_deref(), Some("second"));
    }

    #[test]
    fn builder_cap_size() {
        let mut a = sample_errorbar();
        a.cap_size(8.0);
        assert!(approx_eq(a.cap_size, 8.0));
    }

    #[test]
    fn builder_cap_size_clamps_negative() {
        let mut a = sample_errorbar();
        a.cap_size(-5.0);
        assert!(approx_eq(a.cap_size, 0.0));
    }

    #[test]
    fn builder_line_width() {
        let mut a = sample_errorbar();
        a.line_width(2.5);
        assert!(approx_eq(a.line_width, 2.5));
    }

    #[test]
    fn builder_line_width_clamps_negative() {
        let mut a = sample_errorbar();
        a.line_width(-1.0);
        assert!(approx_eq(a.line_width, 0.0));
    }

    #[test]
    fn builder_yerr_symmetric() {
        let a = sample_errorbar().yerr_symmetric(&[0.5, 1.0, 1.5]);
        match &a.yerr {
            Some(ErrorBarData::Symmetric(v)) => {
                assert_eq!(v.len(), 3);
                assert!(approx_eq(v[0], 0.5));
                assert!(approx_eq(v[1], 1.0));
                assert!(approx_eq(v[2], 1.5));
            }
            _ => panic!("expected Symmetric yerr"),
        }
    }

    #[test]
    fn builder_yerr_asymmetric() {
        let a = sample_errorbar().yerr_asymmetric(&[0.3, 0.5, 0.7], &[1.0, 1.2, 1.4]);
        match &a.yerr {
            Some(ErrorBarData::Asymmetric { low, high }) => {
                assert_eq!(low.len(), 3);
                assert_eq!(high.len(), 3);
                assert!(approx_eq(low[0], 0.3));
                assert!(approx_eq(high[2], 1.4));
            }
            _ => panic!("expected Asymmetric yerr"),
        }
    }

    #[test]
    fn builder_xerr_symmetric() {
        let a = sample_errorbar().xerr_symmetric(&[0.1, 0.2, 0.3]);
        match &a.xerr {
            Some(ErrorBarData::Symmetric(v)) => {
                assert_eq!(v.len(), 3);
                assert!(approx_eq(v[0], 0.1));
            }
            _ => panic!("expected Symmetric xerr"),
        }
    }

    #[test]
    fn builder_xerr_asymmetric() {
        let a = sample_errorbar().xerr_asymmetric(&[0.1, 0.2, 0.3], &[0.4, 0.5, 0.6]);
        match &a.xerr {
            Some(ErrorBarData::Asymmetric { low, high }) => {
                assert_eq!(low.len(), 3);
                assert_eq!(high.len(), 3);
                assert!(approx_eq(low[1], 0.2));
                assert!(approx_eq(high[1], 0.5));
            }
            _ => panic!("expected Asymmetric xerr"),
        }
    }

    #[test]
    fn builder_chaining() {
        let mut a = sample_errorbar();
        a.color(Color::TAB_GREEN)
            .label("Test")
            .cap_size(6.0)
            .line_width(2.0);

        assert_eq!(a.color, Color::TAB_GREEN);
        assert_eq!(a.label.as_deref(), Some("Test"));
        assert!(approx_eq(a.cap_size, 6.0));
        assert!(approx_eq(a.line_width, 2.0));
    }

    #[test]
    fn data_bounds_no_errors() {
        let a = sample_errorbar();
        let (xmin, xmax, ymin, ymax) = a.data_bounds();
        assert!(approx_eq(xmin, 1.0));
        assert!(approx_eq(xmax, 3.0));
        assert!(approx_eq(ymin, 10.0));
        assert!(approx_eq(ymax, 30.0));
    }

    #[test]
    fn data_bounds_with_symmetric_yerr() {
        let a = sample_errorbar().yerr_symmetric(&[2.0, 3.0, 5.0]);
        let (xmin, xmax, ymin, ymax) = a.data_bounds();
        assert!(approx_eq(xmin, 1.0));
        assert!(approx_eq(xmax, 3.0));
        assert!(approx_eq(ymin, 8.0));
        assert!(approx_eq(ymax, 35.0));
    }

    #[test]
    fn data_bounds_with_asymmetric_yerr() {
        let a = sample_errorbar().yerr_asymmetric(&[1.0, 2.0, 3.0], &[5.0, 6.0, 7.0]);
        let (_, _, ymin, ymax) = a.data_bounds();
        assert!(approx_eq(ymin, 9.0));
        assert!(approx_eq(ymax, 37.0));
    }

    #[test]
    fn data_bounds_with_symmetric_xerr() {
        let a = sample_errorbar().xerr_symmetric(&[0.5, 0.5, 0.5]);
        let (xmin, xmax, _, _) = a.data_bounds();
        assert!(approx_eq(xmin, 0.5));
        assert!(approx_eq(xmax, 3.5));
    }

    #[test]
    fn data_bounds_with_both_errors() {
        let a = sample_errorbar()
            .xerr_symmetric(&[0.5, 0.5, 0.5])
            .yerr_symmetric(&[2.0, 3.0, 5.0]);
        let (xmin, xmax, ymin, ymax) = a.data_bounds();
        assert!(approx_eq(xmin, 0.5));
        assert!(approx_eq(xmax, 3.5));
        assert!(approx_eq(ymin, 8.0));
        assert!(approx_eq(ymax, 35.0));
    }

    #[test]
    fn data_bounds_empty_series() {
        let a = ErrorBarArtist {
            x: Series::new(vec![]),
            y: Series::new(vec![]),
            xerr: None,
            yerr: None,
            color: Color::BLACK,
            label: None,
            cap_size: 4.0,
            line_width: 1.0,
        };
        let (xmin, xmax, ymin, ymax) = a.data_bounds();
        assert!(approx_eq(xmin, 0.0));
        assert!(approx_eq(xmax, 1.0));
        assert!(approx_eq(ymin, 0.0));
        assert!(approx_eq(ymax, 1.0));
    }
}
