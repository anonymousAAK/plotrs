//! Heatmap of a smooth gradient matrix rendered with the Plasma colormap.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);

    // Build a 12x12 gradient matrix increasing toward the bottom-right corner.
    let mut data: Vec<Vec<f64>> = Vec::with_capacity(12);
    for i in 0..12 {
        let mut row: Vec<f64> = Vec::with_capacity(12);
        for j in 0..12 {
            row.push(i as f64 + j as f64);
        }
        data.push(row);
    }

    let ax = fig.add_subplot(1, 1, 1);
    ax.heatmap(data)?.colormap(Colormap::Plasma);
    ax.set_title("Plasma Gradient Heatmap");

    fig.save("examples/output/32_heatmap_plasma.png")?;
    Ok(())
}
