//! Line plot with custom x-tick positions and quarter labels.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    // Quarterly revenue (one value per quarter).
    let x: Vec<f64> = vec![0.0, 1.0, 2.0, 3.0];
    let y: Vec<f64> = vec![120.0, 145.0, 138.0, 172.0];

    ax.plot(x.clone(), y.clone())?
        .color(Color::TAB_GREEN)
        .width(2.0)
        .label("revenue");

    ax.set_xticks(&[0.0, 1.0, 2.0, 3.0]);
    ax.set_xticklabels(&["Q1", "Q2", "Q3", "Q4"]);

    ax.set_title("Quarterly Revenue");
    ax.set_xlabel("Quarter");
    ax.set_ylabel("Revenue ($k)");
    ax.legend();
    ax.grid(true);
    fig.save("examples/output/43_custom_ticklabels.png")?;
    Ok(())
}
