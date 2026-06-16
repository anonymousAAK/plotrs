//! Edge case tests for plotkit.
//!
//! Each test documents whether the edge case panics, returns an error,
//! or produces valid output.

use plotkit::{Figure, FigureExt};

// ============================================================================
// Helper: renders a Figure to SVG bytes so we can verify "produces output"
// without writing to disk.
// ============================================================================

fn render_svg(fig: &Figure) -> String {
    fig.to_svg_string().expect("SVG render should succeed")
}

// ============================================================================
// 1. Empty data — plot with empty x/y vectors
// ============================================================================

#[test]
fn edge_empty_data_plot() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let empty: Vec<f64> = vec![];
    let result = ax.plot(&empty, &empty);
    assert!(
        result.is_err(),
        "plot() with empty data should return an error"
    );
    match result.unwrap_err() {
        plotkit::error::PlotError::EmptyData => {} // expected
        other => panic!("Expected EmptyData, got: {:?}", other),
    }
}

#[test]
fn edge_empty_data_scatter() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let empty: Vec<f64> = vec![];
    let result = ax.scatter(&empty, &empty);
    assert!(
        result.is_err(),
        "scatter() with empty data should return an error"
    );
}

#[test]
fn edge_empty_data_bar() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let cats: Vec<&str> = vec![];
    let heights: Vec<f64> = vec![];
    let result = ax.bar(cats.as_slice(), &heights);
    assert!(
        result.is_err(),
        "bar() with empty data should return an error"
    );
}

#[test]
fn edge_empty_data_hist() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let empty: Vec<f64> = vec![];
    let result = ax.hist(&empty, 10);
    assert!(
        result.is_err(),
        "hist() with empty data should return an error"
    );
}

// ============================================================================
// 2. Single point — plot with exactly 1 data point
// ============================================================================

#[test]
fn edge_single_point_plot() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot([42.0], [7.0])
        .expect("single-point plot should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty(), "SVG output should not be empty");
    assert!(svg.contains("<svg"), "output should be valid SVG");
}

