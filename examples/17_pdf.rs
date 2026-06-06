//! PDF output demo: renders a simple plot to a PDF file.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y_sin: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    let y_cos: Vec<f64> = x.iter().map(|&v| v.cos()).collect();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&x, &y_sin)?.label("sin(x)");
    ax.plot(&x, &y_cos)?.label("cos(x)");
    ax.set_title("Trigonometric Functions");
    ax.set_xlabel("x");
    ax.set_ylabel("y");
    ax.legend();
    ax.grid(true);

    fig.save("examples/output/17_pdf.pdf")?;
    Ok(())
}
