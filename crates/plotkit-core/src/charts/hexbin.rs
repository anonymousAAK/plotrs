//! Hexbin chart builder methods.
//!
//! Provides a fluent builder API for configuring [`HexbinArtist`] instances
//! and the hexagonal binning algorithm. Each builder method returns `&mut Self`,
//! allowing calls to be chained for concise, readable chart construction.

use crate::artist::HexbinArtist;
use crate::colormap::Colormap;
use crate::primitives::Color;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Builder methods
// ---------------------------------------------------------------------------

impl HexbinArtist {
    /// Sets the number of hexagons across the x-axis.
    pub fn gridsize(&mut self, gridsize: usize) -> &mut Self {
        self.gridsize = gridsize;
        self
    }

    /// Sets the colormap used to map bin counts to colors.
    pub fn colormap(&mut self, cmap: Colormap) -> &mut Self {
        self.cmap = cmap;
        self
    }

    /// Sets the minimum count threshold; bins with fewer points are hidden.
    pub fn mincnt(&mut self, mincnt: usize) -> &mut Self {
        self.mincnt = mincnt;
        self
    }

    /// Sets the opacity from 0.0 (fully transparent) to 1.0 (fully opaque).
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Sets the legend label.
    pub fn label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }

    /// Sets the hex edge (stroke) color.
    pub fn edgecolor(&mut self, color: Color) -> &mut Self {
        self.edgecolor = Some(color);
        self
    }

    /// Enables or disables auto-attaching a colorbar when this hexbin is drawn.
    pub fn colorbar(&mut self, show: bool) -> &mut Self {
        self.show_colorbar = show;
        self
    }
}

// ---------------------------------------------------------------------------
// Hexagonal binning algorithm
// ---------------------------------------------------------------------------

/// A hex cell key for the flat-top hexagonal grid.
///
/// We use axial coordinates (q, r) to identify each hexagon. Two `i64`
/// values give us exact, hash-friendly keys without floating-point
/// comparison issues.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct HexCell {
    /// Axial q-coordinate (column).
    pub q: i64,
    /// Axial r-coordinate (row).
    pub r: i64,
}

/// Result of binning data points into hexagonal cells.
#[derive(Debug, Clone)]
pub struct HexBinResult {
    /// Center (cx, cy) in data space and point count for each non-empty cell.
    pub cells: Vec<(f64, f64, usize)>,
    /// Maximum count across all cells.
    pub max_count: usize,
    /// Minimum count across all cells (among those >= mincnt).
    pub min_count: usize,
}

/// Computes the 6 vertices of a flat-top regular hexagon centered at
/// `(cx, cy)` with the given circumradius `size`.
///
/// Vertices are returned in counter-clockwise order starting from the
/// rightmost point (angle = 0).
pub fn hexagon_vertices(cx: f64, cy: f64, size: f64) -> [(f64, f64); 6] {
    let mut verts = [(0.0, 0.0); 6];
    for (i, vert) in verts.iter_mut().enumerate() {
        let angle = std::f64::consts::PI / 3.0 * i as f64;
        *vert = (cx + size * angle.cos(), cy + size * angle.sin());
    }
    verts
}

