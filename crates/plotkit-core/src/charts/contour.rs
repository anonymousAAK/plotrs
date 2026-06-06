//! Contour and filled contour chart builder methods.
//!
//! Provides a fluent builder API for configuring [`ContourArtist`] instances.
//! Each method returns `&mut Self`, allowing calls to be chained together
//! for concise, readable chart construction.

use crate::artist::ContourArtist;
use crate::colormap::Colormap;
use crate::primitives::Color;

impl ContourArtist {
    /// Sets the colormap used to map contour levels to colors.
    pub fn colormap(&mut self, cmap: Colormap) -> &mut Self {
        self.cmap = cmap;
        self
    }

    /// Sets explicit contour levels.
    ///
    /// When `None`, levels are automatically computed by evenly spacing
    /// between the minimum and maximum finite z values.
    pub fn levels(&mut self, levels: Vec<f64>) -> &mut Self {
        self.levels = Some(levels);
        self
    }

    /// Sets the number of automatically computed contour levels.
    ///
    /// Only used when explicit levels are not set. Defaults to 10.
    pub fn num_levels(&mut self, n: usize) -> &mut Self {
        self.num_levels = n;
        self
    }

    /// Sets the line width for contour lines (unfilled mode).
    pub fn linewidths(&mut self, w: f64) -> &mut Self {
        self.linewidths = w;
        self
    }

    /// Sets explicit colors for contour levels, overriding the colormap.
    pub fn colors(&mut self, colors: Vec<Color>) -> &mut Self {
        self.colors = Some(colors);
        self
    }

