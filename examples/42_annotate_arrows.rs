//! Line plot annotated with an arrow pointing at the peak and a text label.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|&v| (v - 5.0).sin() + 0.1 * v).collect();

    ax.plot(x.clone(), y.clone())?
        .color(Color::TAB_BLUE)
        .width(2.0)
        .label("signal");

    // Arrow annotation pointing from the label position to the peak.
    ax.annotate("peak", (6.6, 1.4), (8.5, 2.2))
        .arrowstyle(ArrowStyle::Simple)
        .arrow_color(Color::TAB_RED);

    // Free-floating text label.
    ax.text(1.0, 1.8, "rising trend");

    ax.set_title("Annotations and Arrows");
    ax.set_xlabel("x");
    ax.set_ylabel("y");
    ax.legend();
    ax.grid(true);
    fig.save("examples/output/42_annotate_arrows.png")?;
    Ok(())
}
