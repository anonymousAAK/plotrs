//! Using ndarray arrays as chart data.
//!
//! Run with:
//! ```sh
//! cargo run --example 13_ndarray --features ndarray
//! ```

use plotkit::plotkit_ndarray::ndarray::Array1;

fn main() -> plotkit::Result<()> {
    let x: Array1<f64> = Array1::linspace(0.0, 10.0, 100);
    let y: Array1<f64> = x.mapv(f64::sin);

    plotkit::plot(&x, &y)?;
    plotkit::title("sin(x) from ndarray");
    plotkit::xlabel("x");
    plotkit::ylabel("sin(x)");
    plotkit::savefig("examples/output/13_ndarray.png")?;
    Ok(())
}
