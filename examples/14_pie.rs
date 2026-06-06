//! Pie chart example showing language preferences.

use plotkit::prelude::*;
use plotkit::FigureExt;

fn main() -> plotkit::Result<()> {
    let mut fig = Figure::with_size(600, 500);
    let ax = fig.add_subplot(1, 1, 1);
    let sizes = vec![35.0, 25.0, 20.0, 15.0, 5.0];
    ax.pie(sizes)?
        .labels(vec!["Rust", "Python", "Go", "C++", "Other"])
        .autopct(true)
        .explode(vec![0.05, 0.0, 0.0, 0.0, 0.0]);
    ax.set_title("Language Preferences");
    fig.save("examples/output/14_pie.png")?;
    Ok(())
}
