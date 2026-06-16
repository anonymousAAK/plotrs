//! Polars integration for plotkit.
//!
//! This crate activates Polars support in `plotkit-core`, providing
//! [`IntoSeries`](plotkit_core::series::IntoSeries) and
//! [`IntoCategories`](plotkit_core::series::IntoCategories) implementations
//! for Polars [`Series`](polars::prelude::Series). Depend on this crate (or
//! enable the `polars` feature on the `plotkit` umbrella crate) to pass
//! Polars data directly to any plotkit charting function.
//!
//! # Supported types
//!
//! | Type | Conversion |
//! |---|---|
//! | `&Series` (numeric: f64, f32, i32, i64, u32, u64) | O(n) cast to `f64`, nulls become `NAN` |
//! | `Series` (numeric, owned) | delegates to borrowed impl |
//! | `&Series` (string / Utf8) | O(n) extraction to `Vec<String>`, nulls become `"null"` |
//!
//! # Examples
//!
//! ```
//! use polars::prelude::*;
//! use plotkit_core::series::IntoSeries;
//!
//! let s = Series::new("values".into(), &[1.0_f64, 2.0, 3.0]);
//! let plotkit_series = (&s).into_series();
//! assert_eq!(plotkit_series.data, vec![1.0, 2.0, 3.0]);
//! ```

#![deny(missing_docs)]

// Re-export polars so downstream users can access the version we link against.
pub use polars;

// Re-export the core series types for convenience.
pub use plotkit_core::series::{IntoCategories, IntoSeries, Series};

use plotkit_core::error::PlotError;
use plotkit_core::figure::Figure;
use polars::prelude::DataFrame;

/// Adds a pandas/seaborn-style `.plot()` accessor to a Polars [`DataFrame`].
///
/// The terminal methods return a [`Figure`]; save it via the umbrella crate's
/// `FigureExt` (`use plotkit::prelude::*;` then `fig.save("out.png")?`).
///
/// ```
/// use plotkit_polars::DataFramePlotExt;
/// use plotkit_polars::polars::prelude::*;
///
/// let df = DataFrame::new(vec![
///     Series::new("x".into(), &[1.0, 2.0, 3.0]).into(),
///     Series::new("y".into(), &[1.0, 4.0, 9.0]).into(),
/// ])
/// .unwrap();
///
/// let fig = df.plot().line("x", "y").unwrap();
/// assert_eq!(fig.num_axes(), 1);
/// ```
pub trait DataFramePlotExt {
    /// Begins a plot built directly from this DataFrame's columns.
    fn plot(&self) -> DfPlot<'_>;
}

impl DataFramePlotExt for DataFrame {
    fn plot(&self) -> DfPlot<'_> {
        DfPlot {
            df: self,
            width: 800,
            height: 600,
            color_by: None,
        }
    }
}

/// A builder produced by [`DataFramePlotExt::plot`]. Terminal methods
/// (`line`, `scatter`, `hist`, `bar`) consume the builder and return a fully
/// built [`Figure`], ready to `.save(...)` via the umbrella crate's `FigureExt`.
pub struct DfPlot<'a> {
    df: &'a DataFrame,
    width: u32,
    height: u32,
    color_by: Option<String>,
}

