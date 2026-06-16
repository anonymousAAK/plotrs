//! Bar chart styled with the ggplot theme.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    fig.set_theme(Theme::ggplot());

    let categories = vec!["Alpha", "Beta", "Gamma", "Delta", "Epsilon"];
    let heights: Vec<f64> = vec![23.0, 45.0, 12.0, 38.0, 29.0];

    let ax = fig.add_subplot(1, 1, 1);
    ax.bar(categories, heights)?.label("counts");
    ax.set_title("ggplot Theme");
    ax.set_xlabel("Category");
    ax.set_ylabel("Value");

    fig.save("examples/output/28_theme_ggplot.png")?;
    Ok(())
}
