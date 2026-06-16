//! A 1x2 grid comparing sin and cos with a figure-wide suptitle.
use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);

    let x: Vec<f64> = (0..150).map(|i| i as f64 * 0.05).collect();

    // Left: sin.
    {
        let ax = fig.add_subplot(1, 2, 1);
        let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
        ax.plot(x.clone(), y)?.color(Color::TAB_BLUE);
        ax.set_title("sin(x)");
    }

    // Right: cos.
    {
        let ax = fig.add_subplot(1, 2, 2);
        let y: Vec<f64> = x.iter().map(|&v| v.cos()).collect();
        ax.plot(x.clone(), y)?.color(Color::TAB_ORANGE);
        ax.set_title("cos(x)");
    }

    fig.suptitle("Trigonometric Functions");
    fig.tight_layout();
    fig.save("examples/output/25_suptitle_layout.png")?;
    Ok(())
}
