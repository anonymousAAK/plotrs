//! Error bar example showing measurement uncertainty.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    // Simulated measurement data with uncertainty.
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y = vec![2.3, 3.8, 4.1, 5.7, 6.2];
    let yerr = vec![0.4, 0.3, 0.5, 0.2, 0.6];
    let xerr = vec![0.1, 0.15, 0.1, 0.2, 0.1];

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    // Create error bar plot with symmetric y-errors.
    let mut eb = ax
        .errorbar(&x, &y)?
        .yerr_symmetric(&yerr)
        .xerr_symmetric(&xerr);
    eb.color(Color::TAB_BLUE)
        .label("Measurements")
        .cap_size(5.0)
        .line_width(1.5);
    ax.add_errorbar(eb);

    ax.set_title("Error Bar Plot");
    ax.set_xlabel("x");
    ax.set_ylabel("y");
    ax.legend();

    fig.save("examples/output/06_errorbar.png")?;
    Ok(())
}
