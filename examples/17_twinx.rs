//! Twin axes (twinx) example -- dual y-axis plot.
//!
//! Demonstrates plotting two datasets with different y-scales on the same
//! subplot. The primary axes draws temperature on the left y-axis, and the
//! twin axes draws pressure on the right y-axis.

use plotkit::prelude::*;
use plotkit::FigureExt;

fn main() -> plotkit::Result<()> {
    let x: Vec<f64> = (0..100).map(|i| i as f64).collect();
    let temp: Vec<f64> = x.iter().map(|&t| 20.0 + 10.0 * (t * 0.05).sin()).collect();
    let pressure: Vec<f64> = x
        .iter()
        .map(|&t| 1013.0 + 50.0 * (t * 0.03).cos())
        .collect();

    let mut fig = Figure::new();

    // Primary axes: temperature on the left y-axis.
    let ax1 = fig.add_subplot(1, 1, 1);
    ax1.plot(&x, &temp)?.label("Temperature");
    ax1.set_ylabel("Temperature (\u{00B0}C)");
    ax1.set_xlabel("Time (s)");
    ax1.set_title("Dual Y-Axis Plot");
    ax1.legend();

    // Twin axes: pressure on the right y-axis.
    let ax2 = fig.twinx(0);
    ax2.plot(&x, &pressure)?.label("Pressure");
    ax2.set_ylabel("Pressure (hPa)");

    fig.save("examples/output/17_twinx.png")?;
    Ok(())
}
