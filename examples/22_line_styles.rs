//! Three lines sharing the same x with distinct styles and colors.
use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y1: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    let y2: Vec<f64> = x.iter().map(|&v| (v + 1.0).sin()).collect();
    let y3: Vec<f64> = x.iter().map(|&v| (v + 2.0).sin()).collect();

    ax.plot(x.clone(), y1)?
        .label("solid")
        .style(LineStyle::Solid)
        .color(Color::TAB_BLUE);
    ax.plot(x.clone(), y2)?
        .label("dashed")
        .style(LineStyle::Dashed)
        .color(Color::TAB_RED);
    ax.plot(x, y3)?
        .label("dotted")
        .style(LineStyle::Dotted)
        .color(Color::TAB_GREEN);

    ax.legend();
    ax.set_title("Line Styles");
    fig.save("examples/output/22_line_styles.png")?;
    Ok(())
}
