//! A 2x2 grid showing a line, scatter, bar, and histogram subplot.
use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(800, 600);

    // Top-left: line plot.
    {
        let ax = fig.add_subplot(2, 2, 1);
        let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
        let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
        ax.plot(x, y)?.color(Color::TAB_BLUE);
        ax.set_title("Line");
    }

    // Top-right: scatter plot.
    {
        let ax = fig.add_subplot(2, 2, 2);
        let x: Vec<f64> = (0..40).map(|i| i as f64 * 0.25).collect();
        let y: Vec<f64> = x.iter().map(|&v| v.cos()).collect();
        ax.scatter(x, y)?
            .marker(Marker::Circle)
            .color(Color::TAB_ORANGE);
        ax.set_title("Scatter");
    }

    // Bottom-left: bar chart.
    {
        let ax = fig.add_subplot(2, 2, 3);
        let cats = vec!["A", "B", "C", "D"];
        let heights = vec![3.0, 7.0, 2.0, 5.0];
        ax.bar(cats, heights)?.color(Color::TAB_GREEN);
        ax.set_title("Bar");
    }

    // Bottom-right: histogram.
    {
        let ax = fig.add_subplot(2, 2, 4);
        let data: Vec<f64> = (0..200)
            .map(|i| {
                let t = i as f64 * 0.05;
                (t.sin() + (t * 1.7).cos()) * 2.0
            })
            .collect();
        ax.hist(data, 15)?.color(Color::TAB_RED);
        ax.set_title("Histogram");
    }

    fig.suptitle("Subplot Grid");
    fig.save("examples/output/24_subplots_grid.png")?;
    Ok(())
}
