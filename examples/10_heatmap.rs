//! Heatmap example showing a correlation matrix.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    // Simulated correlation matrix (4x4).
    let data = vec![
        vec![1.00, 0.85, -0.30, 0.42],
        vec![0.85, 1.00, -0.15, 0.60],
        vec![-0.30, -0.15, 1.00, -0.50],
        vec![0.42, 0.60, -0.50, 1.00],
    ];

    let labels: Vec<String> = vec!["A", "B", "C", "D"]
        .into_iter()
        .map(String::from)
        .collect();

    let mut fig = Figure::with_size(600, 500);
    let ax = fig.add_subplot(1, 1, 1);
    ax.heatmap(data)?
        .colormap(Colormap::Coolwarm)
        .vmin(-1.0)
        .vmax(1.0)
        .show_values(true)
        .x_labels(labels.clone())
        .y_labels(labels)
        .label("correlation");
    ax.set_title("Correlation Matrix");
    fig.save("examples/output/10_heatmap.png")?;
    Ok(())
}