    /// Sets the legend label.
    pub fn label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }

    /// Returns the effective contour levels.
    ///
    /// If explicit levels are set, returns those. Otherwise, computes
    /// `num_levels` evenly spaced levels between the minimum and maximum
    /// finite z values.
    pub fn effective_levels(&self) -> Vec<f64> {
        if let Some(ref lvls) = self.levels {
            return lvls.clone();
        }
        let (zmin, zmax) = self.z_bounds();
        if (zmax - zmin).abs() < f64::EPSILON {
            return vec![zmin];
        }
        let n = self.num_levels.max(2);
        (0..n)
            .map(|i| zmin + (zmax - zmin) * (i as f64) / ((n - 1) as f64))
            .collect()
    }

    /// Returns `(zmin, zmax)` from the finite z values.
    ///
    /// Falls back to `(0.0, 1.0)` when the data contains no finite values.
    pub fn z_bounds(&self) -> (f64, f64) {
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for row in &self.z {
            for &val in row {
                if val.is_finite() {
                    if val < lo {
                        lo = val;
                    }
                    if val > hi {
                        hi = val;
                    }
                }
            }
        }
        if lo.is_finite() && hi.is_finite() {
            (lo, hi)
        } else {
            (0.0, 1.0)
        }
    }

    /// Computes contour line segments for a single level using marching squares.
    ///
    /// Returns a list of line segments, each represented as `(x0, y0, x1, y1)`.
    pub fn marching_squares(&self, level: f64) -> Vec<(f64, f64, f64, f64)> {
        let ny = self.y.len();
        let nx = self.x.len();
        if ny < 2 || nx < 2 || self.z.len() < 2 {
            return Vec::new();
        }

        let mut segments = Vec::new();

        for j in 0..(ny - 1) {
            if j + 1 >= self.z.len() {
                break;
            }
            let row_bot = &self.z[j];
            let row_top = &self.z[j + 1];
            for i in 0..(nx - 1) {
                if i + 1 >= row_bot.len() || i + 1 >= row_top.len() {
                    break;
                }

                // Four corners of the cell:
                // bl = bottom-left, br = bottom-right, tr = top-right, tl = top-left
                let bl = row_bot[i];
                let br = row_bot[i + 1];
                let tr = row_top[i + 1];
                let tl = row_top[i];

                // Skip cells with NaN values.
                if !bl.is_finite() || !br.is_finite() || !tr.is_finite() || !tl.is_finite() {
                    continue;
                }

                // Classify corners: 1 if above level, 0 if below.
                let case = ((bl >= level) as u8)
                    | (((br >= level) as u8) << 1)
                    | (((tr >= level) as u8) << 2)
                    | (((tl >= level) as u8) << 3);

                // Cell coordinates.
                let x0 = self.x[i];
                let x1 = self.x[i + 1];
                let y0 = self.y[j];
                let y1 = self.y[j + 1];

                // Interpolation helper: fraction along edge where level crosses.
                let interp = |a: f64, b: f64| -> f64 {
                    if (b - a).abs() < f64::EPSILON {
                        0.5
                    } else {
                        (level - a) / (b - a)
                    }
                };

                // Edge midpoints (interpolated):
                // bottom: between bl and br
                // right:  between br and tr
                // top:    between tl and tr
                // left:   between bl and tl
                let bottom = || {
                    let t = interp(bl, br);
                    (x0 + t * (x1 - x0), y0)
                };
                let right = || {
                    let t = interp(br, tr);
                    (x1, y0 + t * (y1 - y0))
                };
                let top = || {
                    let t = interp(tl, tr);
                    (x0 + t * (x1 - x0), y1)
                };
                let left = || {
                    let t = interp(bl, tl);
                    (x0, y0 + t * (y1 - y0))
                };

                // 16 marching squares cases.
                match case {
                    0 | 15 => { /* entirely below or above -- no contour */ }
                    1 | 14 => {
                        let (ax, ay) = bottom();
                        let (bx, by) = left();
                        segments.push((ax, ay, bx, by));
                    }
                    2 | 13 => {
                        let (ax, ay) = bottom();
                        let (bx, by) = right();
                        segments.push((ax, ay, bx, by));
                    }
                    3 | 12 => {
                        let (ax, ay) = left();
                        let (bx, by) = right();
                        segments.push((ax, ay, bx, by));
                    }
                    4 | 11 => {
                        let (ax, ay) = right();
                        let (bx, by) = top();
                        segments.push((ax, ay, bx, by));
                    }
                    5 | 10 => {
                        // Ambiguous saddle point -- use simple decomposition
                        // (no saddle disambiguation for v0.4).
                        let (ax, ay) = bottom();
                        let (bx, by) = left();
                        segments.push((ax, ay, bx, by));
                        let (cx, cy) = right();
                        let (dx, dy) = top();
                        segments.push((cx, cy, dx, dy));
                    }
                    6 | 9 => {
                        let (ax, ay) = bottom();
                        let (bx, by) = top();
                        segments.push((ax, ay, bx, by));
                    }
                    7 | 8 => {
                        let (ax, ay) = left();
                        let (bx, by) = top();
                        segments.push((ax, ay, bx, by));
                    }
                    _ => unreachable!(),
                }
            }
        }

        segments
    }

    /// Computes the average z value for each grid cell.
    ///
    /// Returns a 2D vector of shape `[ny-1][nx-1]` where each element is the
    /// mean of the four corner z values. Used for the simplified filled
    /// contour approach.
    pub fn cell_averages(&self) -> Vec<Vec<f64>> {
        let ny = self.y.len();
        let nx = self.x.len();
        if ny < 2 || nx < 2 || self.z.len() < 2 {
            return Vec::new();
        }

        let mut avgs = Vec::with_capacity(ny - 1);
        for j in 0..(ny - 1) {
            if j + 1 >= self.z.len() {
                break;
            }
            let mut row = Vec::with_capacity(nx - 1);
            let row_bot = &self.z[j];
            let row_top = &self.z[j + 1];
            for i in 0..(nx - 1) {
                if i + 1 >= row_bot.len() || i + 1 >= row_top.len() {
                    break;
                }
                let bl = row_bot[i];
                let br = row_bot[i + 1];
                let tl = row_top[i];
                let tr = row_top[i + 1];
                let vals = [bl, br, tl, tr];
                let finite: Vec<f64> = vals.iter().copied().filter(|v| v.is_finite()).collect();
                let avg = if finite.is_empty() {
                    f64::NAN
                } else {
                    finite.iter().sum::<f64>() / finite.len() as f64
                };
                row.push(avg);
            }
            avgs.push(row);
        }
        avgs
    }
}

