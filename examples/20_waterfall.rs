//! Waterfall chart example.
//!
//! Demonstrates a financial waterfall chart showing how individual revenue
//! and cost items contribute to the final net profit figure.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let categories = vec![
        "Revenue",
        "Services",
        "COGS",
        "Operating\nExpenses",
        "Tax",
        "Net Profit",
    ];
    let values = vec![500.0, 120.0, -200.0, -150.0, -40.0, 230.0];

    let mut fig = Figure::with_size(900, 600);
    let ax = fig.add_subplot(1, 1, 1);

    ax.waterfall(categories.as_slice(), &values)?
        .total(&[0, 5]) // Revenue and Net Profit are totals
        .show_values(true)
        .label("P&L Waterfall");

    ax.set_title("Profit & Loss Waterfall");
    ax.set_ylabel("Amount ($K)");
    ax.legend();

    fig.save("examples/output/20_waterfall.png")?;
    Ok(())
}
