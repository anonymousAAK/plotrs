//! Heatmap chart builder methods.
//!
//! Provides a fluent builder API for configuring [`HeatmapArtist`] instances.
//! Each method returns `&mut Self`, allowing calls to be chained together
//! for concise, readable chart construction.

use crate::artist::HeatmapArtist;
use crate::colormap::Colormap;

impl HeatmapArtist {
    /// Sets the colormap used to map cell values to colors.
    pub fn colormap(&mut self, cmap: Colormap) -> &mut Self {
        self.cmap = cmap;
        self
    }

    /// Sets the minimum value for colormap normalisation.
    pub fn vmin(&mut self, min: f64) -> &mut Self {
        self.vmin = Some(min);
        self
    }

    /// Sets the maximum value for colormap normalisation.
    pub fn vmax(&mut self, max: f64) -> &mut Self {
        self.vmax = Some(max);
        self
    }

    /// Enables or disables drawing cell values as text in each cell.
    pub fn show_values(&mut self, show: bool) -> &mut Self {
        self.show_values = show;
        self
    }

    /// Sets the legend label.
    pub fn label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }

    /// Sets the x-axis labels for the heatmap columns.
    pub fn x_labels(&mut self, labels: Vec<String>) -> &mut Self {
        self.x_labels = Some(labels);
        self
    }

    /// Sets the y-axis labels for the heatmap rows.
    pub fn y_labels(&mut self, labels: Vec<String>) -> &mut Self {
        self.y_labels = Some(labels);
        self
    }

    /// Returns the effective minimum value for colormap normalisation.
    pub fn effective_vmin(&self) -> f64 {
        if let Some(v) = self.vmin {
            return v;
        }
        let mut lo = f64::INFINITY;
        for row in &self.data {
            for &val in row {
                if val.is_finite() && val < lo {
                    lo = val;
                }
            }
        }
        if lo.is_finite() { lo } else { 0.0 }
    }

    /// Returns the effective maximum value for colormap normalisation.
    pub fn effective_vmax(&self) -> f64 {
        if let Some(v) = self.vmax {
            return v;
        }
        let mut hi = f64::NEG_INFINITY;
        for row in &self.data {
            for &val in row {
                if val.is_finite() && val > hi {
                    hi = val;
                }
            }
        }
        if hi.is_finite() { hi } else { 1.0 }
    }
}

#[cfg(test)]
mod tests {
    use crate::artist::HeatmapArtist;
    use crate::colormap::Colormap;
    use crate::primitives::Color;

    fn sample_heatmap() -> HeatmapArtist {
        HeatmapArtist {
            data: vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]],
            x_labels: None, y_labels: None,
            cmap: Colormap::Viridis, vmin: None, vmax: None,
            show_values: false, color: Color::TAB_BLUE, label: None,
        }
    }

    #[test]
    fn builder_colormap() {
        let mut h = sample_heatmap();
        h.colormap(Colormap::Plasma);
        assert_eq!(h.cmap, Colormap::Plasma);
    }

    #[test]
    fn builder_vmin() {
        let mut h = sample_heatmap();
        h.vmin(-10.0);
        assert_eq!(h.vmin, Some(-10.0));
    }

    #[test]
    fn builder_vmax() {
        let mut h = sample_heatmap();
        h.vmax(100.0);
        assert_eq!(h.vmax, Some(100.0));
    }

    #[test]
    fn builder_show_values() {
        let mut h = sample_heatmap();
        assert!(!h.show_values);
        h.show_values(true);
        assert!(h.show_values);
    }

    #[test]
    fn builder_label() {
        let mut h = sample_heatmap();
        h.label("my heatmap");
        assert_eq!(h.label.as_deref(), Some("my heatmap"));
    }

    #[test]
    fn builder_x_labels() {
        let mut h = sample_heatmap();
        h.x_labels(vec!["A".into(), "B".into(), "C".into()]);
        assert_eq!(h.x_labels.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn builder_y_labels() {
        let mut h = sample_heatmap();
        h.y_labels(vec!["row1".into(), "row2".into()]);
        assert_eq!(h.y_labels.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn effective_vmin_auto() {
        let h = sample_heatmap();
        assert!((h.effective_vmin() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn effective_vmax_auto() {
        let h = sample_heatmap();
        assert!((h.effective_vmax() - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn effective_vmin_explicit() {
        let mut h = sample_heatmap();
        h.vmin(-5.0);
        assert!((h.effective_vmin() - (-5.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn effective_vmax_explicit() {
        let mut h = sample_heatmap();
        h.vmax(50.0);
        assert!((h.effective_vmax() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn effective_vmin_empty_data() {
        let h = HeatmapArtist {
            data: vec![], x_labels: None, y_labels: None,
            cmap: Colormap::Viridis, vmin: None, vmax: None,
            show_values: false, color: Color::TAB_BLUE, label: None,
        };
        assert!((h.effective_vmin() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn effective_vmax_empty_data() {
        let h = HeatmapArtist {
            data: vec![], x_labels: None, y_labels: None,
            cmap: Colormap::Viridis, vmin: None, vmax: None,
            show_values: false, color: Color::TAB_BLUE, label: None,
        };
        assert!((h.effective_vmax() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn data_bounds_basic() {
        let h = sample_heatmap();
        let (xmin, xmax, ymin, ymax) = h.data_bounds();
        assert!((xmin - 0.0).abs() < f64::EPSILON);
        assert!((xmax - 3.0).abs() < f64::EPSILON);
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn data_bounds_empty() {
        let h = HeatmapArtist {
            data: vec![], x_labels: None, y_labels: None,
            cmap: Colormap::Viridis, vmin: None, vmax: None,
            show_values: false, color: Color::TAB_BLUE, label: None,
        };
        assert_eq!(h.data_bounds(), (0.0, 1.0, 0.0, 1.0));
    }

    #[test]
    fn data_bounds_single_cell() {
        let h = HeatmapArtist {
            data: vec![vec![42.0]], x_labels: None, y_labels: None,
            cmap: Colormap::Viridis, vmin: None, vmax: None,
            show_values: false, color: Color::TAB_BLUE, label: None,
        };
        let (xmin, xmax, ymin, ymax) = h.data_bounds();
        assert!((xmin - 0.0).abs() < f64::EPSILON);
        assert!((xmax - 1.0).abs() < f64::EPSILON);
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_chaining() {
        let mut h = sample_heatmap();
        h.colormap(Colormap::Plasma).vmin(0.0).vmax(10.0)
            .show_values(true).label("chained");
        assert_eq!(h.cmap, Colormap::Plasma);
        assert_eq!(h.vmin, Some(0.0));
        assert_eq!(h.vmax, Some(10.0));
        assert!(h.show_values);
        assert_eq!(h.label.as_deref(), Some("chained"));
    }

    #[test]
    fn effective_bounds_with_nan() {
        let h = HeatmapArtist {
            data: vec![vec![f64::NAN, 3.0], vec![1.0, f64::NAN]],
            x_labels: None, y_labels: None,
            cmap: Colormap::Viridis, vmin: None, vmax: None,
            show_values: false, color: Color::TAB_BLUE, label: None,
        };
        assert!((h.effective_vmin() - 1.0).abs() < f64::EPSILON);
        assert!((h.effective_vmax() - 3.0).abs() < f64::EPSILON);
    }
}
