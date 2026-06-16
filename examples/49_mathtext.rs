//! Super/subscript markup in titles, labels, and annotations (`^`, `_`, `{}`).

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|&v| v * v).collect();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&x, &y)?.color(Color::TAB_RED).label("quadratic");

    // `^` raises, `_` lowers; `{...}` groups multiple characters.
    ax.set_title("Quadratic growth: y = x^2");
    ax.set_xlabel("x");
    ax.set_ylabel("f(x) = x^2");
    ax.text(1.5, 80.0, "E = mc^2");
    ax.text(4.0, 50.0, "H_2O and CO_2");
    ax.text(6.5, 20.0, "x_{i+1} = a^{n-1}");
    ax.legend();
    fig.save("examples/output/49_mathtext.png")?;
    Ok(())
}
