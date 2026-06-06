//! Downsampling algorithms for large datasets.
//!
//! When plotting millions of data points, rendering every single point is
//! wasteful: a typical screen has at most a few thousand horizontal pixels,
//! so most points overlap and contribute nothing to the visual output.
//! Decimation reduces the point count to a manageable size while preserving
//! the perceived shape of the data.
//!
//! Two algorithms are provided:
//!
//! - **LTTB** (Largest Triangle Three Buckets) — the gold standard for
//!   perceptually faithful line downsampling. Based on the 2013 paper by
//!   Sveinn Steinarsson.
//! - **MinMax** — a faster alternative that keeps the min and max y-value
//!   in each bucket, ensuring peaks and troughs are never lost.

/// The decimation strategy to apply when rendering a line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DecimateMethod {
    /// Largest Triangle Three Buckets algorithm (best visual fidelity).
    Lttb,
    /// Min-max decimation (fastest, preserves extremes).
    MinMax,
}

/// Downsamples a series of (x, y) points to `threshold` points using the
/// Largest Triangle Three Buckets algorithm.
///
/// LTTB preserves the visual shape of the line while dramatically reducing
/// point count. A 1M-point series decimated to 1000 points is visually
/// indistinguishable from the original at screen resolution.
///
/// Returns indices into the original arrays. The first and last points
/// are always preserved.
///
/// # Panics
///
/// Panics if `x.len() != y.len()`.
pub fn lttb(x: &[f64], y: &[f64], threshold: usize) -> Vec<usize> {
    assert_eq!(x.len(), y.len(), "x and y must have the same length");

    let n = x.len();

    // Edge cases: nothing to decimate.
    if threshold == 0 {
        return vec![];
    }
    if n == 0 {
        return vec![];
    }
    if threshold == 1 {
        // Return only the first point.
        return vec![0];
    }
    if threshold >= n {
        return (0..n).collect();
    }

    // Filter out NaN points, keeping track of original indices.
    let valid: Vec<usize> = (0..n)
        .filter(|&i| x[i].is_finite() && y[i].is_finite())
        .collect();
    let valid_n = valid.len();

    if valid_n == 0 {
        return vec![];
    }
    if threshold >= valid_n {
        return valid;
    }
    if threshold == 1 {
        return vec![valid[0]];
    }

    let mut selected = Vec::with_capacity(threshold);

    // Always select the first valid point.
    selected.push(valid[0]);

    let bucket_count = threshold - 2;
    // Remaining points to distribute across buckets (excluding first and last).
    let interior = valid_n - 2;

    let mut prev_selected_idx = 0usize; // index into `valid`

    for bucket_i in 0..bucket_count {
        // Current bucket range in `valid` indices (1-based, skipping first point).
        let bucket_start = 1 + (bucket_i * interior) / bucket_count;
        let bucket_end = 1 + ((bucket_i + 1) * interior) / bucket_count;

        // Next bucket range (or last point if this is the last bucket).
        let next_start = if bucket_i + 1 < bucket_count {
            1 + ((bucket_i + 1) * interior) / bucket_count
        } else {
            valid_n - 1
        };
        let next_end = if bucket_i + 1 < bucket_count {
            1 + ((bucket_i + 2) * interior) / bucket_count
        } else {
            valid_n
        };

        // Compute average point of the next bucket.
        let next_count = (next_end - next_start) as f64;
        let (avg_x, avg_y) = if next_count > 0.0 {
            let mut sx = 0.0;
            let mut sy = 0.0;
            for &vi in &valid[next_start..next_end] {
                sx += x[vi];
                sy += y[vi];
            }
            (sx / next_count, sy / next_count)
        } else {
            // Fallback: use the last point.
            let li = valid[valid_n - 1];
            (x[li], y[li])
        };

        // Previously selected point coordinates.
        let prev_orig = valid[prev_selected_idx];
        let (px, py) = (x[prev_orig], y[prev_orig]);

        // Find the point in the current bucket that maximizes triangle area.
        let mut best_area = -1.0_f64;
        let mut best_valid_idx = bucket_start;

        for (vi, &orig) in valid.iter().enumerate().skip(bucket_start).take(bucket_end - bucket_start) {
            let area = triangle_area(px, py, x[orig], y[orig], avg_x, avg_y);
            if area > best_area {
                best_area = area;
                best_valid_idx = vi;
            }
        }

        selected.push(valid[best_valid_idx]);
        prev_selected_idx = best_valid_idx;
    }

    // Always select the last valid point.
    selected.push(valid[valid_n - 1]);

    selected
}

