//! A line plot rendered with the built-in dark theme.
use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);
    fig.set_theme(Theme::dark());

    let ax = fig.add_subplot(1, 1, 1);
    let x: Vec<f64> = (0..200).map(|i| i as f64 * 0.05).collect();
    let y: Vec<f64> = x.iter().map(|&v| (v).sin() * (-0.1 * v).exp()).collect();

    ax.plot(x, y)?.label("damped sine").color(Color::TAB_BLUE);
    ax.legend();
    ax.grid(true);
    ax.set_title("Dark Theme");
    fig.save("examples/output/26_theme_dark.png")?;
    Ok(())
}
