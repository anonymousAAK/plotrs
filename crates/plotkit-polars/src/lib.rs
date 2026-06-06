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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use plotkit_core::series::IntoCategories;
    use plotkit_core::series::IntoSeries as PlotIntoSeries;
    use polars::prelude::{
        DataType, Float64Chunked, IntoSeries as PolarsIntoSeries, NamedFrom, Series,
        StringChunked,
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
