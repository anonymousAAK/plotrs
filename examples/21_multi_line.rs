//! Three line series (sin, cos, sin*cos) with a legend and grid.
use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    let x: Vec<f64> = (0..200).map(|i| i as f64 * 0.05).collect();
    let sin: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    let cos: Vec<f64> = x.iter().map(|&v| v.cos()).collect();
    let product: Vec<f64> = x.iter().map(|&v| v.sin() * v.cos()).collect();

    ax.plot(x.clone(), sin)?
        .label("sin(x)")
        .color(Color::TAB_BLUE);
    ax.plot(x.clone(), cos)?
        .label("cos(x)")
        .color(Color::TAB_ORANGE);
    ax.plot(x, product)?
        .label("sin(x)*cos(x)")
        .color(Color::TAB_GREEN);

    ax.legend();
    ax.grid(true);
    ax.set_title("Multiple Line Series");
    fig.save("examples/output/21_multi_line.png")?;
    Ok(())
}