#[cfg(test)]
mod tests {
    use crate::artist::ContourArtist;
    use crate::colormap::Colormap;
    use crate::primitives::Color;

    fn sample_contour() -> ContourArtist {
        ContourArtist {
            x: vec![0.0, 1.0, 2.0],
            y: vec![0.0, 1.0, 2.0],
            z: vec![
                vec![0.0, 1.0, 2.0],
                vec![1.0, 2.0, 3.0],
                vec![2.0, 3.0, 4.0],
            ],
            levels: None,
            filled: false,
            cmap: Colormap::Viridis,
            colors: None,
            linewidths: 1.0,
            label: None,
            color: Color::TAB_BLUE,
            num_levels: 10,
        }
    }

    #[test]
    fn builder_colormap() {
        let mut c = sample_contour();
        c.colormap(Colormap::Plasma);
        assert_eq!(c.cmap, Colormap::Plasma);
    }

    #[test]
    fn builder_levels() {
        let mut c = sample_contour();
        c.levels(vec![1.0, 2.0, 3.0]);
        assert_eq!(c.levels, Some(vec![1.0, 2.0, 3.0]));
    }

    #[test]
    fn builder_num_levels() {
        let mut c = sample_contour();
        c.num_levels(5);
        assert_eq!(c.num_levels, 5);
    }

