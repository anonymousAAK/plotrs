//! Bar chart example.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let categories = vec!["A", "B", "C", "D", "E"];
    let values = vec![23.0, 45.0, 12.0, 67.0, 34.0];

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.bar(categories.as_slice(), &values)?;
    ax.set_title("Bar Chart");
    ax.set_ylabel("Value");
    fig.save("examples/output/03_bar.png")?;
    Ok(())
}