impl DfPlot<'_> {
    /// Sets the output figure size in pixels (default 800x600).
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Splits the data into one labelled series per distinct value of `column`
    /// (categorical grouping, like seaborn `hue`). Applies to `line`/`scatter`.
    pub fn color_by(mut self, column: &str) -> Self {
        self.color_by = Some(column.to_string());
        self
    }

    /// Plots `y` against `x` as a line chart.
    pub fn line(self, x: &str, y: &str) -> Result<Figure, PlotError> {
        self.xy(x, y, XyKind::Line)
    }

    /// Plots `y` against `x` as a scatter chart.
    pub fn scatter(self, x: &str, y: &str) -> Result<Figure, PlotError> {
        self.xy(x, y, XyKind::Scatter)
    }

    /// Draws a histogram of a single numeric `column`.
    pub fn hist(self, column: &str, bins: usize) -> Result<Figure, PlotError> {
        let data = col_f64(self.df, column)?;
        let mut fig = Figure::with_size(self.width, self.height);
        let ax = fig.add_subplot(1, 1, 1);
        ax.hist(data, bins)?.label(column);
        ax.set_xlabel(column);
        ax.set_ylabel("count");
        Ok(fig)
    }

    /// Draws a bar chart with string categories from `category` and heights
    /// from the numeric column `value`.
    pub fn bar(self, category: &str, value: &str) -> Result<Figure, PlotError> {
        let cats = col_str(self.df, category)?;
        let vals = col_f64(self.df, value)?;
        let mut fig = Figure::with_size(self.width, self.height);
        let ax = fig.add_subplot(1, 1, 1);
        ax.bar(cats, vals)?;
        ax.set_xlabel(category);
        ax.set_ylabel(value);
        Ok(fig)
    }

    fn xy(self, x: &str, y: &str, kind: XyKind) -> Result<Figure, PlotError> {
        let xs = col_f64(self.df, x)?;
        let ys = col_f64(self.df, y)?;
        let mut fig = Figure::with_size(self.width, self.height);
        let ax = fig.add_subplot(1, 1, 1);

        if let Some(group_col) = &self.color_by {
            let keys = col_str(self.df, group_col)?;
            // Group (x, y) rows by category value, preserving first-seen order.
            let mut groups: Vec<(String, Vec<f64>, Vec<f64>)> = Vec::new();
            for i in 0..xs.len().min(ys.len()).min(keys.len()) {
                match groups.iter_mut().find(|g| g.0 == keys[i]) {
                    Some(g) => {
                        g.1.push(xs[i]);
                        g.2.push(ys[i]);
                    }
                    None => groups.push((keys[i].clone(), vec![xs[i]], vec![ys[i]])),
                }
            }
            for (name, gx, gy) in groups {
                match kind {
                    XyKind::Line => {
                        ax.plot(gx, gy)?.label(&name);
                    }
                    XyKind::Scatter => {
                        ax.scatter(gx, gy)?.label(&name);
                    }
                }
            }
            ax.legend();
        } else {
            match kind {
                XyKind::Line => {
                    ax.plot(xs, ys)?.label(y);
                }
                XyKind::Scatter => {
                    ax.scatter(xs, ys)?.label(y);
                }
            }
        }
        ax.set_xlabel(x);
        ax.set_ylabel(y);
        Ok(fig)
    }
}

#[derive(Clone, Copy)]
enum XyKind {
    Line,
    Scatter,
}

/// Extracts a numeric column as `Vec<f64>` (nulls → NaN) via the `IntoSeries`
/// bridge. Returns [`PlotError::UnknownColumn`] if the column is absent.
fn col_f64(df: &DataFrame, name: &str) -> Result<Vec<f64>, PlotError> {
    let column = df
        .column(name)
        .map_err(|_| PlotError::UnknownColumn(name.to_string()))?;
    let series = column.as_materialized_series();
    Ok(IntoSeries::into_series(series).data)
}

