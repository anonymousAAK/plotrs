//! Log-log plot of a power law y = x^2 over four decades.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    // x spans [1, 1000] on a log scale; y = x^2 becomes a straight line
    // with slope 2 when both axes are logarithmic.
    let x: Vec<f64> = (0..100)
        .map(|i| 10f64.powf(i as f64 / 99.0 * 3.0))
        .collect();
    let y: Vec<f64> = x.iter().map(|&v| v * v).collect();

    ax.plot(&x, &y)?
        .color(Color::TAB_RED)
        .width(2.0)
        .label("y = x^2");

    ax.set_xscale(Scale::Log10);
    ax.set_yscale(Scale::Log10);
    ax.set_title("Log-Log Power Law");
    ax.set_xlabel("x");
    ax.set_ylabel("y");
    ax.grid(true);
    ax.legend();

    fig.save("examples/output/34_loglog.png")?;
    Ok(())
}
