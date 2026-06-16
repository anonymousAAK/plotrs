//! Step chart builder methods.
//!
//! Provides a fluent builder API for configuring [`StepArtist`] instances.

use crate::artist::StepArtist;
use crate::primitives::Color;

impl StepArtist {
    /// Sets the step line color.
    pub fn color(&mut self, c: Color) -> &mut Self {
        self.color = c;
        self
    }
    /// Sets the step line width.
    pub fn width(&mut self, w: f64) -> &mut Self {
        self.width = w;
        self
    }
    /// Sets the step alignment mode.
    pub fn where_step(&mut self, w: crate::artist::StepWhere) -> &mut Self {
        self.where_step = w;
        self
    }
    /// Sets the legend label.
    pub fn label(&mut self, l: &str) -> &mut Self {
        self.label = Some(l.to_string());
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
    use crate::artist::StepWhere;
    use crate::series::Series;
    const T: f64 = 1e-12;
    fn eq(a: f64, b: f64) -> bool {
        (a - b).abs() < T
    }
    fn s() -> StepArtist {
        StepArtist {
            x: Series::new(vec![1.0, 2.0, 3.0, 4.0]),
            y: Series::new(vec![1.0, 3.0, 2.0, 4.0]),
            color: Color::TAB_BLUE,
            width: 1.5,
            where_step: StepWhere::Pre,
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
    fn w() {
        let mut a = s();
        a.width(3.0);
        assert!(eq(a.width, 3.0));
    }
    #[test]
    fn wp() {
        let mut a = s();
        a.where_step(StepWhere::Pre);
        assert!(matches!(a.where_step, StepWhere::Pre));
    }
    #[test]
    fn wpo() {
        let mut a = s();
        a.where_step(StepWhere::Post);
        assert!(matches!(a.where_step, StepWhere::Post));
    }
    #[test]
    fn wm() {
        let mut a = s();
        a.where_step(StepWhere::Mid);
        assert!(matches!(a.where_step, StepWhere::Mid));
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
            .width(2.0)
            .where_step(StepWhere::Post)
            .label("T")
            .alpha(0.8);
        assert_eq!(a.color, Color::TAB_GREEN);
        assert!(eq(a.width, 2.0));
        assert!(matches!(a.where_step, StepWhere::Post));
        assert_eq!(a.label.as_deref(), Some("T"));
        assert!(eq(a.alpha, 0.8));
    }
    #[test]
    fn dp() {
        let a = s();
        assert!(matches!(a.where_step, StepWhere::Pre));
    }
}