/// Extracts a string/categorical column as `Vec<String>` (nulls → "null").
fn col_str(df: &DataFrame, name: &str) -> Result<Vec<String>, PlotError> {
    let column = df
        .column(name)
        .map_err(|_| PlotError::UnknownColumn(name.to_string()))?;
    let series = column.as_materialized_series();
    let chunked = series
        .str()
        .map_err(|e| PlotError::UnknownColumn(format!("{name}: not a string column ({e})")))?;
    Ok(chunked
        .into_iter()
        .map(|opt| opt.unwrap_or("null").to_string())
        .collect())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use plotkit_core::series::IntoCategories;
    use plotkit_core::series::IntoSeries as PlotIntoSeries;
    use polars::prelude::{
        DataType, Float64Chunked, IntoSeries as PolarsIntoSeries, NamedFrom, Series, StringChunked,
    };

    // -- IntoSeries: f64 ---------------------------------------------------

    #[test]
    fn series_from_polars_f64() {
        let s = Series::new("a".into(), &[1.0_f64, 2.5, 3.0]);
        let ps = PlotIntoSeries::into_series(&s);
        assert_eq!(ps.data, vec![1.0, 2.5, 3.0]);
    }

    #[test]
    fn series_from_polars_f64_owned() {
        let s = Series::new("a".into(), &[4.0_f64, 5.0]);
        let ps = PlotIntoSeries::into_series(s);
        assert_eq!(ps.data, vec![4.0, 5.0]);
    }

    // -- IntoSeries: f32 ---------------------------------------------------

    #[test]
    fn series_from_polars_f32() {
        let s = Series::new("b".into(), &[1.5_f32, 2.5]);
        let ps = PlotIntoSeries::into_series(&s);
        assert_eq!(ps.data, vec![1.5_f64, 2.5]);
    }

    // -- IntoSeries: i32 ---------------------------------------------------

    #[test]
    fn series_from_polars_i32() {
        let s = Series::new("c".into(), &[10_i32, 20, 30]);
        let ps = PlotIntoSeries::into_series(&s);
        assert_eq!(ps.data, vec![10.0, 20.0, 30.0]);
    }

    // -- IntoSeries: i64 ---------------------------------------------------

    #[test]
    fn series_from_polars_i64() {
        let s = Series::new("d".into(), &[100_i64, 200]);
        let ps = PlotIntoSeries::into_series(&s);
        assert_eq!(ps.data, vec![100.0, 200.0]);
    }

    // -- IntoSeries: u32 ---------------------------------------------------

    #[test]
    fn series_from_polars_u32() {
        let s = Series::new("e".into(), &[1_u32, 2, 3]);
        let ps = PlotIntoSeries::into_series(&s);
        assert_eq!(ps.data, vec![1.0, 2.0, 3.0]);
    }

    // -- IntoSeries: u64 ---------------------------------------------------

    #[test]
    fn series_from_polars_u64() {
        let s = Series::new("f".into(), &[42_u64, 99]);
        let ps = PlotIntoSeries::into_series(&s);
        assert_eq!(ps.data, vec![42.0, 99.0]);
    }

    // -- IntoSeries: null handling -----------------------------------------

    #[test]
    fn series_null_values_become_nan() {
        let ca = Float64Chunked::new("g".into(), &[Some(1.0), None, Some(3.0)]);
        let s: Series = PolarsIntoSeries::into_series(ca);
        let ps = PlotIntoSeries::into_series(&s);
        assert_eq!(ps.data.len(), 3);
        assert_eq!(ps.data[0], 1.0);
        assert!(ps.data[1].is_nan());
        assert_eq!(ps.data[2], 3.0);
    }

    // -- IntoSeries: empty -------------------------------------------------

    #[test]
    fn series_empty_polars() {
        let s = Series::new_empty("empty".into(), &DataType::Float64);
        let ps = PlotIntoSeries::into_series(&s);
        assert!(ps.is_empty());
    }

    // -- IntoCategories: string series -------------------------------------

    #[test]
    fn categories_from_polars_str() {
        let s = Series::new("labels".into(), &["foo", "bar", "baz"]);
        let cats = IntoCategories::into_categories(&s);
        assert_eq!(cats.labels, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn categories_null_becomes_null_string() {
        let ca = StringChunked::new("h".into(), &[Some("a"), None, Some("c")]);
        let s: Series = PolarsIntoSeries::into_series(ca);
        let cats = IntoCategories::into_categories(&s);
        assert_eq!(cats.labels, vec!["a", "null", "c"]);
    }

    #[test]
    fn categories_empty_polars() {
        let s = Series::new_empty("empty".into(), &DataType::String);
        let cats = IntoCategories::into_categories(&s);
        assert!(cats.is_empty());
    }

    // -- Panic on wrong dtype ----------------------------------------------

    #[test]
    fn series_string_cast_produces_nan() {
        // Polars casts non-numeric strings to null (NAN in plotkit).
        let s = Series::new("bad".into(), &["not", "numeric"]);
        let ps = PlotIntoSeries::into_series(&s);
        assert_eq!(ps.data.len(), 2);
        assert!(ps.data[0].is_nan());
        assert!(ps.data[1].is_nan());
    }

    #[test]
    #[should_panic(expected = "is not a string type")]
    fn categories_panics_on_numeric_dtype() {
        let s = Series::new("bad".into(), &[1.0_f64, 2.0]);
        let _ = IntoCategories::into_categories(&s);
    }
}
