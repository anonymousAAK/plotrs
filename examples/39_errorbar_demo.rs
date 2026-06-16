//! Error bar plot showing measurements with symmetric y-error.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    let x: Vec<f64> = (0..8).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&v| 2.0 * v + 1.0).collect();
    let yerr: Vec<f64> = x.iter().map(|&v| 0.5 + 0.2 * v).collect();

    // `errorbar` returns an owned artist; configure it, then add it.
    let eb = ax.errorbar(x.clone(), y.clone())?.yerr_symmetric(&yerr);
    // After `yerr_symmetric` (which consumes self), chain the &mut-self builders.
    let mut eb = eb;
    eb.cap_size(5.0)
        .line_width(1.5)
        .color(Color::TAB_RED)
        .label("Measurements");
    ax.add_errorbar(eb);

    ax.set_title("Error Bar Demo");
    ax.set_xlabel("x");
    ax.set_ylabel("y");
    ax.legend();
    ax.grid(true);
    fig.save("examples/output/39_errorbar_demo.png")?;
    Ok(())
}
