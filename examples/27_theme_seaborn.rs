//! Multi-line plot styled with the seaborn theme and a legend.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    fig.set_theme(Theme::seaborn());

    let x: Vec<f64> = (0..200).map(|i| i as f64 * 0.05).collect();
    let y1: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    let y2: Vec<f64> = x.iter().map(|&v| v.cos()).collect();
    let y3: Vec<f64> = x.iter().map(|&v| (0.5 * v).sin()).collect();

    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(x.clone(), y1)?.width(2.0).label("sin(x)");
    ax.plot(x.clone(), y2)?.width(2.0).label("cos(x)");
    ax.plot(x, y3)?.width(2.0).label("sin(x/2)");
    ax.set_title("Seaborn Theme");
    ax.set_xlabel("x");
    ax.set_ylabel("y");
    ax.legend();
    ax.set_legend_loc(Loc::UpperRight);

    fig.save("examples/output/27_theme_seaborn.png")?;
    Ok(())
}