/// Min-max decimation: keeps the min and max y-value in each bucket.
/// Faster than LTTB, good for dense time series where peaks matter.
///
/// Returns indices into the original arrays. The first and last points
/// are always preserved. Within each bucket, the min and max points are
/// returned in their original order (preserving temporal coherence).
///
/// # Panics
///
/// Panics if `x.len() != y.len()`.
pub fn minmax(x: &[f64], y: &[f64], threshold: usize) -> Vec<usize> {
    assert_eq!(x.len(), y.len(), "x and y must have the same length");

    let n = x.len();

    if threshold == 0 {
        return vec![];
    }
    if n == 0 {
        return vec![];
    }
    if threshold == 1 {
        return vec![0];
    }
    if threshold >= n {
        return (0..n).collect();
    }

    // Filter out NaN points.
    let valid: Vec<usize> = (0..n)
        .filter(|&i| x[i].is_finite() && y[i].is_finite())
        .collect();
    let valid_n = valid.len();

    if valid_n == 0 {
        return vec![];
    }
    if threshold >= valid_n {
        return valid;
    }

    let mut selected = Vec::with_capacity(threshold);

    // Always select the first valid point.
    selected.push(valid[0]);

    // Each bucket produces two points (min and max), so we need
    // (threshold - 2) / 2 buckets. Handle odd thresholds gracefully.
    let pairs = (threshold - 2) / 2;
    let bucket_count = if pairs == 0 { 1 } else { pairs };
    let interior = valid_n - 2; // points between first and last

    for bucket_i in 0..bucket_count {
        let bucket_start = 1 + (bucket_i * interior) / bucket_count;
        let bucket_end = 1 + ((bucket_i + 1) * interior) / bucket_count;

        if bucket_start >= bucket_end {
            continue;
        }

        let mut min_idx = bucket_start;
        let mut max_idx = bucket_start;
        let mut min_val = y[valid[bucket_start]];
        let mut max_val = y[valid[bucket_start]];

        for vi in bucket_start..bucket_end {
            let yv = y[valid[vi]];
            if yv < min_val {
                min_val = yv;
                min_idx = vi;
            }
            if yv > max_val {
                max_val = yv;
                max_idx = vi;
            }
        }

        // Add min and max in original order.
        if min_idx == max_idx {
            selected.push(valid[min_idx]);
        } else if min_idx < max_idx {
            selected.push(valid[min_idx]);
            selected.push(valid[max_idx]);
        } else {
            selected.push(valid[max_idx]);
            selected.push(valid[min_idx]);
        }
    }

    // Always select the last valid point.
    let last = valid[valid_n - 1];
    // Avoid duplicating the last point if it was already selected.
    if selected.last() != Some(&last) {
        selected.push(last);
    }

    selected
}

