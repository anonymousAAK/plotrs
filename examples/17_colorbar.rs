//! Heatmap with a colorbar showing the color-to-value mapping.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    // Generate a 2D grid of values (e.g. temperature distribution).
    let size = 10;
    let mut data = Vec::new();
    for row in 0..size {
        let mut row_data = Vec::new();
        for col in 0..size {
            let x = col as f64 / size as f64;
            let y = row as f64 / size as f64;
            let val = (x * std::f64::consts::PI).sin() * (y * std::f64::consts::PI).cos();
            row_data.push(val);
        }
        data.push(row_data);
    }

    let mut fig = Figure::with_size(700, 500);
    let ax = fig.add_subplot(1, 1, 1);
    ax.heatmap(data)?
        .colormap(Colormap::Viridis)
        .colorbar(true);
    ax.set_title("Heatmap with Colorbar");

    fig.save("examples/output/17_colorbar.png")?;
    Ok(())
}
