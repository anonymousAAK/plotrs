//! Violin plot showing the distribution of three datasets.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    // Three datasets with distinct shapes.
    let dist_a: Vec<f64> = (0..60)
        .map(|i| 3.0 + (i as f64 * 0.3).sin() * 1.5)
        .collect();
    let dist_b: Vec<f64> = (0..60)
        .map(|i| 5.0 + (i as f64 * 0.17).cos() * 2.5)
        .collect();
    let dist_c: Vec<f64> = (0..60)
        .map(|i| 4.0 + (i as f64 * 0.45).sin() * (i as f64 * 0.05).cos() * 2.0)
        .collect();

    ax.violin(vec![dist_a, dist_b, dist_c])?;

    ax.set_title("Violin Plot of Three Distributions");
    ax.set_xlabel("Dataset");
    ax.set_ylabel("Value");
    ax.grid(true);
    fig.save("examples/output/41_violin_multi.png")?;
    Ok(())
}
