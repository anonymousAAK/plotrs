//! Violin plot example comparing different distributions.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    // --- Normal-like distribution ---
    let normal: Vec<f64> = (0..200)
        .map(|i| {
            let t = i as f64 * 0.1;
            50.0 + 10.0 * (t * 0.7).sin() + 5.0 * (t * 1.3).cos() + (t * 0.3).sin() * 3.0
        })
        .collect();

    // --- Right-skewed distribution ---
    let skewed: Vec<f64> = (0..200)
        .map(|i| {
            let t = i as f64 / 200.0;
            // Exponential-like via repeated sin/cos transforms.
            let base = t * t * 80.0;
            base + 5.0 * (i as f64 * 0.5).sin().abs()
        })
        .collect();

    // --- Bimodal distribution ---
    let bimodal: Vec<f64> = (0..200)
        .map(|i| {
            if i < 100 {
                30.0 + 5.0 * (i as f64 * 0.2).sin() + 2.0 * (i as f64 * 0.7).cos()
            } else {
                70.0 + 5.0 * (i as f64 * 0.3).sin() + 2.0 * (i as f64 * 0.5).cos()
            }
        })
        .collect();

    // --- Uniform-like distribution ---
    let uniform: Vec<f64> = (0..200)
        .map(|i| {
            let t = i as f64 / 199.0;
            t * 100.0 + 3.0 * (i as f64 * 0.4).sin()
        })
        .collect();

    let mut fig = Figure::with_size(900, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.violin(vec![normal, skewed, bimodal, uniform])?
        .label("distributions")
        .widths(0.8);
    ax.set_title("Violin Plot -- Distribution Comparison");
    ax.set_xlabel("Distribution");
    ax.set_ylabel("Value");
    ax.legend();
    fig.save("examples/output/17_violin.png")?;
    Ok(())
}