#[test]
fn edge_single_point_scatter() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    ax.scatter([1.0], [2.0])
        .expect("single-point scatter should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

#[test]
fn edge_single_point_hist() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    ax.hist([5.0], 10)
        .expect("single-point hist should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

// ============================================================================
// 3. Very large values — x/y values like 1e15
// ============================================================================

#[test]
fn edge_very_large_values() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let x = vec![1e15, 2e15, 3e15];
    let y = vec![4e15, 5e15, 6e15];
    ax.plot(&x, &y).expect("large-value plot should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty(), "should produce output with large values");
    assert!(svg.contains("<svg"), "should be valid SVG");
}

#[test]
fn edge_very_large_values_scatter() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    ax.scatter([1e15, 2e15], [3e15, 4e15])
        .expect("large-value scatter should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

// ============================================================================
// 4. Very small values — x/y values like 1e-15
// ============================================================================

#[test]
fn edge_very_small_values() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let x = vec![1e-15, 2e-15, 3e-15];
    let y = vec![4e-15, 5e-15, 6e-15];
    ax.plot(&x, &y).expect("small-value plot should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty(), "should produce output with small values");
}

#[test]
fn edge_very_small_values_hist() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let data = vec![1e-15, 2e-15, 3e-15, 4e-15, 5e-15];
    ax.hist(&data, 5).expect("small-value hist should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

// ============================================================================
// 5. Negative values — all negative data
// ============================================================================

#[test]
fn edge_all_negative_plot() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let x = vec![-10.0, -5.0, -1.0];
    let y = vec![-100.0, -50.0, -10.0];
    ax.plot(&x, &y).expect("negative-value plot should succeed");
    let svg = render_svg(&fig);
    assert!(
        !svg.is_empty(),
        "should produce output with negative values"
    );
}

#[test]
fn edge_all_negative_bar() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let cats: &[&str] = &["a", "b", "c"];
    let heights = vec![-3.0, -5.0, -1.0];
    ax.bar(cats, &heights)
        .expect("negative-height bar should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

#[test]
fn edge_all_negative_hist() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let data = vec![-10.0, -5.0, -3.0, -1.0, -0.5];
    ax.hist(&data, 5)
        .expect("negative-value hist should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

// ============================================================================
// 6. NaN/Inf values — data containing f64::NAN or f64::INFINITY
// ============================================================================

#[test]
fn edge_nan_inf_plot() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let x = vec![1.0, f64::NAN, 3.0, f64::INFINITY, 5.0];
    let y = vec![2.0, 4.0, f64::NEG_INFINITY, 6.0, f64::NAN];
    ax.plot(&x, &y)
        .expect("NaN/Inf plot should succeed (data accepted)");
    // Rendering should not panic even with NaN/Inf values
    let svg = render_svg(&fig);
    assert!(!svg.is_empty(), "should produce output even with NaN/Inf");
}

#[test]
fn edge_all_nan_plot() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let x = vec![f64::NAN, f64::NAN, f64::NAN];
    let y = vec![f64::NAN, f64::NAN, f64::NAN];
    ax.plot(&x, &y)
        .expect("all-NaN plot should succeed (data accepted)");
    // Rendering should not panic
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

#[test]
fn edge_all_inf_plot() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let x = vec![f64::INFINITY, f64::NEG_INFINITY];
    let y = vec![f64::INFINITY, f64::NEG_INFINITY];
    ax.plot(&x, &y)
        .expect("all-Inf plot should succeed (data accepted)");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

#[test]
fn edge_nan_inf_hist() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    // Mix of finite and non-finite values. hist should succeed on finite subset.
    let data = vec![1.0, f64::NAN, 3.0, f64::INFINITY, 5.0];
    ax.hist(&data, 5).expect("hist with NaN/Inf should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

#[test]
fn edge_all_nan_hist() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    // All NaN: series is not empty (len=3), but no finite values.
    // hist may behave differently here -- let's find out.
    let data = vec![f64::NAN, f64::NAN, f64::NAN];
    // This should NOT panic regardless of whether it returns Ok or Err.
    let result = ax.hist(&data, 5);
    // Document the behavior:
    match result {
        Ok(_) => {
            // If it succeeds, rendering should not panic
            let svg = render_svg(&fig);
            assert!(!svg.is_empty());
        }
        Err(e) => {
            eprintln!("all-NaN hist returned error (acceptable): {:?}", e);
        }
    }
}

// ============================================================================
// 7. Zero-size figure — Figure with width=0 or height=0
// ============================================================================

#[test]
fn edge_zero_width_figure() {
    let mut fig = Figure::with_size(0, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot([1.0, 2.0], [3.0, 4.0]).unwrap();
    // Should not panic during rendering
    let svg = fig.to_svg_string().expect("zero-width SVG should not fail");
    assert!(svg.contains("<svg"), "should still produce SVG markup");
}

#[test]
fn edge_zero_height_figure() {
    let mut fig = Figure::with_size(800, 0);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot([1.0, 2.0], [3.0, 4.0]).unwrap();
    let svg = fig
        .to_svg_string()
        .expect("zero-height SVG should not fail");
    assert!(svg.contains("<svg"));
}

#[test]
fn edge_zero_both_figure() {
    let mut fig = Figure::with_size(0, 0);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot([1.0, 2.0], [3.0, 4.0]).unwrap();
    let svg = fig.to_svg_string().expect("zero-size SVG should not fail");
    assert!(svg.contains("<svg"));
}

// ============================================================================
// 8. Huge data — 100,000 data points
// ============================================================================

#[test]
fn edge_huge_data_plot() {
    let n = 100_000;
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&x, &y).expect("100k-point plot should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty(), "should produce SVG for 100k points");
    // SVG should be substantial (many path segments)
    assert!(svg.len() > 1000, "SVG for 100k points should be sizable");
}

#[test]
fn edge_huge_data_scatter() {
    let n = 100_000;
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|v| v.cos()).collect();
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    ax.scatter(&x, &y)
        .expect("100k-point scatter should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

#[test]
fn edge_huge_data_hist() {
    let n = 100_000;
    let data: Vec<f64> = (0..n).map(|i| (i as f64 * 0.001).sin()).collect();
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    ax.hist(&data, 50).expect("100k-point hist should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

// ============================================================================
// 9. Mismatched lengths — x has 10 elements, y has 5
// ============================================================================

#[test]
fn edge_mismatched_lengths_plot() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..5).map(|i| i as f64).collect();
    let result = ax.plot(&x, &y);
    assert!(result.is_err(), "mismatched lengths should return an error");
    match result.unwrap_err() {
        plotkit::error::PlotError::SeriesLengthMismatch { expected, got } => {
            assert_eq!(expected, 10);
            assert_eq!(got, 5);
        }
        other => panic!("Expected SeriesLengthMismatch, got: {:?}", other),
    }
}

#[test]
fn edge_mismatched_lengths_scatter() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..5).map(|i| i as f64).collect();
    let result = ax.scatter(&x, &y);
    assert!(result.is_err());
    match result.unwrap_err() {
        plotkit::error::PlotError::SeriesLengthMismatch {
            expected: 10,
            got: 5,
        } => {}
        other => panic!("Expected SeriesLengthMismatch(10,5), got: {:?}", other),
    }
}

#[test]
fn edge_mismatched_lengths_bar() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let cats: &[&str] = &["a", "b", "c"];
    let heights = vec![1.0, 2.0]; // only 2 heights for 3 categories
    let result = ax.bar(cats, &heights);
    assert!(result.is_err());
}

// ============================================================================
// 10. Unicode text — title/labels with Unicode chars like "μ ± σ"
// ============================================================================

#[test]
fn edge_unicode_title() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot([1.0, 2.0, 3.0], [1.0, 4.0, 9.0]).unwrap();
    ax.set_title("μ ± σ — Gaussian distribution (日本語)");
    ax.set_xlabel("Δx (μm)");
    ax.set_ylabel("Ω (Ω·m⁻¹)");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty(), "Unicode labels should not break rendering");
    // Check that Unicode text appears in the SVG (possibly HTML-escaped)
    assert!(
        svg.contains("Gaussian") || svg.contains("μ"),
        "Unicode text should appear in SVG"
    );
}

#[test]
fn edge_unicode_suptitle() {
    let mut fig = Figure::new();
    fig.suptitle("Ωmega — α β γ δ ε ζ η θ ι κ λ μ");
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot([1.0, 2.0], [1.0, 2.0]).unwrap();
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

#[test]
fn edge_unicode_legend_labels() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot([1.0, 2.0], [1.0, 2.0]).unwrap().label("α-series");
    ax.plot([1.0, 2.0], [2.0, 3.0]).unwrap().label("β-series");
    ax.legend();
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

// ============================================================================
// 11. Empty string labels — empty title, xlabel, ylabel
// ============================================================================

#[test]
fn edge_empty_string_labels() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot([1.0, 2.0, 3.0], [1.0, 4.0, 9.0]).unwrap();
    ax.set_title("");
    ax.set_xlabel("");
    ax.set_ylabel("");
    let svg = render_svg(&fig);
    assert!(
        !svg.is_empty(),
        "empty-string labels should not break rendering"
    );
}

#[test]
fn edge_empty_string_suptitle() {
    let mut fig = Figure::new();
    fig.suptitle("");
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot([1.0, 2.0], [1.0, 2.0]).unwrap();
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

// ============================================================================
// 12. Multiple subplots — 3x3 grid of subplots
// ============================================================================

#[test]
fn edge_3x3_subplots() {
    let mut fig = Figure::new();
    for i in 1..=9 {
        let ax = fig.add_subplot(3, 3, i);
        let x = vec![0.0, 1.0, 2.0];
        let y = vec![i as f64, (i * 2) as f64, (i * 3) as f64];
        ax.plot(&x, &y).expect("subplot plot should succeed");
        ax.set_title(&format!("Plot {}", i));
    }
    let svg = render_svg(&fig);
    assert!(!svg.is_empty(), "3x3 grid should produce valid SVG");
    // Should have at least 9 plot areas (clip paths)
    assert!(svg.len() > 500, "3x3 SVG should be substantial");
}

#[test]
fn edge_subplots_mixed_chart_types() {
    let mut fig = Figure::with_size(1200, 900);
    // Subplot 1: line
    let ax1 = fig.add_subplot(2, 2, 1);
    ax1.plot([1.0, 2.0, 3.0], [1.0, 4.0, 9.0]).unwrap();
    // Subplot 2: scatter
    let ax2 = fig.add_subplot(2, 2, 2);
    ax2.scatter([1.0, 2.0, 3.0], [9.0, 4.0, 1.0]).unwrap();
    // Subplot 3: bar
    let ax3 = fig.add_subplot(2, 2, 3);
    ax3.bar(&["a", "b", "c"][..], [3.0, 7.0, 5.0]).unwrap();
    // Subplot 4: hist
    let ax4 = fig.add_subplot(2, 2, 4);
    ax4.hist([1.0, 2.0, 2.0, 3.0, 3.0, 3.0], 3).unwrap();
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

// ============================================================================
// 13. Bar chart with 0 values — all bars height 0
// ============================================================================

#[test]
fn edge_bar_all_zero_heights() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let cats: &[&str] = &["a", "b", "c", "d"];
    let heights = vec![0.0, 0.0, 0.0, 0.0];
    ax.bar(cats, &heights)
        .expect("zero-height bars should succeed");
    let svg = render_svg(&fig);
    assert!(
        !svg.is_empty(),
        "zero-height bar chart should produce valid SVG"
    );
}

#[test]
fn edge_bar_mixed_zero_and_nonzero() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let cats: &[&str] = &["a", "b", "c"];
    let heights = vec![0.0, 5.0, 0.0];
    ax.bar(cats, &heights)
        .expect("mixed zero/non-zero bars should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

// ============================================================================
// 14. Histogram with single value — all values the same
// ============================================================================

#[test]
fn edge_hist_all_same_value() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let data = vec![5.0, 5.0, 5.0, 5.0, 5.0];
    ax.hist(&data, 10)
        .expect("all-same-value hist should succeed");
    let svg = render_svg(&fig);
    assert!(
        !svg.is_empty(),
        "constant-data histogram should produce valid SVG"
    );
}

#[test]
fn edge_hist_all_same_value_single_bin() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let data = vec![42.0; 100];
    ax.hist(&data, 1)
        .expect("all-same-value, 1 bin should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

#[test]
fn edge_hist_all_same_value_many_bins() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    let data = vec![3.125; 50];
    ax.hist(&data, 100)
        .expect("all-same-value, 100 bins should succeed");
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}

// ============================================================================
// Additional edge cases worth checking
// ============================================================================

#[test]
fn edge_figure_no_axes_render() {
    // Render a figure with no axes at all
    let fig = Figure::new();
    let svg = render_svg(&fig);
    assert!(
        !svg.is_empty(),
        "empty figure should still produce SVG wrapper"
    );
    assert!(svg.contains("<svg"));
}

#[test]
fn edge_axes_no_data_render() {
    // Axes exist but have no artists
    let mut fig = Figure::new();
    fig.add_subplot(1, 1, 1);
    let svg = render_svg(&fig);
    assert!(
        !svg.is_empty(),
        "axes with no data should render background/spines"
    );
}

#[test]
fn edge_two_points_plot() {
    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot([0.0, 1.0], [0.0, 1.0]).unwrap();
    let svg = render_svg(&fig);
    assert!(!svg.is_empty());
}
