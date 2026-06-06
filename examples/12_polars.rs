//! Polars integration example — plot directly from a DataFrame.

use plotkit::plotkit_polars::polars::prelude::{NamedFrom, Series};

fn main() -> plotkit::Result<()> {
    // Build Polars Series for x and y data.
    let x = Series::new("x".into(), &[0.0_f64, 1.0, 2.0, 3.0, 4.0, 5.0]);
    let y = Series::new("y".into(), &[0.0_f64, 1.0, 4.0, 9.0, 16.0, 25.0]);

    // Pass Polars Series directly — IntoSeries conversion is automatic.
    plotkit::plot(&x, &y)?;
    plotkit::title("y = x^2 (from Polars)");
    plotkit::xlabel("x");
    plotkit::ylabel("y");
    plotkit::savefig("examples/output/12_polars.png")?;
    Ok(())
}
