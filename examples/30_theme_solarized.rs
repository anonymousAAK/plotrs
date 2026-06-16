//! Scatter plot styled with the solarized theme.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    fig.set_theme(Theme::solarized());

    let x: Vec<f64> = (0..80).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|&v| v + (v * 1.7).sin() * 0.8).collect();

    let ax = fig.add_subplot(1, 1, 1);
    ax.scatter(x, y)?
        .marker(Marker::Circle)
        .size(7.0)
        .label("samples");
    ax.set_title("Solarized Theme");
    ax.set_xlabel("x");
    ax.set_ylabel("y");
    ax.legend();

    fig.save("examples/output/30_theme_solarized.png")?;
    Ok(())
}
