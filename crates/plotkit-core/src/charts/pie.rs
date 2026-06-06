//! Pie chart builder methods.
//!
//! Provides a fluent builder API for configuring [`PieArtist`] instances.
//! Each method returns `&mut Self`, allowing calls to be chained together
//! for concise, readable chart construction.

use crate::artist::PieArtist;
use crate::primitives::Color;

impl PieArtist {
    /// Sets the wedge labels displayed next to each slice.
    ///
    /// The number of labels should match the number of wedge sizes. If
    /// fewer labels are provided, extra wedges will have no label.
    ///
    /// # Arguments
    ///
    /// * `labels` - A vector of label strings, one per wedge.
    ///
    /// # Examples
    ///
    /// ```
    /// # use plotkit_core::artist::PieArtist;
    /// let mut pie = PieArtist {
    ///     sizes: vec![1.0, 2.0, 3.0],
    ///     labels: None,
    ///     colors: None,
    ///     explode: None,
    ///     autopct: false,
    ///     start_angle: 90.0,
    ///     radius: 1.0,
    ///     label: None,
    ///     color: plotkit_core::primitives::Color::TAB_BLUE,
    /// };
    /// pie.labels(vec!["A", "B", "C"]);
    /// assert_eq!(pie.labels.as_ref().unwrap().len(), 3);
    /// ```
    pub fn labels(&mut self, labels: Vec<&str>) -> &mut Self {
        self.labels = Some(labels.into_iter().map(String::from).collect());
        self
    }

    /// Sets custom colors for each wedge.
    ///
    /// When not set, the theme color cycle is used automatically.
    ///
    /// # Arguments
    ///
    /// * `colors` - A vector of [`Color`] values, one per wedge.
    ///
    /// # Examples
    ///
    /// ```
    /// # use plotkit_core::artist::PieArtist;
    /// # use plotkit_core::primitives::Color;
    /// let mut pie = PieArtist {
    ///     sizes: vec![1.0, 2.0],
    ///     labels: None,
    ///     colors: None,
    ///     explode: None,
    ///     autopct: false,
    ///     start_angle: 90.0,
    ///     radius: 1.0,
    ///     label: None,
    ///     color: Color::TAB_BLUE,
    /// };
    /// pie.colors(vec![Color::TAB_RED, Color::TAB_GREEN]);
    /// assert_eq!(pie.colors.as_ref().unwrap().len(), 2);
    /// ```
    pub fn colors(&mut self, colors: Vec<Color>) -> &mut Self {
        self.colors = Some(colors);
        self
    }

    /// Sets the explode offsets for each wedge.
    ///
    /// Each value represents the fraction of the radius by which the
    /// corresponding wedge is offset from the center. A value of `0.0`
    /// means no offset; `0.1` pushes the wedge outward by 10% of the
    /// radius.
    ///
    /// # Arguments
    ///
    /// * `explode` - A vector of offset fractions, one per wedge.
    ///
    /// # Examples
    ///
    /// ```
    /// # use plotkit_core::artist::PieArtist;
    /// # use plotkit_core::primitives::Color;
    /// let mut pie = PieArtist {
    ///     sizes: vec![1.0, 2.0, 3.0],
    ///     labels: None,
    ///     colors: None,
    ///     explode: None,
    ///     autopct: false,
    ///     start_angle: 90.0,
    ///     radius: 1.0,
    ///     label: None,
    ///     color: Color::TAB_BLUE,
    /// };
    /// pie.explode(vec![0.1, 0.0, 0.0]);
    /// assert_eq!(pie.explode.as_ref().unwrap()[0], 0.1);
    /// ```
    pub fn explode(&mut self, explode: Vec<f64>) -> &mut Self {
        self.explode = Some(explode);
        self
    }

