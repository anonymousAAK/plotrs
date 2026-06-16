//! Symmetric-log y-axis spanning negative to positive values.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    // x sweeps across zero; y is a steeply growing odd function so the
    // values span several orders of magnitude on both sides of zero.
    let x: Vec<f64> = (0..200).map(|i| -5.0 + i as f64 * 0.05).collect();
    let y: Vec<f64> = x.iter().map(|&v| v * v * v).collect();

    ax.plot(&x, &y)?
        .color(Color::TAB_BLUE)
        .width(2.0)
        .label("y = x^3");

    ax.set_yscale(Scale::SymLog { linthresh: 1.0 });
    ax.set_title("SymLog Y-Axis");
    ax.set_xlabel("x");
    ax.set_ylabel("y (symlog)");
    ax.grid(true);
    ax.legend();

    fig.save("examples/output/33_symlog.png")?;
    Ok(())
}
