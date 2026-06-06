//! Theme showcase — the same line plot rendered with each built-in theme.
//!
//! Produces a 2x3 subplot grid (plus the default theme for 7 total panels,
//! wrapped into a 3x3 grid with the last two slots left empty).

use plotkit::prelude::*;
use plotkit::FigureExt;

fn main() -> plotkit::Result<()> {
    // Generate sample data: sin and cos curves.
    let x: Vec<f64> = (0..80).map(|i| i as f64 * 0.1).collect();
    let y_sin: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    let y_cos: Vec<f64> = x.iter().map(|&v| v.cos()).collect();

    let themes: Vec<(&str, Theme)> = vec![
        ("Default", Theme::default()),
        ("Dark", Theme::dark()),
        ("Seaborn", Theme::seaborn()),
        ("ggplot2", Theme::ggplot()),
        ("Publication", Theme::publication()),
        ("Nature", Theme::nature()),
        ("Solarized", Theme::solarized()),
    ];

    // Create a 3-row x 3-column grid; only the first 7 cells are used.
    let mut fig = Figure::subplots_with_size(3, 3, 1800, 1600);
    fig.suptitle("Built-in Themes");

    for (i, (name, theme)) in themes.iter().enumerate() {
        if let Some(ax) = fig.axes_mut(i) {
            ax.set_theme(theme.clone());
            ax.plot(&x, &y_sin)?.label("sin(x)");
            ax.plot(&x, &y_cos)?.label("cos(x)");
            ax.set_title(name);
            ax.set_xlabel("x");
            ax.set_ylabel("y");
            ax.legend();
        }
    }

    fig.save("examples/output/20_themes.png")?;
    println!("Saved examples/output/20_themes.png");
    Ok(())
}
