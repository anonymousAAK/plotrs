//! Log-scale and semi-log plotting examples.

use plotkit::prelude::*;
use plotkit::FigureExt;

fn main() -> plotkit::Result<()> {
    let x: Vec<f64> = (1..=1000).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|v| v.powi(2)).collect();

    let mut fig = Figure::with_size(1000, 400);

    // Log-log plot
    let ax1 = fig.add_subplot(1, 2, 1);
    ax1.plot(&x, &y)?;
    ax1.set_xscale(Scale::Log10);
    ax1.set_yscale(Scale::Log10);
    ax1.set_title("Log-Log");
    ax1.set_xlabel("x");
    ax1.set_ylabel("y = x^2");
    ax1.grid(true);

    // Semilogy
    let ax2 = fig.add_subplot(1, 2, 2);
    ax2.plot(&x, &y)?;
    ax2.set_yscale(Scale::Log10);
    ax2.set_title("Semilog-Y");
    ax2.set_xlabel("x");
    ax2.set_ylabel("y = x^2");
    ax2.grid(true);

    std::fs::create_dir_all("examples/output").ok();
    fig.save("examples/output/15_logscale.png")?;
    Ok(())
}
