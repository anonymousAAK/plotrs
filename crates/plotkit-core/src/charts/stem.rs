//! Stem chart builder methods.
//!
//! Provides a fluent builder API for configuring [`StemArtist`] instances.

use crate::artist::StemArtist;
use crate::primitives::Color;

impl StemArtist {
    /// Sets the stem line and marker color.
    pub fn color(&mut self, c: Color) -> &mut Self {
        self.color = c;
        self
    }
    /// Sets the legend label.
    pub fn label(&mut self, l: &str) -> &mut Self {
        self.label = Some(l.to_string());
        self
    }
    /// Sets the baseline y-value from which stems originate.
    pub fn baseline(&mut self, b: f64) -> &mut Self {
        self.baseline = b;
        self
    }
    /// Sets the marker circle diameter in pixels.
    pub fn marker_size(&mut self, s: f64) -> &mut Self {
        self.marker_size = s;
        self
    }
    /// Sets the stem line width in pixels.
    pub fn width(&mut self, w: f64) -> &mut Self {
        self.line_width = w;
        self
    }
    /// Sets the opacity.
    pub fn alpha(&mut self, a: f64) -> &mut Self {
        self.alpha = a.clamp(0.0, 1.0);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::Series;
    const T: f64 = 1e-12;
    fn eq(a: f64, b: f64) -> bool {
        (a - b).abs() < T
    }
    fn s() -> StemArtist {
        StemArtist {
            x: Series::new(vec![1.0, 2.0, 3.0, 4.0]),
            y: Series::new(vec![1.0, 3.0, 2.0, 4.0]),
            color: Color::TAB_BLUE,
            line_width: 1.5,
            marker_size: 6.0,
            baseline: 0.0,
            label: None,
            alpha: 1.0,
        }
    }
    #[test]
    fn c() {
        let mut a = s();
        a.color(Color::TAB_RED);
        assert_eq!(a.color, Color::TAB_RED);
    }
    #[test]
    fn l() {
        let mut a = s();
        assert!(a.label.is_none());
        a.label("x");
        assert_eq!(a.label.as_deref(), Some("x"));
    }
    #[test]
    fn lo() {
        let mut a = s();
        a.label("a");
        a.label("b");
        assert_eq!(a.label.as_deref(), Some("b"));
    }
    #[test]
    fn bl() {
        let mut a = s();
        a.baseline(2.5);
        assert!(eq(a.baseline, 2.5));
    }
    #[test]
    fn bn() {
        let mut a = s();
        a.baseline(-1.0);
        assert!(eq(a.baseline, -1.0));
    }
    #[test]
    fn ms() {
        let mut a = s();
        a.marker_size(10.0);
        assert!(eq(a.marker_size, 10.0));
    }
    #[test]
    fn w() {
        let mut a = s();
        a.width(3.0);
        assert!(eq(a.line_width, 3.0));
    }
    #[test]
    fn ac() {
        let mut a = s();
        a.alpha(0.5);
        assert!(eq(a.alpha, 0.5));
        a.alpha(-1.0);
        assert!(eq(a.alpha, 0.0));
        a.alpha(2.0);
        assert!(eq(a.alpha, 1.0));
    }
    #[test]
    fn ab() {
        let mut a = s();
        a.alpha(0.0);
        assert!(eq(a.alpha, 0.0));
        a.alpha(1.0);
        assert!(eq(a.alpha, 1.0));
    }
    #[test]
    fn ch() {
        let mut a = s();
        a.color(Color::TAB_GREEN)
            .baseline(1.0)
            .marker_size(8.0)
            .width(2.0)
            .label("T")
            .alpha(0.8);
        assert_eq!(a.color, Color::TAB_GREEN);
        assert!(eq(a.baseline, 1.0));
        assert!(eq(a.marker_size, 8.0));
        assert!(eq(a.line_width, 2.0));
        assert_eq!(a.label.as_deref(), Some("T"));
        assert!(eq(a.alpha, 0.8));
    }
    #[test]
    fn db() {
        let a = s();
        assert!(eq(a.baseline, 0.0));
    }
    #[test]
    fn dm() {
        let a = s();
        assert!(eq(a.marker_size, 6.0));
    }
}
