//! Scatter plot colored by a value dimension using the Turbo colormap.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    let n = 300;
    let x: Vec<f64> = (0..n).map(|i| i as f64 * 0.05).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.sin() + 0.3 * (2.0 * v).cos()).collect();
    // Color each point by its distance from the origin.
    let values: Vec<f64> = x
        .iter()
        .zip(&y)
        .map(|(&a, &b)| (a * a + b * b).sqrt())
        .collect();

    ax.scatter(&x, &y)?
        .c(values)
        .cmap(Colormap::Turbo)
        .size(8.0)
        .label("samples");

    ax.set_title("Scatter Colored by Value");
    ax.set_xlabel("x");
    ax.set_ylabel("y");

    fig.save("examples/output/38_scatter_colorbar.png")?;
    Ok(())
}
