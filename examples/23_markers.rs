//! Four scatter series, each using a different marker variant.
use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    let x: Vec<f64> = (0..20).map(|i| i as f64 * 0.5).collect();
    let y1: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    let y2: Vec<f64> = x.iter().map(|&v| v.sin() + 1.0).collect();
    let y3: Vec<f64> = x.iter().map(|&v| v.sin() + 2.0).collect();
    let y4: Vec<f64> = x.iter().map(|&v| v.sin() + 3.0).collect();

    ax.scatter(x.clone(), y1)?
        .label("circle")
        .marker(Marker::Circle)
        .color(Color::TAB_BLUE);
    ax.scatter(x.clone(), y2)?
        .label("square")
        .marker(Marker::Square)
        .color(Color::TAB_ORANGE);
    ax.scatter(x.clone(), y3)?
        .label("triangle")
        .marker(Marker::Triangle)
        .color(Color::TAB_GREEN);
    ax.scatter(x, y4)?
        .label("diamond")
        .marker(Marker::Diamond)
        .color(Color::TAB_RED);

    ax.legend();
    ax.set_title("Marker Styles");
    fig.save("examples/output/23_markers.png")?;
    Ok(())
}
