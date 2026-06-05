//! Fill-between example showing a shaded area.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y1: Vec<f64> = x.iter().map(|&v| v.sin() - 0.5).collect();
    let y2: Vec<f64> = x.iter().map(|&v| v.sin() + 0.5).collect();
    let y_mid: Vec<f64> = x.iter().map(|&v| v.sin()).collect();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.fill_between(&x, &y1, &y2)?.alpha(0.3).label("range");
    ax.plot(&x, &y_mid)?.label("sin(x)");
    ax.set_title("Fill Between");
    ax.set_xlabel("x");
    ax.legend();
    fig.save("examples/output/05_fill_between.png")?;
    Ok(())
}
