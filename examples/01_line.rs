//! Line plot example — the flagship demo.

fn main() -> plotkit::Result<()> {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();

    plotkit::plot(&x, &y)?;
    plotkit::title("sin(x)");
    plotkit::xlabel("x");
    plotkit::ylabel("y");
    plotkit::savefig("examples/output/01_line.png")?;
    Ok(())
}