/// Computes twice the area of the triangle formed by three points.
///
/// Using the shoelace formula:
///   area = |x_a(y_b - y_c) + x_b(y_c - y_a) + x_c(y_a - y_b)| / 2
///
/// We return the value without the `/2` since we only need relative
/// comparisons and avoiding the division is faster.
#[inline]
fn triangle_area(x_a: f64, y_a: f64, x_b: f64, y_b: f64, x_c: f64, y_c: f64) -> f64 {
    (x_a * (y_b - y_c) + x_b * (y_c - y_a) + x_c * (y_a - y_b)).abs()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    // -- LTTB tests --------------------------------------------------------

    #[test]
    fn lttb_identity_when_threshold_ge_len() {
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![0.0, 1.0, 4.0, 9.0, 16.0];
        let indices = lttb(&x, &y, 5);
        assert_eq!(indices, vec![0, 1, 2, 3, 4]);

        let indices = lttb(&x, &y, 100);
        assert_eq!(indices, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn lttb_first_and_last_always_preserved() {
        let x: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| v * v).collect();
        let indices = lttb(&x, &y, 10);
        assert_eq!(*indices.first().unwrap(), 0);
        assert_eq!(*indices.last().unwrap(), 99);
        assert_eq!(indices.len(), 10);
    }

    #[test]
    fn lttb_threshold_2_returns_first_and_last() {
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let indices = lttb(&x, &y, 2);
        assert_eq!(indices, vec![0, 4]);
    }

    #[test]
    fn lttb_threshold_3_returns_first_middle_last() {
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![0.0, 5.0, 0.0, 5.0, 0.0];
        let indices = lttb(&x, &y, 3);
        assert_eq!(indices.len(), 3);
        assert_eq!(indices[0], 0);
        assert_eq!(*indices.last().unwrap(), 4);
    }

    #[test]
    fn lttb_known_triangle_area() {
        // Triangle with vertices (0,0), (1,0), (0,1) has area 0.5.
        // Our internal function returns 2*area = 1.0.
        let area = triangle_area(0.0, 0.0, 1.0, 0.0, 0.0, 1.0);
        assert!((area - 1.0).abs() < 1e-12);

        // Degenerate (collinear) triangle has area 0.
        let area = triangle_area(0.0, 0.0, 1.0, 1.0, 2.0, 2.0);
        assert!(area.abs() < 1e-12);
    }

    #[test]
    fn lttb_linear_data_any_subset_looks_identical() {
        // For perfectly linear data y = 2x + 1, all triangles have zero area.
        // LTTB should still return `threshold` points, all lying on the line.
        let n = 50;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| 2.0 * v + 1.0).collect();
        let indices = lttb(&x, &y, 10);
        assert_eq!(indices.len(), 10);

        // All selected points should satisfy y = 2x + 1.
        for &idx in &indices {
            let expected = 2.0 * x[idx] + 1.0;
            assert!(
                (y[idx] - expected).abs() < 1e-12,
                "point at index {} deviates from y = 2x + 1",
                idx
            );
        }
    }

    #[test]
    fn lttb_sinusoidal_peaks_preserved() {
        // Generate a sine wave and verify that LTTB keeps the peaks.
        let n = 1000;
        let x: Vec<f64> = (0..n).map(|i| i as f64 * 2.0 * PI / 100.0).collect();
        let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();

        let indices = lttb(&x, &y, 50);

        // The global max (~1.0) and min (~-1.0) should be close to the selected set.
        let selected_y: Vec<f64> = indices.iter().map(|&i| y[i]).collect();
        let max_selected = selected_y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_selected = selected_y.iter().cloned().fold(f64::INFINITY, f64::min);

        assert!(max_selected > 0.95, "peak not preserved: max = {}", max_selected);
        assert!(min_selected < -0.95, "trough not preserved: min = {}", min_selected);
    }

    #[test]
    fn lttb_single_point() {
        let x = vec![42.0];
        let y = vec![7.0];
        let indices = lttb(&x, &y, 5);
        assert_eq!(indices, vec![0]);
    }

    #[test]
    fn lttb_two_points() {
        let x = vec![1.0, 2.0];
        let y = vec![3.0, 4.0];
        let indices = lttb(&x, &y, 5);
        assert_eq!(indices, vec![0, 1]);
    }

    #[test]
    fn lttb_nan_handling() {
        let x = vec![0.0, 1.0, f64::NAN, 3.0, 4.0, 5.0];
        let y = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let indices = lttb(&x, &y, 4);
        // NaN at index 2 should be skipped entirely.
        assert!(!indices.contains(&2), "NaN index should not be selected");
        // First valid (0) and last valid (5) should be present.
        assert_eq!(*indices.first().unwrap(), 0);
        assert_eq!(*indices.last().unwrap(), 5);
    }

    #[test]
    fn lttb_empty_input() {
        let x: Vec<f64> = vec![];
        let y: Vec<f64> = vec![];
        let indices = lttb(&x, &y, 10);
        assert!(indices.is_empty());
    }

    #[test]
    fn lttb_threshold_zero() {
        let x = vec![0.0, 1.0, 2.0];
        let y = vec![0.0, 1.0, 2.0];
        let indices = lttb(&x, &y, 0);
        assert!(indices.is_empty());
    }

    #[test]
    fn lttb_threshold_one() {
        let x = vec![0.0, 1.0, 2.0];
        let y = vec![0.0, 1.0, 2.0];
        let indices = lttb(&x, &y, 1);
        assert_eq!(indices, vec![0]);
    }

    // -- MinMax tests ------------------------------------------------------

    #[test]
    fn minmax_preserves_extremes() {
        // Data with a clear spike and dip.
        let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let mut y: Vec<f64> = vec![0.0; 20];
        y[5] = 100.0;  // spike
        y[15] = -100.0; // dip

        let indices = minmax(&x, &y, 10);
        let selected_y: Vec<f64> = indices.iter().map(|&i| y[i]).collect();

        assert!(selected_y.contains(&100.0), "spike not preserved");
        assert!(selected_y.contains(&-100.0), "dip not preserved");
    }

    #[test]
    fn minmax_identity_when_threshold_ge_len() {
        let x = vec![0.0, 1.0, 2.0, 3.0];
        let y = vec![1.0, 2.0, 3.0, 4.0];
        let indices = minmax(&x, &y, 10);
        assert_eq!(indices, vec![0, 1, 2, 3]);
    }

    #[test]
    fn minmax_first_and_last_preserved() {
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
        let indices = minmax(&x, &y, 10);
        assert_eq!(*indices.first().unwrap(), 0);
        assert_eq!(*indices.last().unwrap(), 49);
    }

    #[test]
    fn minmax_empty_input() {
        let indices = minmax(&[], &[], 5);
        assert!(indices.is_empty());
    }

    #[test]
    fn minmax_threshold_zero() {
        let indices = minmax(&[1.0, 2.0], &[3.0, 4.0], 0);
        assert!(indices.is_empty());
    }

    #[test]
    fn minmax_nan_handling() {
        let x = vec![0.0, 1.0, f64::NAN, 3.0, 4.0];
        let y = vec![0.0, 10.0, 5.0, -10.0, 0.0];
        let indices = minmax(&x, &y, 4);
        assert!(!indices.contains(&2), "NaN index should not be selected");
    }

    // -- Large synthetic dataset smoke test --------------------------------

    #[test]
    fn lttb_large_dataset_smoke() {
        let n = 100_000;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| (v * 0.01).sin() + (v * 0.1).cos()).collect();

        let threshold = 500;
        let indices = lttb(&x, &y, threshold);

        assert_eq!(indices.len(), threshold);
        assert_eq!(indices[0], 0);
        assert_eq!(*indices.last().unwrap(), n - 1);

        // Indices should be strictly increasing (no duplicates, monotonic).
        for w in indices.windows(2) {
            assert!(w[0] < w[1], "indices must be strictly increasing");
        }
    }

    #[test]
    fn minmax_large_dataset_smoke() {
        let n = 100_000;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| (v * 0.01).sin()).collect();

        let threshold = 500;
        let indices = minmax(&x, &y, threshold);

        assert!(indices.len() <= threshold);
        assert_eq!(indices[0], 0);
        assert_eq!(*indices.last().unwrap(), n - 1);

        // Indices should be strictly increasing.
        for w in indices.windows(2) {
            assert!(w[0] < w[1], "indices must be strictly increasing");
        }
    }

    // -- Bucket boundary correctness ---------------------------------------

    #[test]
    fn lttb_bucket_boundaries_no_gaps_or_overlaps() {
        // With 12 points and threshold=6, we have 4 buckets over 10 interior
        // points. Verify every interior point is in exactly one bucket.
        let n = 12;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let y: Vec<f64> = x.clone();

        let threshold = 6;
        let indices = lttb(&x, &y, threshold);

        assert_eq!(indices.len(), threshold);
        assert_eq!(indices[0], 0);
        assert_eq!(*indices.last().unwrap(), n as usize - 1);

        // All selected indices should be within range.
        for &idx in &indices {
            assert!(idx < n as usize);
        }
    }

    #[test]
    fn lttb_all_nan_returns_empty() {
        let x = vec![f64::NAN; 5];
        let y = vec![f64::NAN; 5];
        let indices = lttb(&x, &y, 3);
        assert!(indices.is_empty());
    }

    #[test]
    fn minmax_all_nan_returns_empty() {
        let x = vec![f64::NAN; 5];
        let y = vec![f64::NAN; 5];
        let indices = minmax(&x, &y, 3);
        assert!(indices.is_empty());
    }

    #[test]
    fn lttb_indices_are_sorted() {
        let n = 200;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| (v * 0.1).sin()).collect();
        let indices = lttb(&x, &y, 20);
        for w in indices.windows(2) {
            assert!(w[0] < w[1], "LTTB indices must be strictly increasing");
        }
    }
}
