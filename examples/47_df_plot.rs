//! Polars `df.plot()` accessor — a grouped line chart straight from a DataFrame.

use plotkit::plotkit_polars::polars::prelude::*;
use plotkit::plotkit_polars::DataFramePlotExt;
use plotkit::prelude::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Two groups (A, B) over the same x with different curves.
    let x = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [0.0, 1.0, 4.0, 9.0, 16.0, 25.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let g = ["A", "A", "A", "A", "A", "A", "B", "B", "B", "B", "B", "B"];

    let df = DataFrame::new(vec![
        Series::new("x".into(), &x).into(),
        Series::new("y".into(), &y).into(),
        Series::new("group".into(), &g).into(),
    ])?;

    // One labelled series per distinct `group`, legend added automatically.
    let mut fig = df.plot().color_by("group").line("x", "y")?;
    fig.suptitle("df.plot() — grouped by category");
    fig.save("examples/output/47_df_plot.png")?;
    Ok(())
}
