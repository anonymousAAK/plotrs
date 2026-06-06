//! Grouped (side-by-side) bar chart example.
//!
//! Demonstrates how to create a grouped bar chart using the `bar_group()`
//! convenience method, which automatically computes bar widths and offsets
//! so that multiple series sit side by side within each category.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let categories = vec!["Math", "Science", "English", "History"];
    let class_a = vec![85.0, 90.0, 78.0, 92.0];
    let class_b = vec![80.0, 88.0, 82.0, 85.0];
    let class_c = vec![90.0, 75.0, 88.0, 79.0];

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    ax.bar_group(
        categories.as_slice(),
        &[
            ("Class A", class_a),
            ("Class B", class_b),
            ("Class C", class_c),
        ],
    )?;

    ax.set_title("Average Scores by Subject (Grouped)");
    ax.set_ylabel("Score");
    ax.legend();
    fig.save("examples/output/19_grouped_bar.png")?;
    Ok(())
}
