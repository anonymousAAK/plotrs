//! Stacked bar chart example.
//!
//! Demonstrates how to stack multiple bar series on top of each other using
//! the `bottom()` builder method. Each subsequent series uses the cumulative
//! heights of the series below as its base offset.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let categories = vec!["Q1", "Q2", "Q3", "Q4"];
    let product_a = vec![20.0, 35.0, 30.0, 25.0];
    let product_b = vec![25.0, 32.0, 34.0, 20.0];
    let product_c = vec![15.0, 18.0, 22.0, 28.0];

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    // First series sits on the baseline (bottom = 0, the default).
    ax.bar(categories.as_slice(), &product_a)?
        .label("Product A")
        .color(Color::TAB_BLUE);

    // Second series is stacked on top of the first.
    ax.bar(categories.as_slice(), &product_b)?
        .bottom(product_a.clone())
        .label("Product B")
        .color(Color::TAB_ORANGE);

    // Third series is stacked on top of the first two.
    let bottom_c: Vec<f64> = product_a
        .iter()
        .zip(product_b.iter())
        .map(|(a, b)| a + b)
        .collect();
    ax.bar(categories.as_slice(), &product_c)?
        .bottom(bottom_c)
        .label("Product C")
        .color(Color::TAB_GREEN);

    ax.set_title("Quarterly Revenue by Product (Stacked)");
    ax.set_ylabel("Revenue ($K)");
    ax.legend();
    fig.save("examples/output/18_stacked_bar.png")?;
    Ok(())
}
