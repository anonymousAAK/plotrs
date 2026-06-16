//! Line plot styled with the publication theme.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    fig.set_theme(Theme::publication());

    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x
        .iter()
        .map(|&v| (-0.2 * v).exp() * (2.0 * v).sin())
        .collect();

    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(x, y)?.width(1.5).label("damped oscillation");
    ax.set_title("Publication Theme");
    ax.set_xlabel("time (s)");
    ax.set_ylabel("amplitude");
    ax.legend();

    fig.save("examples/output/29_theme_publication.png")?;
    Ok(())
}