    #[test]
    fn builder_linewidths() {
        let mut c = sample_contour();
        c.linewidths(2.5);
        assert!((c.linewidths - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_colors() {
        let mut c = sample_contour();
        c.colors(vec![Color::TAB_RED, Color::TAB_BLUE]);
        assert_eq!(c.colors.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn builder_label() {
        let mut c = sample_contour();
        c.label("my contour");
        assert_eq!(c.label.as_deref(), Some("my contour"));
    }

    #[test]
    fn builder_chaining() {
        let mut c = sample_contour();
        c.colormap(Colormap::Plasma)
            .linewidths(2.0)
            .num_levels(5)
            .label("chained");
        assert_eq!(c.cmap, Colormap::Plasma);
        assert!((c.linewidths - 2.0).abs() < f64::EPSILON);
        assert_eq!(c.num_levels, 5);
        assert_eq!(c.label.as_deref(), Some("chained"));
    }

    #[test]
    fn z_bounds_basic() {
        let c = sample_contour();
        let (lo, hi) = c.z_bounds();
        assert!((lo - 0.0).abs() < f64::EPSILON);
        assert!((hi - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn z_bounds_empty() {
        let c = ContourArtist {
            x: vec![], y: vec![], z: vec![],
            levels: None, filled: false, cmap: Colormap::Viridis,
            colors: None, linewidths: 1.0, label: None,
            color: Color::TAB_BLUE, num_levels: 10,
        };
        let (lo, hi) = c.z_bounds();
        assert!((lo - 0.0).abs() < f64::EPSILON);
        assert!((hi - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn z_bounds_with_nan() {
        let c = ContourArtist {
            x: vec![0.0, 1.0],
            y: vec![0.0, 1.0],
            z: vec![
                vec![f64::NAN, 3.0],
                vec![1.0, f64::NAN],
            ],
            levels: None, filled: false, cmap: Colormap::Viridis,
            colors: None, linewidths: 1.0, label: None,
            color: Color::TAB_BLUE, num_levels: 10,
        };
        let (lo, hi) = c.z_bounds();
        assert!((lo - 1.0).abs() < f64::EPSILON);
        assert!((hi - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn effective_levels_auto() {
        let c = sample_contour();
        let levels = c.effective_levels();
        assert_eq!(levels.len(), 10);
        assert!((levels[0] - 0.0).abs() < f64::EPSILON);
        assert!((levels[9] - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn effective_levels_explicit() {
        let mut c = sample_contour();
        c.levels(vec![1.0, 2.0, 3.0]);
        let levels = c.effective_levels();
        assert_eq!(levels, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn effective_levels_constant_z() {
        let c = ContourArtist {
            x: vec![0.0, 1.0],
            y: vec![0.0, 1.0],
            z: vec![vec![5.0, 5.0], vec![5.0, 5.0]],
            levels: None, filled: false, cmap: Colormap::Viridis,
            colors: None, linewidths: 1.0, label: None,
            color: Color::TAB_BLUE, num_levels: 10,
        };
        let levels = c.effective_levels();
        assert_eq!(levels.len(), 1);
        assert!((levels[0] - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn data_bounds_basic() {
        let c = sample_contour();
        let (xmin, xmax, ymin, ymax) = c.data_bounds();
        assert!((xmin - 0.0).abs() < f64::EPSILON);
        assert!((xmax - 2.0).abs() < f64::EPSILON);
        assert!((ymin - 0.0).abs() < f64::EPSILON);
        assert!((ymax - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn data_bounds_empty() {
        let c = ContourArtist {
            x: vec![], y: vec![], z: vec![],
            levels: None, filled: false, cmap: Colormap::Viridis,
            colors: None, linewidths: 1.0, label: None,
            color: Color::TAB_BLUE, num_levels: 10,
        };
        assert_eq!(c.data_bounds(), (0.0, 1.0, 0.0, 1.0));
    }

    #[test]
    fn data_bounds_single_point() {
        let c = ContourArtist {
            x: vec![5.0], y: vec![3.0], z: vec![vec![1.0]],
            levels: None, filled: false, cmap: Colormap::Viridis,
            colors: None, linewidths: 1.0, label: None,
            color: Color::TAB_BLUE, num_levels: 10,
        };
        let (xmin, xmax, ymin, ymax) = c.data_bounds();
        assert!((xmin - 5.0).abs() < f64::EPSILON);
        assert!((xmax - 5.0).abs() < f64::EPSILON);
        assert!((ymin - 3.0).abs() < f64::EPSILON);
        assert!((ymax - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn marching_squares_no_contour() {
        let c = ContourArtist {
            x: vec![0.0, 1.0],
            y: vec![0.0, 1.0],
            z: vec![vec![0.0, 0.0], vec![0.0, 0.0]],
            levels: None, filled: false, cmap: Colormap::Viridis,
            colors: None, linewidths: 1.0, label: None,
            color: Color::TAB_BLUE, num_levels: 10,
        };
        let segs = c.marching_squares(1.0);
        assert!(segs.is_empty());
    }

    #[test]
    fn marching_squares_all_above() {
        let c = ContourArtist {
            x: vec![0.0, 1.0],
            y: vec![0.0, 1.0],
            z: vec![vec![5.0, 5.0], vec![5.0, 5.0]],
            levels: None, filled: false, cmap: Colormap::Viridis,
            colors: None, linewidths: 1.0, label: None,
            color: Color::TAB_BLUE, num_levels: 10,
        };
        let segs = c.marching_squares(1.0);
        assert!(segs.is_empty());
    }

    #[test]
    fn marching_squares_produces_segments() {
        let c = ContourArtist {
            x: vec![0.0, 1.0, 2.0],
            y: vec![0.0, 1.0, 2.0],
            z: vec![
                vec![0.0, 1.0, 2.0],
                vec![1.0, 2.0, 3.0],
                vec![2.0, 3.0, 4.0],
            ],
            levels: None, filled: false, cmap: Colormap::Viridis,
            colors: None, linewidths: 1.0, label: None,
            color: Color::TAB_BLUE, num_levels: 10,
        };
        let segs = c.marching_squares(1.5);
        assert!(!segs.is_empty(), "should produce at least one segment for level 1.5");
    }

    #[test]
    fn marching_squares_with_nan() {
        let c = ContourArtist {
            x: vec![0.0, 1.0],
            y: vec![0.0, 1.0],
            z: vec![vec![0.0, f64::NAN], vec![1.0, 2.0]],
            levels: None, filled: false, cmap: Colormap::Viridis,
            colors: None, linewidths: 1.0, label: None,
            color: Color::TAB_BLUE, num_levels: 10,
        };
        let segs = c.marching_squares(0.5);
        assert!(segs.is_empty());
    }

    #[test]
    fn marching_squares_small_grid() {
        let c = ContourArtist {
            x: vec![0.0],
            y: vec![0.0],
            z: vec![vec![1.0]],
            levels: None, filled: false, cmap: Colormap::Viridis,
            colors: None, linewidths: 1.0, label: None,
            color: Color::TAB_BLUE, num_levels: 10,
        };
        let segs = c.marching_squares(0.5);
        assert!(segs.is_empty());
    }

    #[test]
    fn cell_averages_basic() {
        let c = sample_contour();
        let avgs = c.cell_averages();
        assert_eq!(avgs.len(), 2);
        assert_eq!(avgs[0].len(), 2);
        // avg of (0, 1, 1, 2) = 1.0
        assert!((avgs[0][0] - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cell_averages_empty() {
        let c = ContourArtist {
            x: vec![], y: vec![], z: vec![],
            levels: None, filled: false, cmap: Colormap::Viridis,
            colors: None, linewidths: 1.0, label: None,
            color: Color::TAB_BLUE, num_levels: 10,
        };
        let avgs = c.cell_averages();
        assert!(avgs.is_empty());
    }

    #[test]
    fn cell_averages_with_nan() {
        let c = ContourArtist {
            x: vec![0.0, 1.0],
            y: vec![0.0, 1.0],
            z: vec![
                vec![f64::NAN, f64::NAN],
                vec![f64::NAN, f64::NAN],
            ],
            levels: None, filled: false, cmap: Colormap::Viridis,
            colors: None, linewidths: 1.0, label: None,
            color: Color::TAB_BLUE, num_levels: 10,
        };
        let avgs = c.cell_averages();
        assert_eq!(avgs.len(), 1);
        assert_eq!(avgs[0].len(), 1);
        assert!(avgs[0][0].is_nan());
    }

    #[test]
    fn marching_squares_diagonal_contour() {
        let c = ContourArtist {
            x: vec![0.0, 1.0],
            y: vec![0.0, 1.0],
            z: vec![
                vec![0.0, 1.0],
                vec![1.0, 2.0],
            ],
            levels: None, filled: false, cmap: Colormap::Viridis,
            colors: None, linewidths: 1.0, label: None,
            color: Color::TAB_BLUE, num_levels: 10,
        };
        let segs = c.marching_squares(0.5);
        assert_eq!(segs.len(), 1);
        // bl=0 < 0.5, br=1 >= 0.5, tr=2 >= 0.5, tl=1 >= 0.5
        // case = 0 | 2 | 4 | 8 = 14 -> same as case 1:
        // bottom edge and left edge
        let (x0, y0, x1, y1) = segs[0];
        // bottom edge: interp(0, 1) at level 0.5 -> t=0.5 -> x=0.5, y=0
        // left edge: interp(0, 1) at level 0.5 -> t=0.5 -> x=0, y=0.5
        assert!((x0 - 0.5).abs() < f64::EPSILON);
        assert!((y0 - 0.0).abs() < f64::EPSILON);
        assert!((x1 - 0.0).abs() < f64::EPSILON);
        assert!((y1 - 0.5).abs() < f64::EPSILON);
    }
}
