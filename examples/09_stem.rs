//! Stem chart example showing a discrete signal.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let x: Vec<f64> = (0..20).map(|i| i as f64 * 0.5).collect();
    let y: Vec<f64> = x.iter().map(|&v| (v * 0.8).sin() * 3.0).collect();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.stem(&x, &y)?
        .label("sin(0.8x)")
        .marker_size(8.0)
        .width(1.5);
    ax.set_title("Stem Chart");
    ax.set_xlabel("x");
    ax.set_ylabel("y");
    ax.legend();
    fig.save("examples/output/09_stem.png")?;
    Ok(())
}
