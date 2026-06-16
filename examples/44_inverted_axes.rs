//! Line plot with an inverted y-axis (larger values at the bottom).

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    // Depth profile: depth increases downward, so invert the y-axis.
    let x: Vec<f64> = (0..50).map(|i| i as f64 * 0.2).collect();
    let depth: Vec<f64> = x.iter().map(|&v| 2.0 + v * 0.8 + (v).sin()).collect();

    ax.plot(x.clone(), depth.clone())?
        .color(Color::TAB_BLUE)
        .width(2.0)
        .label("depth");

    ax.invert_yaxis();

    ax.set_title("Inverted Y-Axis (Depth Profile)");
    ax.set_xlabel("Distance");
    ax.set_ylabel("Depth");
    ax.legend();
    ax.grid(true);
    fig.save("examples/output/44_inverted_axes.png")?;
    Ok(())
}
