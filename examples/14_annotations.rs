//! Annotations example -- text labels and arrow annotations on a sine curve.

use plotkit::prelude::*;
use plotkit::FigureExt;

fn main() {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();

    let mut fig = Figure::new();
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&x, &y).unwrap();

    // Annotate the peak with an arrow.
    ax.annotate("peak", (std::f64::consts::FRAC_PI_2, 1.0), (3.0, 1.3))
        .arrowstyle(ArrowStyle::Simple);

    // Place a plain text label.
    ax.text(5.0, -0.5, "sin(x)").fontsize(14.0);

    ax.set_title("Annotations Demo");
    fig.save("examples/output/14_annotations.png").unwrap();
}
