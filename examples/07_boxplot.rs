//! Box plot example showing statistical distribution summaries.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    // Generate some sample data groups with different distributions.
    let group_a: Vec<f64> = (0..50).map(|i| 10.0 + (i as f64 * 0.3).sin() * 5.0 + i as f64 * 0.1).collect();
    let group_b: Vec<f64> = (0..50).map(|i| 20.0 + (i as f64 * 0.5).cos() * 8.0).collect();
    let group_c: Vec<f64> = (0..50).map(|i| 15.0 + (i as f64 * 0.2).sin() * 3.0 + if i == 45 { 30.0 } else { 0.0 }).collect();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.boxplot(vec![group_a, group_b, group_c])?
        .label("measurements")
        .width(0.6);
    ax.set_title("Box Plot Example");
    ax.set_xlabel("Group");
    ax.set_ylabel("Value");
    ax.legend();
    fig.save("examples/output/07_boxplot.png")?;
    Ok(())
}
