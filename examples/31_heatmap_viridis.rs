//! Heatmap of a 10x10 ripple pattern rendered with the Viridis colormap.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);

    // Build a 10x10 ripple matrix: sin(i) * cos(j).
    let mut data: Vec<Vec<f64>> = Vec::with_capacity(10);
    for i in 0..10 {
        let mut row: Vec<f64> = Vec::with_capacity(10);
        for j in 0..10 {
            row.push((i as f64).sin() * (j as f64).cos());
        }
        data.push(row);
    }

    let ax = fig.add_subplot(1, 1, 1);
    ax.heatmap(data)?
        .colormap(Colormap::Viridis)
        .label("ripple");
    ax.set_title("Viridis Ripple Heatmap");

    fig.save("examples/output/31_heatmap_viridis.png")?;
    Ok(())
}