/// Bins `(x, y)` data points into a flat-top hexagonal grid.
///
/// The grid is sized so that roughly `gridsize` hexagons span the x-extent
/// of the data. Only cells with at least `mincnt` points are included in
/// the result.
///
/// # Algorithm
///
/// Uses the standard "nearest hex center" approach for flat-top hexagons:
///   1. Compute hex size (circumradius) from the data extent and `gridsize`.
///   2. For each point, convert to fractional axial coordinates then round
///      to the nearest hex cell using cube-coordinate rounding.
///   3. Accumulate counts in a hash map keyed by axial `(q, r)`.
///   4. Convert each cell's axial coordinates back to Cartesian centers.
pub fn bin_hexagonal(x: &[f64], y: &[f64], gridsize: usize, mincnt: usize) -> HexBinResult {
    if x.is_empty() || y.is_empty() {
        return HexBinResult {
            cells: Vec::new(),
            max_count: 0,
            min_count: 0,
        };
    }

    let (xmin, xmax, ymin, ymax) = data_extent(x, y);
    let x_range = (xmax - xmin).max(f64::EPSILON);
    let _y_range = (ymax - ymin).max(f64::EPSILON);

    // For flat-top hexagons:
    //   width  = 2 * size
    //   height = sqrt(3) * size
    // We want `gridsize` hexes across the x range.
    let size = x_range / (gridsize as f64 * 1.5 + 0.5);
    let size = size.max(f64::EPSILON);

    let sqrt3 = 3.0_f64.sqrt();

    // Bin each point.
    let mut counts: HashMap<HexCell, usize> = HashMap::new();

    let n = x.len().min(y.len());
    for i in 0..n {
        let px = x[i];
        let py = y[i];
        if !px.is_finite() || !py.is_finite() {
            continue;
        }

        // Convert Cartesian to fractional axial coordinates (flat-top).
        let fq = (2.0 / 3.0 * (px - xmin)) / size;
        let fr = (-(px - xmin) / 3.0 + sqrt3 / 3.0 * (py - ymin)) / size;

        // Round to nearest hex using cube-coordinate rounding.
        let (q, r) = axial_round(fq, fr);

        let cell = HexCell { q, r };
        *counts.entry(cell).or_insert(0) += 1;
    }

    // Filter by mincnt and compute centers.
    let mut cells = Vec::with_capacity(counts.len());
    let mut max_count = 0usize;
    let mut min_count = usize::MAX;

    for (cell, count) in &counts {
        if *count < mincnt {
            continue;
        }
        // Convert axial back to Cartesian.
        let cx = xmin + size * (3.0 / 2.0 * cell.q as f64);
        let cy = ymin + size * (sqrt3 / 2.0 * cell.q as f64 + sqrt3 * cell.r as f64);
        cells.push((cx, cy, *count));
        if *count > max_count {
            max_count = *count;
        }
        if *count < min_count {
            min_count = *count;
        }
    }

    if cells.is_empty() {
        min_count = 0;
    }

    // Sort cells for deterministic rendering order (bottom-left to top-right).
    cells.sort_by(|a, b| {
        a.1.partial_cmp(&b.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
    });

    HexBinResult {
        cells,
        max_count,
        min_count,
    }
}

/// Returns the hex circumradius for a given gridsize and x-extent.
///
/// This is exposed so the rendering pipeline can compute hex size from
/// the same parameters as the binning algorithm.
pub fn hex_size_for_gridsize(x_range: f64, gridsize: usize) -> f64 {
    let x_range = x_range.max(f64::EPSILON);
    let size = x_range / (gridsize as f64 * 1.5 + 0.5);
    size.max(f64::EPSILON)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Rounds fractional axial coordinates to the nearest hex cell using
/// cube-coordinate rounding.
fn axial_round(fq: f64, fr: f64) -> (i64, i64) {
    // Convert axial (q, r) to cube (x, y, z) where x=q, z=r, y=-x-z.
    let fx = fq;
    let fz = fr;
    let fy = -fx - fz;

    let mut rx = fx.round();
    let mut ry = fy.round();
    let mut rz = fz.round();

    let dx = (rx - fx).abs();
    let dy = (ry - fy).abs();
    let dz = (rz - fz).abs();

    if dx > dy && dx > dz {
        rx = -ry - rz;
    } else if dy > dz {
        ry = -rx - rz;
    } else {
        rz = -rx - ry;
    }

    let _ = ry; // y is implicit in axial coordinates
    (rx as i64, rz as i64)
}

/// Computes (xmin, xmax, ymin, ymax) for finite values in the data.
fn data_extent(x: &[f64], y: &[f64]) -> (f64, f64, f64, f64) {
    let mut xmin = f64::INFINITY;
    let mut xmax = f64::NEG_INFINITY;
    let mut ymin = f64::INFINITY;
    let mut ymax = f64::NEG_INFINITY;

    for &v in x {
        if v.is_finite() {
            if v < xmin {
                xmin = v;
            }
            if v > xmax {
                xmax = v;
            }
        }
    }
    for &v in y {
        if v.is_finite() {
            if v < ymin {
                ymin = v;
            }
            if v > ymax {
                ymax = v;
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
    if (xmax - xmin).abs() < f64::EPSILON {
        xmin -= 0.5;
        xmax += 0.5;
    }
    if (ymax - ymin).abs() < f64::EPSILON {
        ymin -= 0.5;
        ymax += 0.5;
    }

    (xmin, xmax, ymin, ymax)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artist::HexbinArtist;

    fn sample_hexbin() -> HexbinArtist {
        HexbinArtist {
            x: vec![0.0, 1.0, 2.0, 3.0, 4.0],
            y: vec![0.0, 1.0, 2.0, 3.0, 4.0],
            gridsize: 10,
            cmap: Colormap::Viridis,
            mincnt: 1,
            alpha: 1.0,
            color: Color::TAB_BLUE,
            label: None,
            edgecolor: None,
            show_colorbar: false,
        }
    }

    // -- Hexagon vertex tests --

    #[test]
    fn hexagon_vertices_count() {
        let verts = hexagon_vertices(0.0, 0.0, 1.0);
        assert_eq!(verts.len(), 6);
    }

    #[test]
    fn hexagon_vertices_symmetry() {
        let verts = hexagon_vertices(0.0, 0.0, 1.0);
        // First vertex should be at (1, 0) for flat-top hex at origin with size 1.
        assert!((verts[0].0 - 1.0).abs() < 1e-10);
        assert!((verts[0].1 - 0.0).abs() < 1e-10);
        // Opposite vertex (index 3) should be at (-1, 0).
        assert!((verts[3].0 - (-1.0)).abs() < 1e-10);
        assert!((verts[3].1 - 0.0).abs() < 1e-10);
    }

    #[test]
    fn hexagon_vertices_at_center() {
        let cx = 5.0;
        let cy = 3.0;
        let size = 2.0;
        let verts = hexagon_vertices(cx, cy, size);
        // First vertex: (cx + size, cy)
        assert!((verts[0].0 - (cx + size)).abs() < 1e-10);
        assert!((verts[0].1 - cy).abs() < 1e-10);
    }

    #[test]
    fn hexagon_vertices_equidistant_from_center() {
        let cx = 1.0;
        let cy = 2.0;
        let size = 3.0;
        let verts = hexagon_vertices(cx, cy, size);
        for (vx, vy) in &verts {
            let dist = ((vx - cx).powi(2) + (vy - cy).powi(2)).sqrt();
            assert!(
                (dist - size).abs() < 1e-10,
                "vertex ({vx}, {vy}) distance {dist} should equal size {size}"
            );
        }
    }

    // -- Binning algorithm tests --

    #[test]
    fn bin_empty_data() {
        let result = bin_hexagonal(&[], &[], 10, 1);
        assert!(result.cells.is_empty());
        assert_eq!(result.max_count, 0);
    }

    #[test]
    fn bin_single_point() {
        let result = bin_hexagonal(&[1.0], &[1.0], 10, 1);
        assert_eq!(result.cells.len(), 1);
        assert_eq!(result.cells[0].2, 1); // count = 1
    }

    #[test]
    fn bin_identical_points_same_cell() {
        // All 100 points at the same location should land in one cell.
        let x = vec![5.0; 100];
        let y = vec![5.0; 100];
        let result = bin_hexagonal(&x, &y, 10, 1);
        assert_eq!(result.cells.len(), 1);
        assert_eq!(result.cells[0].2, 100);
        assert_eq!(result.max_count, 100);
    }

    #[test]
    fn bin_count_accuracy() {
        // Place 50 points at (0,0) and 30 points at (100,100).
        let mut x = vec![0.0; 50];
        let mut y = vec![0.0; 50];
        x.extend(vec![100.0; 30]);
        y.extend(vec![100.0; 30]);
        let result = bin_hexagonal(&x, &y, 5, 1);

        // Total points across all cells should be 80.
        let total: usize = result.cells.iter().map(|c| c.2).sum();
        assert_eq!(total, 80);
        assert_eq!(result.max_count, 50);
    }

    #[test]
    fn bin_mincnt_filtering() {
        // Place 10 points at (0,0) and 1 point at (100,100) with mincnt=2.
        let mut x = vec![0.0; 10];
        let mut y = vec![0.0; 10];
        x.push(100.0);
        y.push(100.0);
        let result = bin_hexagonal(&x, &y, 5, 2);

        // The single-point cell should be filtered out.
        for cell in &result.cells {
            assert!(cell.2 >= 2, "all cells should have count >= 2");
        }
    }

    #[test]
    fn bin_gridsize_effect() {
        // With a larger gridsize, we should get more cells (finer grid).
        let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
        let y: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();

        let result_coarse = bin_hexagonal(&x, &y, 5, 1);
        let result_fine = bin_hexagonal(&x, &y, 20, 1);

        assert!(
            result_fine.cells.len() >= result_coarse.cells.len(),
            "finer grid ({}) should produce >= cells than coarse grid ({})",
            result_fine.cells.len(),
            result_coarse.cells.len()
        );
    }

    #[test]
    fn bin_nan_points_skipped() {
        let x = vec![1.0, f64::NAN, 3.0];
        let y = vec![1.0, 2.0, f64::NAN];
        let result = bin_hexagonal(&x, &y, 10, 1);
        // Only (1.0, 1.0) is valid.
        let total: usize = result.cells.iter().map(|c| c.2).sum();
        assert_eq!(total, 1);
    }

    // -- Data bounds tests --

    #[test]
    fn data_bounds_basic() {
        let h = HexbinArtist {
            x: vec![1.0, 2.0, 3.0],
            y: vec![10.0, 20.0, 30.0],
            gridsize: 10,
            cmap: Colormap::Viridis,
            mincnt: 1,
            alpha: 1.0,
            color: Color::TAB_BLUE,
            label: None,
            edgecolor: None,
            show_colorbar: false,
        };
        let (xmin, xmax, ymin, ymax) = h.data_bounds();
        assert!((xmin - 1.0).abs() < f64::EPSILON);
        assert!((xmax - 3.0).abs() < f64::EPSILON);
        assert!((ymin - 10.0).abs() < f64::EPSILON);
        assert!((ymax - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn data_bounds_empty() {
        let h = HexbinArtist {
            x: vec![],
            y: vec![],
            gridsize: 10,
            cmap: Colormap::Viridis,
            mincnt: 1,
            alpha: 1.0,
            color: Color::TAB_BLUE,
            label: None,
            edgecolor: None,
            show_colorbar: false,
        };
        assert_eq!(h.data_bounds(), (0.0, 1.0, 0.0, 1.0));
    }

    // -- Builder method tests --

    #[test]
    fn builder_gridsize() {
        let mut h = sample_hexbin();
        h.gridsize(30);
        assert_eq!(h.gridsize, 30);
    }

    #[test]
    fn builder_colormap() {
        let mut h = sample_hexbin();
        h.colormap(Colormap::Plasma);
        assert_eq!(h.cmap, Colormap::Plasma);
    }

    #[test]
    fn builder_mincnt() {
        let mut h = sample_hexbin();
        h.mincnt(5);
        assert_eq!(h.mincnt, 5);
    }

    #[test]
    fn builder_alpha() {
        let mut h = sample_hexbin();
        h.alpha(0.5);
        assert!((h.alpha - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_alpha_clamp() {
        let mut h = sample_hexbin();
        h.alpha(2.0);
        assert!((h.alpha - 1.0).abs() < f64::EPSILON);
        h.alpha(-0.5);
        assert!((h.alpha - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_label() {
        let mut h = sample_hexbin();
        h.label("hexbin test");
        assert_eq!(h.label.as_deref(), Some("hexbin test"));
    }

    #[test]
    fn builder_edgecolor() {
        let mut h = sample_hexbin();
        h.edgecolor(Color::BLACK);
        assert_eq!(h.edgecolor, Some(Color::BLACK));
    }

    #[test]
    fn builder_chaining() {
        let mut h = sample_hexbin();
        h.gridsize(15)
            .colormap(Colormap::Inferno)
            .mincnt(3)
            .alpha(0.7)
            .label("chained")
            .edgecolor(Color::WHITE);
        assert_eq!(h.gridsize, 15);
        assert_eq!(h.cmap, Colormap::Inferno);
        assert_eq!(h.mincnt, 3);
        assert!((h.alpha - 0.7).abs() < f64::EPSILON);
        assert_eq!(h.label.as_deref(), Some("chained"));
        assert_eq!(h.edgecolor, Some(Color::WHITE));
    }

    // -- Axial rounding tests --

    #[test]
    fn axial_round_at_origin() {
        let (q, r) = axial_round(0.0, 0.0);
        assert_eq!(q, 0);
        assert_eq!(r, 0);
    }

    #[test]
    fn axial_round_exact_integer() {
        let (q, r) = axial_round(3.0, -2.0);
        assert_eq!(q, 3);
        assert_eq!(r, -2);
    }

    #[test]
    fn axial_round_near_boundary() {
        // Values very close to 0.5 should round deterministically.
        let (q1, r1) = axial_round(0.49, 0.0);
        let (q2, r2) = axial_round(0.51, 0.0);
        // Both should resolve to valid hex cells.
        assert!(q1 == 0 || q1 == 1);
        assert!(q2 == 0 || q2 == 1);
        let _ = (r1, r2); // suppress unused warnings
    }

    // -- hex_size_for_gridsize tests --

    #[test]
    fn hex_size_positive() {
        let size = hex_size_for_gridsize(10.0, 20);
        assert!(size > 0.0);
    }

    #[test]
    fn hex_size_increases_with_range() {
        let s1 = hex_size_for_gridsize(5.0, 10);
        let s2 = hex_size_for_gridsize(10.0, 10);
        assert!(s2 > s1);
    }

    #[test]
    fn hex_size_decreases_with_gridsize() {
        let s1 = hex_size_for_gridsize(10.0, 5);
        let s2 = hex_size_for_gridsize(10.0, 20);
        assert!(s2 < s1);
    }
}
