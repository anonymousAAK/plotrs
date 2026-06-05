//! Golden snapshot tests for plotkit charts.
//!
//! Each test renders a chart to PNG bytes and asserts the hash matches
//! a committed snapshot. Run `cargo insta review` to update snapshots
//! when visuals change intentionally.

use plotkit::prelude::*;
use plotkit::FigureExt;

fn render_to_bytes(fig: &plotkit::Figure) -> Vec<u8> {
    fig.to_png_bytes().expect("render failed")
}

fn hash_png(bytes: &[u8]) -> String {
    // Simple hash: use the length + first/last bytes as a fingerprint
    format!(
        "png_{}_{:02x}{:02x}",
        bytes.len(),
        bytes[0],
        bytes[bytes.len() - 1]
    )
}

#[test]
fn snapshot_line_default() {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&x, &y).unwrap().label("sin(x)");
    ax.set_title("Line Plot");
    ax.set_xlabel("x");
    ax.set_ylabel("y");
    let bytes = render_to_bytes(&fig);
    insta::assert_snapshot!(hash_png(&bytes));
}

#[test]
fn snapshot_scatter_default() {
    let x: Vec<f64> = (0..50).map(|i| i as f64 * 0.2).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.cos()).collect();
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.scatter(&x, &y).unwrap();
    ax.set_title("Scatter");
    let bytes = render_to_bytes(&fig);
    insta::assert_snapshot!(hash_png(&bytes));
}

#[test]
fn snapshot_bar_default() {
    let cats = vec!["A", "B", "C", "D"];
    let vals = vec![10.0, 25.0, 15.0, 30.0];
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.bar(cats.as_slice(), &vals).unwrap();
    ax.set_title("Bar Chart");
    let bytes = render_to_bytes(&fig);
    insta::assert_snapshot!(hash_png(&bytes));
}

#[test]
fn snapshot_histogram_default() {
    // Deterministic data
    let data: Vec<f64> = (0..200)
        .map(|i| (i as f64 * 0.37).sin() * 3.0 + 5.0)
        .collect();
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.hist(&data, 20).unwrap();
    ax.set_title("Histogram");
    let bytes = render_to_bytes(&fig);
    insta::assert_snapshot!(hash_png(&bytes));
}

#[test]
fn snapshot_fill_between_default() {
    let x: Vec<f64> = (0..80).map(|i| i as f64 * 0.1).collect();
    let y1: Vec<f64> = x.iter().map(|&v| v.sin() - 0.3).collect();
    let y2: Vec<f64> = x.iter().map(|&v| v.sin() + 0.3).collect();
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.fill_between(&x, &y1, &y2).unwrap().alpha(0.4);
    ax.set_title("Fill Between");
    let bytes = render_to_bytes(&fig);
    insta::assert_snapshot!(hash_png(&bytes));
}

#[test]
fn snapshot_multi_series_with_legend() {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y1: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    let y2: Vec<f64> = x.iter().map(|&v| v.cos()).collect();
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&x, &y1).unwrap().label("sin(x)");
    ax.plot(&x, &y2).unwrap().label("cos(x)");
    ax.set_title("Multi-Series");
    ax.legend();
    let bytes = render_to_bytes(&fig);
    insta::assert_snapshot!(hash_png(&bytes));
}

#[test]
fn snapshot_dark_theme() {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    let mut fig = Figure::with_size(800, 600);
    fig.set_theme(Theme::dark());
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&x, &y).unwrap();
    ax.set_title("Dark Theme");
    let bytes = render_to_bytes(&fig);
    insta::assert_snapshot!(hash_png(&bytes));
}