    /// Enables or disables automatic percentage labels on each wedge.
    ///
    /// When enabled, each wedge displays its percentage of the total as
    /// text positioned at the midpoint of the wedge arc.
    ///
    /// # Arguments
    ///
    /// * `show` - `true` to show percentage labels, `false` to hide them.
    ///
    /// # Examples
    ///
    /// ```
    /// # use plotkit_core::artist::PieArtist;
    /// # use plotkit_core::primitives::Color;
    /// let mut pie = PieArtist {
    ///     sizes: vec![1.0, 2.0],
    ///     labels: None,
    ///     colors: None,
    ///     explode: None,
    ///     autopct: false,
    ///     start_angle: 90.0,
    ///     radius: 1.0,
    ///     label: None,
    ///     color: Color::TAB_BLUE,
    /// };
    /// pie.autopct(true);
    /// assert!(pie.autopct);
    /// ```
    pub fn autopct(&mut self, show: bool) -> &mut Self {
        self.autopct = show;
        self
    }

    /// Sets the starting angle for the first wedge in degrees.
    ///
    /// The default is `90.0` (top of the circle). Angles increase
    /// counter-clockwise.
    ///
    /// # Arguments
    ///
    /// * `angle` - The starting angle in degrees.
    ///
    /// # Examples
    ///
    /// ```
    /// # use plotkit_core::artist::PieArtist;
    /// # use plotkit_core::primitives::Color;
    /// let mut pie = PieArtist {
    ///     sizes: vec![1.0, 2.0],
    ///     labels: None,
    ///     colors: None,
    ///     explode: None,
    ///     autopct: false,
    ///     start_angle: 90.0,
    ///     radius: 1.0,
    ///     label: None,
    ///     color: Color::TAB_BLUE,
    /// };
    /// pie.start_angle(0.0);
    /// assert!((pie.start_angle - 0.0).abs() < f64::EPSILON);
    /// ```
    pub fn start_angle(&mut self, angle: f64) -> &mut Self {
        self.start_angle = angle;
        self
    }

    /// Sets the radius of the pie chart in data-space units.
    ///
    /// The default radius is `1.0`. The pie is drawn in a square
    /// coordinate space that accommodates the radius plus any explode
    /// offsets.
    ///
    /// # Arguments
    ///
    /// * `radius` - The radius of the pie.
    ///
    /// # Examples
    ///
    /// ```
    /// # use plotkit_core::artist::PieArtist;
    /// # use plotkit_core::primitives::Color;
    /// let mut pie = PieArtist {
    ///     sizes: vec![1.0, 2.0],
    ///     labels: None,
    ///     colors: None,
    ///     explode: None,
    ///     autopct: false,
    ///     start_angle: 90.0,
    ///     radius: 1.0,
    ///     label: None,
    ///     color: Color::TAB_BLUE,
    /// };
    /// pie.radius(0.8);
    /// assert!((pie.radius - 0.8).abs() < f64::EPSILON);
    /// ```
    pub fn radius(&mut self, radius: f64) -> &mut Self {
        self.radius = radius;
        self
    }

    /// Sets the legend label for the pie chart.
    ///
    /// When a legend is displayed on the figure, this label will appear
    /// next to the color swatch for this pie chart.
    ///
    /// # Arguments
    ///
    /// * `label` - A string slice that will be stored as the legend entry.
    ///
    /// # Examples
    ///
    /// ```
    /// # use plotkit_core::artist::PieArtist;
    /// # use plotkit_core::primitives::Color;
    /// let mut pie = PieArtist {
    ///     sizes: vec![1.0, 2.0],
    ///     labels: None,
    ///     colors: None,
    ///     explode: None,
    ///     autopct: false,
    ///     start_angle: 90.0,
    ///     radius: 1.0,
    ///     label: None,
    ///     color: Color::TAB_BLUE,
    /// };
    /// pie.label("pie chart");
    /// assert_eq!(pie.label.as_deref(), Some("pie chart"));
    /// ```
    pub fn label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::artist::PieArtist;
    use crate::primitives::Color;

    fn sample_pie() -> PieArtist {
        PieArtist {
            sizes: vec![35.0, 25.0, 20.0, 15.0, 5.0],
            labels: None,
            colors: None,
            explode: None,
            autopct: false,
            start_angle: 90.0,
            radius: 1.0,
            label: None,
            color: Color::TAB_BLUE,
        }
    }

