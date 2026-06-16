//! Density histogram of 1000 pseudo-normal samples.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    // Build pseudo-normal samples by summing several decorrelated sine
    // waves and a fractional term (a cheap central-limit-style generator).
    let samples: Vec<f64> = (0..1000)
        .map(|i| {
            let t = i as f64;
            let a = (t * 0.7).sin();
            let b = (t * 1.3 + 1.0).sin();
            let c = (t * 2.1 + 2.0).sin();
            let d = (t * 0.37).fract() - 0.5;
            (a + b + c + d) * 1.5
        })
        .collect();

    ax.hist(samples, 30)?
        .color(Color::TAB_PURPLE)
        .density(true)
        .label("density");

    ax.set_title("Density Histogram");
    ax.set_xlabel("value");
    ax.set_ylabel("density");

    fig.save("examples/output/36_density_hist.png")?;
    Ok(())
}
