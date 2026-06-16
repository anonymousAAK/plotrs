//! Twin x-axis: two series sharing the y-axis with independent top/bottom x-axes.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);

    // Primary axes: value vs. bottom x-axis.
    {
        let ax = fig.add_subplot(1, 1, 1);
        let x1: Vec<f64> = (0..50).map(|i| i as f64 * 0.2).collect();
        let y1: Vec<f64> = x1.iter().map(|&v| v.sin() + 2.0).collect();
        ax.plot(x1, y1)?
            .color(Color::TAB_BLUE)
            .width(2.0)
            .label("bottom-axis series");
        ax.set_title("Twin X-Axis Demo");
        ax.set_xlabel("bottom x");
        ax.set_ylabel("shared y");
    }

    // Twin axes sharing the y-axis, with its own x-axis on top.
    {
        let ax2 = fig.twiny(0);
        let x2: Vec<f64> = (0..50).map(|i| i as f64 * 5.0).collect();
        let y2: Vec<f64> = x2.iter().map(|&v| (v * 0.02).cos() + 2.0).collect();
        ax2.plot(x2, y2)?
            .color(Color::TAB_RED)
            .width(2.0)
            .label("top-axis series");
        ax2.set_xlabel("top x");
    }

    fig.save("examples/output/45_twiny.png")?;
    Ok(())
}