    #[test]
    fn builder_labels() {
        let mut p = sample_pie();
        p.labels(vec!["A", "B", "C", "D", "E"]);
        assert_eq!(p.labels.as_ref().unwrap().len(), 5);
        assert_eq!(p.labels.as_ref().unwrap()[0], "A");
    }

    #[test]
    fn builder_colors() {
        let mut p = sample_pie();
        p.colors(vec![Color::TAB_RED, Color::TAB_GREEN]);
        assert_eq!(p.colors.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn builder_explode() {
        let mut p = sample_pie();
        p.explode(vec![0.1, 0.0, 0.0, 0.0, 0.0]);
        assert!((p.explode.as_ref().unwrap()[0] - 0.1).abs() < f64::EPSILON);
        assert!((p.explode.as_ref().unwrap()[1] - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_autopct() {
        let mut p = sample_pie();
        assert!(!p.autopct);
        p.autopct(true);
        assert!(p.autopct);
    }

    #[test]
    fn builder_start_angle() {
        let mut p = sample_pie();
        p.start_angle(45.0);
        assert!((p.start_angle - 45.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_radius() {
        let mut p = sample_pie();
        p.radius(0.5);
        assert!((p.radius - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_label() {
        let mut p = sample_pie();
        p.label("my pie");
        assert_eq!(p.label.as_deref(), Some("my pie"));
    }

    #[test]
    fn builder_chaining() {
        let mut p = sample_pie();
        p.labels(vec!["A", "B", "C", "D", "E"])
            .autopct(true)
            .explode(vec![0.05, 0.0, 0.0, 0.0, 0.0])
            .start_angle(0.0)
            .radius(0.9)
            .label("chained");
        assert!(p.autopct);
        assert!((p.start_angle - 0.0).abs() < f64::EPSILON);
        assert!((p.radius - 0.9).abs() < f64::EPSILON);
        assert_eq!(p.label.as_deref(), Some("chained"));
        assert_eq!(p.labels.as_ref().unwrap()[0], "A");
    }

    #[test]
    fn data_bounds_default() {
        let p = sample_pie();
        let (xmin, xmax, ymin, ymax) = p.data_bounds();
        assert!((xmin - (-1.1)).abs() < f64::EPSILON);
        assert!((xmax - 1.1).abs() < f64::EPSILON);
        assert!((ymin - (-1.1)).abs() < f64::EPSILON);
        assert!((ymax - 1.1).abs() < f64::EPSILON);
    }

    #[test]
    fn data_bounds_custom_radius() {
        let mut p = sample_pie();
        p.radius(2.0);
        let (xmin, xmax, ymin, ymax) = p.data_bounds();
        assert!((xmin - (-2.2)).abs() < f64::EPSILON);
        assert!((xmax - 2.2).abs() < f64::EPSILON);
        assert!((ymin - (-2.2)).abs() < f64::EPSILON);
        assert!((ymax - 2.2).abs() < f64::EPSILON);
    }

    #[test]
    fn data_bounds_with_explode() {
        let mut p = sample_pie();
        p.explode(vec![0.2, 0.0, 0.0, 0.0, 0.0]);
        let (xmin, xmax, ymin, ymax) = p.data_bounds();
        // max_explode = 0.2, extent = 1.0 * (1.0 + 0.2) + 0.1 = 1.3
        assert!((xmin - (-1.3)).abs() < f64::EPSILON);
        assert!((xmax - 1.3).abs() < f64::EPSILON);
        assert!((ymin - (-1.3)).abs() < f64::EPSILON);
        assert!((ymax - 1.3).abs() < f64::EPSILON);
    }

    #[test]
    fn data_bounds_empty_sizes() {
        let p = PieArtist {
            sizes: vec![],
            labels: None,
            colors: None,
            explode: None,
            autopct: false,
            start_angle: 90.0,
            radius: 1.0,
            label: None,
            color: Color::TAB_BLUE,
        };
        let (xmin, xmax, ymin, ymax) = p.data_bounds();
        assert!((xmin - (-1.1)).abs() < f64::EPSILON);
        assert!((xmax - 1.1).abs() < f64::EPSILON);
        assert!((ymin - (-1.1)).abs() < f64::EPSILON);
        assert!((ymax - 1.1).abs() < f64::EPSILON);
    }
}
