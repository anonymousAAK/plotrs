//! ndarray integration for plotkit.
//!
//! This crate activates ndarray support in `plotkit-core`, providing
//! [`IntoSeries`](plotkit_core::series::IntoSeries) implementations for
//! common `ndarray` one-dimensional array types. Depend on this crate (or
//! enable the `ndarray` feature on the `plotkit` umbrella crate) to pass
//! ndarray arrays directly to any plotkit charting function.
//!
//! # Supported types
//!
//! | Type | Conversion |
//! |---|---|
//! | `&Array1<f64>` / `Array1<f64>` | zero-copy (contiguous slice borrow) |
//! | `ArrayView1<'_, f64>` | zero-copy when contiguous, O(n) copy when strided |
//! | `&Array1<f32>` / `Array1<f32>` | O(n) widening cast |
//! | `&Array1<i32>` / `Array1<i32>` | O(n) cast to `f64` |
//! | `&Array1<i64>` / `Array1<i64>` | O(n) cast to `f64` |
//! | `ArrayView1<'_, f32/i32/i64>` | O(n) cast to `f64` |
//!
//! # Examples
//!
//! ```
//! use ndarray::array;
//! use plotkit_core::series::IntoSeries;
//!
//! let arr = array![1.0, 2.0, 3.0];
//! let series = (&arr).into_series();
//! assert_eq!(series.data, vec![1.0, 2.0, 3.0]);
//! ```

#![deny(missing_docs)]

// Re-export ndarray so downstream users can access the version we link against.
pub use ndarray;

// Re-export the core series types for convenience.
pub use plotkit_core::series::{IntoSeries, Series};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use ndarray::{array, Array1};
    use plotkit_core::series::IntoSeries;

    // -- f64 ----------------------------------------------------------------

    #[test]
    fn array1_f64_ref() {
        let arr = array![1.0, 2.0, 3.0];
        let s = (&arr).into_series();
        assert_eq!(s.data, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn array1_f64_owned() {
        let arr = array![4.0, 5.0, 6.0];
        let s = arr.into_series();
        assert_eq!(s.data, vec![4.0, 5.0, 6.0]);
    }

    #[test]
    fn array_view1_f64_contiguous() {
        let arr = array![7.0, 8.0, 9.0];
        let view = arr.view();
        let s = view.into_series();
        assert_eq!(s.data, vec![7.0, 8.0, 9.0]);
    }

    #[test]
    fn array_view1_f64_strided() {
        let arr = array![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        // Take every other element — produces a non-contiguous view.
        let view = arr.slice(ndarray::s![..;2]);
        let s = view.into_series();
        assert_eq!(s.data, vec![1.0, 3.0, 5.0]);
    }

    #[test]
    fn empty_array_f64() {
        let arr: Array1<f64> = array![];
        let s = (&arr).into_series();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn single_element_f64() {
        let arr = array![42.0];
        let s = (&arr).into_series();
        assert_eq!(s.data, vec![42.0]);
    }

    // -- f32 ----------------------------------------------------------------

    #[test]
    fn array1_f32_ref() {
        let arr: Array1<f32> = array![1.5f32, 2.5, 3.5];
        let s = (&arr).into_series();
        assert_eq!(s.data, vec![1.5, 2.5, 3.5]);
    }

    #[test]
    fn array1_f32_owned() {
        let arr: Array1<f32> = array![0.1f32, 0.2];
        let s = arr.into_series();
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn array_view1_f32() {
        let arr: Array1<f32> = array![10.0f32, 20.0, 30.0];
        let view = arr.view();
        let s = view.into_series();
        assert_eq!(s.data, vec![10.0, 20.0, 30.0]);
    }

    // -- i32 ----------------------------------------------------------------

    #[test]
    fn array1_i32_ref() {
        let arr: Array1<i32> = array![1, 2, 3];
        let s = (&arr).into_series();
        assert_eq!(s.data, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn array1_i32_owned() {
        let arr: Array1<i32> = array![10, 20, 30];
        let s = arr.into_series();
        assert_eq!(s.data, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn array_view1_i32() {
        let arr: Array1<i32> = array![100, 200];
        let view = arr.view();
        let s = view.into_series();
        assert_eq!(s.data, vec![100.0, 200.0]);
    }

    // -- i64 ----------------------------------------------------------------

    #[test]
    fn array1_i64_ref() {
        let arr: Array1<i64> = array![1i64, 2, 3];
        let s = (&arr).into_series();
        assert_eq!(s.data, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn array1_i64_owned() {
        let arr: Array1<i64> = array![100i64, 200, 300];
        let s = arr.into_series();
        assert_eq!(s.data, vec![100.0, 200.0, 300.0]);
    }

    #[test]
    fn array_view1_i64() {
        let arr: Array1<i64> = array![1000i64, 2000];
        let view = arr.view();
        let s = view.into_series();
        assert_eq!(s.data, vec![1000.0, 2000.0]);
    }

    // -- Edge cases ---------------------------------------------------------

    #[test]
    fn nan_and_infinity() {
        let arr = array![f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1.0];
        let s = (&arr).into_series();
        assert_eq!(s.len(), 4);
        assert!(s.data[0].is_nan());
        assert!(s.data[1].is_infinite());
        assert!(s.data[2].is_infinite());
        assert_eq!(s.data[3], 1.0);
    }

    #[test]
    fn large_array() {
        let arr = Array1::from_vec((0..10_000).map(|i| i as f64).collect());
        let s = (&arr).into_series();
        assert_eq!(s.len(), 10_000);
        assert_eq!(s.data[0], 0.0);
        assert_eq!(s.data[9_999], 9_999.0);
    }

    #[test]
    fn reversed_slice_view() {
        let arr = array![1.0, 2.0, 3.0, 4.0, 5.0];
        let view = arr.slice(ndarray::s![..;-1]);
        let s = view.into_series();
        assert_eq!(s.data, vec![5.0, 4.0, 3.0, 2.0, 1.0]);
    }

    #[test]
    fn sub_slice_view() {
        let arr = array![10.0, 20.0, 30.0, 40.0, 50.0];
        let view = arr.slice(ndarray::s![1..4]);
        let s = view.into_series();
        assert_eq!(s.data, vec![20.0, 30.0, 40.0]);
    }
}
