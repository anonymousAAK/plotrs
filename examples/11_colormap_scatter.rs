//! Scatter plot with colormap-driven per-point colors.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let n = 200;
    let x: Vec<f64> = (0..n).map(|i| i as f64 * 0.05).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    // Use the y value as the color dimension so the colormap follows the wave.
    let c: Vec<f64> = y.clone();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.scatter(&x, &y)?
        .c(c)
        .cmap(Colormap::Viridis)
        .size(8.0)
        .label("sin(x)");
    ax.set_title("Scatter with Viridis Colormap");
    ax.set_xlabel("x");
    ax.set_ylabel("y");
    ax.legend();
    fig.save("examples/output/11_colormap_scatter.png")?;
    Ok(())
}
