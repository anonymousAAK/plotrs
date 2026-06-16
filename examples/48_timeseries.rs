//! Time-axis scale — x values are Unix timestamps; ticks render as dates.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    // One year of daily samples starting 2024-01-01 00:00 UTC (Unix seconds).
    let start = 1_704_067_200.0_f64;
    let day = 86_400.0;
    let x: Vec<f64> = (0..365).map(|i| start + i as f64 * day).collect();
    let y: Vec<f64> = (0..365)
        .map(|i| {
            let t = i as f64 / 365.0 * std::f64::consts::TAU;
            50.0 + 30.0 * t.sin() + (i as f64 * 0.3).sin() * 4.0
        })
        .collect();

    let mut fig = Figure::with_size(900, 500);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&x, &y)?.color(Color::TAB_BLUE).label("temperature");
    ax.set_xscale(Scale::Time);
    ax.set_title("Daily temperature over 2024");
    ax.set_xlabel("date");
    ax.set_ylabel("°C");
    ax.grid(true);
    ax.legend();
    fig.save("examples/output/48_timeseries.png")?;
    Ok(())
}
