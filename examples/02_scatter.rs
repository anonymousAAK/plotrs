//! Scatter plot example.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let x: Vec<f64> = (0..50).map(|i| i as f64 * 0.2).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.sin() + 0.1 * v).collect();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.scatter(&x, &y)?.label("data");
    ax.set_title("Scatter Plot");
    ax.set_xlabel("x");
    ax.set_ylabel("y");
    ax.legend();
    fig.save("examples/output/02_scatter.png")?;
    Ok(())
}
