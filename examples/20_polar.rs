//! Polar plot examples: line plot and filled radar chart.

use plotkit::prelude::*;
use plotkit::FigureExt;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::subplots_with_size(1, 2, 1200, 550);

    // -----------------------------------------------------------------------
    // Subplot 1: Polar line plot (cardioid)
    // -----------------------------------------------------------------------
    {
        let ax = fig.axes_mut(0).unwrap();
        let n = 360;
        let theta: Vec<f64> = (0..n)
            .map(|i| i as f64 * std::f64::consts::TAU / n as f64)
            .collect();

        // Cardioid: r = 1 + cos(theta)
        let r: Vec<f64> = theta.iter().map(|&t| 1.0 + t.cos()).collect();

        ax.polar_plot(&theta, &r)?
            .color(Color::TAB_BLUE)
            .linewidth(2.0)
            .label("Cardioid");

        // Rose curve: r = cos(3*theta)
        let r2: Vec<f64> = theta.iter().map(|&t| (3.0 * t).cos().abs()).collect();
        ax.polar_plot(&theta, &r2)?
            .color(Color::TAB_RED)
            .linewidth(1.5)
            .label("Rose");

        ax.set_title("Polar Line Plots");
        ax.legend();
        ax.set_legend_loc(Loc::UpperRight);
    }

    // -----------------------------------------------------------------------
    // Subplot 2: Filled radar chart (performance metrics)
    // -----------------------------------------------------------------------
    {
        let ax = fig.axes_mut(1).unwrap();
        let categories = ["Speed", "Power", "Range", "Safety", "Comfort", "Price"];
        let n = categories.len();

        // Angles evenly spaced around the circle, with the path closing back
        // to the first point.
        let mut theta: Vec<f64> = (0..n)
            .map(|i| i as f64 * std::f64::consts::TAU / n as f64)
            .collect();
        theta.push(theta[0]); // close the polygon

        // Dataset A
        let mut r_a = vec![8.0, 7.0, 6.0, 9.0, 5.0, 7.0];
        r_a.push(r_a[0]);
        ax.polar_fill(&theta, &r_a)?
            .color(Color::TAB_BLUE)
            .alpha(0.25)
            .linewidth(2.0)
            .label("Model A");

        // Dataset B
        let mut r_b = vec![6.0, 9.0, 8.0, 5.0, 8.0, 6.0];
        r_b.push(r_b[0]);
        ax.polar_fill(&theta, &r_b)?
            .color(Color::TAB_ORANGE)
            .alpha(0.25)
            .linewidth(2.0)
            .label("Model B");

        ax.set_title("Radar Chart Comparison");
        ax.legend();
        ax.set_legend_loc(Loc::UpperRight);
    }

    fig.save("examples/output/20_polar.png")?;
    Ok(())
}
