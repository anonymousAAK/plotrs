//! Filled polar (radar) chart comparing two profiles.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    let categories = ["Speed", "Power", "Range", "Safety", "Comfort", "Price"];
    let n = categories.len();

    // Angles evenly spaced around the circle, closing back to the first point.
    let mut theta: Vec<f64> = (0..n)
        .map(|i| i as f64 * std::f64::consts::TAU / n as f64)
        .collect();
    theta.push(theta[0]);

    // Profile A.
    let mut r_a = vec![8.0, 7.0, 6.0, 9.0, 5.0, 7.0];
    r_a.push(r_a[0]);
    ax.polar_fill(&theta, &r_a)?
        .color(Color::TAB_BLUE)
        .alpha(0.25)
        .linewidth(2.0)
        .label("Model A");

    // Profile B.
    let mut r_b = vec![6.0, 9.0, 8.0, 5.0, 8.0, 6.0];
    r_b.push(r_b[0]);
    ax.polar_fill(&theta, &r_b)?
        .color(Color::TAB_ORANGE)
        .alpha(0.25)
        .linewidth(2.0)
        .label("Model B");

    ax.set_title("Filled Radar Chart");
    ax.legend();
    ax.set_legend_loc(Loc::UpperRight);
    fig.save("examples/output/46_polar_fill.png")?;
    Ok(())
}
