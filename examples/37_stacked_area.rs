//! Stacked area chart built from two fill_between layers.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();

    // Layer boundaries: baseline 0 -> y1 -> y2 (cumulative stack).
    let y0: Vec<f64> = vec![0.0; x.len()];
    let y1: Vec<f64> = x.iter().map(|&v| 1.0 + v.sin().abs()).collect();
    let y2: Vec<f64> = x
        .iter()
        .zip(&y1)
        .map(|(&v, &a)| a + 1.0 + (0.5 * v).cos().abs())
        .collect();

    // Bottom band: 0 .. y1
    ax.fill_between(&x, &y0, &y1)?
        .color(Color::TAB_BLUE)
        .alpha(0.6)
        .label("Series A");

    // Top band: y1 .. y2
    ax.fill_between(&x, &y1, &y2)?
        .color(Color::TAB_ORANGE)
        .alpha(0.6)
        .label("Series B");

    ax.set_title("Stacked Area Chart");
    ax.set_xlabel("x");
    ax.set_ylabel("cumulative value");
    ax.legend();

    fig.save("examples/output/37_stacked_area.png")?;
    Ok(())
}
