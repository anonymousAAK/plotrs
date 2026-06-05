//! Step chart example showing Pre and Post alignment modes.

use plotkit::prelude::*;
use plotkit::artist::StepWhere;

fn main() -> plotkit::Result<()> {
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let y1: Vec<f64> = x.iter().map(|v| (v * 0.5_f64).sin() * 3.0 + 4.0).collect();
    let y2: Vec<f64> = x.iter().map(|v| (v * 0.5_f64).cos() * 2.0 + 2.0).collect();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.step(&x, &y1)?.label("pre (default)");
    ax.step(&x, &y2)?
        .where_step(StepWhere::Post)
        .label("post")
        .color(Color::TAB_RED);
    ax.set_title("Step Chart");
    ax.set_xlabel("x");
    ax.set_ylabel("y");
    ax.legend();
    fig.save("examples/output/08_step.png")?;
    Ok(())
}
