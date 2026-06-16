//! Contour and filled contour example showing a 2D Gaussian function.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    // Create a 2D grid.
    let n = 50;
    let x: Vec<f64> = (0..n)
        .map(|i| -3.0 + 6.0 * i as f64 / (n - 1) as f64)
        .collect();
    let y: Vec<f64> = x.clone();

    // Compute z = exp(-(x^2 + y^2)) -- a 2D Gaussian.
    let z: Vec<Vec<f64>> = y
        .iter()
        .map(|&yi| x.iter().map(|&xi| (-xi * xi - yi * yi).exp()).collect())
        .collect();

    // --- Filled contour (contourf) ---
    let mut fig = Figure::with_size(800, 400);
    let ax1 = fig.add_subplot(1, 2, 1);
    ax1.contourf(&x, &y, z.clone())?
        .colormap(Colormap::Viridis)
        .num_levels(12)
        .label("filled");
    ax1.set_title("contourf");
    ax1.set_xlabel("x");
    ax1.set_ylabel("y");

    // --- Contour lines ---
    let ax2 = fig.add_subplot(1, 2, 2);
    ax2.contour(&x, &y, z)?
        .colormap(Colormap::Plasma)
        .num_levels(10)
        .linewidths(1.5)
        .label("contour");
    ax2.set_title("contour");
    ax2.set_xlabel("x");
    ax2.set_ylabel("y");

    fig.save("examples/output/17_contour.png")?;
    Ok(())
}
