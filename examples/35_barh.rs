//! Horizontal bar chart across five string categories.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    let categories = vec!["Rust", "Go", "Python", "C++", "TypeScript"];
    let widths: Vec<f64> = vec![92.0, 74.0, 68.0, 81.0, 59.0];

    ax.barh(categories, widths)?
        .color(Color::TAB_GREEN)
        .label("Popularity");

    ax.set_title("Horizontal Bar Chart");
    ax.set_xlabel("Score");
    ax.set_ylabel("Language");

    fig.save("examples/output/35_barh.png")?;
    Ok(())
}
