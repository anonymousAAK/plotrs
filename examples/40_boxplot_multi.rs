//! Box-and-whisker plot comparing three datasets.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    // Three datasets with different spreads and centers.
    let group_a: Vec<f64> = (0..40)
        .map(|i| 5.0 + (i as f64 * 0.37).sin() * 2.0)
        .collect();
    let group_b: Vec<f64> = (0..40)
        .map(|i| 7.0 + (i as f64 * 0.21).cos() * 3.0)
        .collect();
    let group_c: Vec<f64> = (0..40)
        .map(|i| 4.0 + (i as f64 * 0.5).sin() * 1.2)
        .collect();

    ax.boxplot(vec![group_a, group_b, group_c])?;

    ax.set_title("Boxplot of Three Groups");
    ax.set_xlabel("Group");
    ax.set_ylabel("Value");
    ax.grid(true);
    fig.save("examples/output/40_boxplot_multi.png")?;
    Ok(())
}
