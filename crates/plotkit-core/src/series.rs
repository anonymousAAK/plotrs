//! Data series abstraction for chart input.

/// A sequence of `f64` values representing one dimension of chart data.
///
/// `Series` is the canonical data container used throughout the charting
/// pipeline. Users rarely construct one directly; instead they pass any
/// type that implements [`IntoSeries`] (slices, vectors, arrays, ranges,
/// etc.) and the conversion happens automatically.
#[derive(Debug, Clone)]
pub struct Series {
    /// The underlying data values.
    pub data: Vec<f64>,
}

impl Series {
    /// Creates a new series from a vector of values.
    ///
    /// # Examples
    ///
    /// ```
    /// use plotkit_core::series::Series;
    ///
    /// let s = Series::new(vec![1.0, 2.0, 3.0]);
    /// assert_eq!(s.len(), 3);
    /// ```
    pub fn new(data: Vec<f64>) -> Self {
        Self { data }
    }

    /// Returns the number of data points.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the series contains no data points.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the minimum finite value, or `None` if the series is empty
    /// or contains no finite values.
    ///
    /// Non-finite values (`NaN`, `+Inf`, `-Inf`) are ignored.
    pub fn min(&self) -> Option<f64> {
        self.data
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .reduce(f64::min)
    }

    /// Returns the maximum finite value, or `None` if the series is empty
    /// or contains no finite values.
    ///
    /// Non-finite values (`NaN`, `+Inf`, `-Inf`) are ignored.
    pub fn max(&self) -> Option<f64> {
        self.data
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .reduce(f64::max)
    }

    /// Returns `(min, max)` of the finite values, or `None` if the series
    /// is empty or contains no finite values.
    pub fn bounds(&self) -> Option<(f64, f64)> {
        Some((self.min()?, self.max()?))
    }
}

// ---------------------------------------------------------------------------
// IntoSeries
// ---------------------------------------------------------------------------

/// Trait for types that can be converted into a [`Series`].
///
/// This is the primary entry-point for user data. Any function that accepts
/// chart data should be generic over `impl IntoSeries` so that callers can
/// pass slices, vectors, arrays, integer collections, or ranges without
/// manual conversion.
pub trait IntoSeries {
    /// Converts this value into a [`Series`].
    fn into_series(self) -> Series;
}

// -- f64 containers --------------------------------------------------------

impl IntoSeries for Vec<f64> {
    /// Zero-copy conversion from an owned `Vec<f64>`.
    fn into_series(self) -> Series {
        Series::new(self)
    }
}

impl IntoSeries for &[f64] {
    /// Clones the slice data into a new [`Series`].
    fn into_series(self) -> Series {
        Series::new(self.to_vec())
    }
}

impl IntoSeries for &Vec<f64> {
    /// Clones the vector data into a new [`Series`].
    fn into_series(self) -> Series {
        Series::new(self.clone())
    }
}

impl<const N: usize> IntoSeries for [f64; N] {
    /// Converts a fixed-size array of `f64` into a [`Series`].
    fn into_series(self) -> Series {
        Series::new(self.to_vec())
    }
}

impl<const N: usize> IntoSeries for &[f64; N] {
    /// Clones a fixed-size array reference into a [`Series`].
    fn into_series(self) -> Series {
        Series::new(self.to_vec())
    }
}

// -- Identity --------------------------------------------------------------

impl IntoSeries for Series {
    /// Identity conversion — returns `self` unchanged.
    fn into_series(self) -> Series {
        self
    }
}

// -- Range -----------------------------------------------------------------

impl IntoSeries for std::ops::Range<i32> {
    /// Converts an integer range into a [`Series`] of `f64` values.
    ///
    /// # Examples
    ///
    /// ```
    /// use plotkit_core::series::IntoSeries;
    ///
    /// let s = (0..5).into_series();
    /// assert_eq!(s.data, vec![0.0, 1.0, 2.0, 3.0, 4.0]);
    /// ```
    fn into_series(self) -> Series {
        Series::new(self.map(|v| v as f64).collect())
    }
}

// -- i32 containers --------------------------------------------------------

impl IntoSeries for Vec<i32> {
    /// Converts a `Vec<i32>` into a [`Series`] by casting each element to `f64`.
    fn into_series(self) -> Series {
        Series::new(self.into_iter().map(|v| v as f64).collect())
    }
}

impl IntoSeries for &[i32] {
    /// Converts an `i32` slice into a [`Series`] by casting each element to `f64`.
    fn into_series(self) -> Series {
        Series::new(self.iter().map(|&v| v as f64).collect())
    }
}

// -- ndarray (feature-gated) -----------------------------------------------

#[cfg(feature = "ndarray")]
mod ndarray_impls {
    use super::{IntoSeries, Series};
    use ndarray::{Array1, ArrayView1};

    // -- f64 (zero-copy where possible) ------------------------------------

    impl IntoSeries for &Array1<f64> {
        /// Converts a reference to an `Array1<f64>` into a [`Series`].
        ///
        /// Uses `as_slice()` for zero-copy access on contiguous arrays and
        /// falls back to an iterator copy for non-standard memory layouts.
        fn into_series(self) -> Series {
            match self.as_slice() {
                Some(slice) => Series::new(slice.to_vec()),
                None => Series::new(self.iter().copied().collect()),
            }
        }
    }

    impl IntoSeries for Array1<f64> {
        /// Converts an owned `Array1<f64>` into a [`Series`].
        ///
        /// Attempts to unwrap the underlying `Vec` without copying. Falls
        /// back to a copy when the array is not in standard layout.
        fn into_series(self) -> Series {
            if self.is_standard_layout() {
                let len = self.len();
                let (vec, offset) = self.into_raw_vec_and_offset();
                let offset = offset.unwrap_or(0);
                if offset == 0 && vec.len() == len {
                    Series::new(vec)
                } else {
                    Series::new(vec[offset..offset + len].to_vec())
                }
            } else {
                Series::new(self.iter().copied().collect())
            }
        }
    }

    impl IntoSeries for ArrayView1<'_, f64> {
        /// Converts an `ArrayView1<f64>` into a [`Series`].
        ///
        /// Uses `as_slice()` for contiguous views and falls back to
        /// element-wise iteration for strided views.
        fn into_series(self) -> Series {
            match self.as_slice() {
                Some(slice) => Series::new(slice.to_vec()),
                None => Series::new(self.iter().copied().collect()),
            }
        }
    }

    // -- Cast types (f32, i32, i64) ----------------------------------------

    macro_rules! impl_into_series_ndarray_cast {
        ($t:ty) => {
            impl IntoSeries for &Array1<$t> {
                fn into_series(self) -> Series {
                    Series::new(self.iter().map(|&v| v as f64).collect())
                }
            }

            impl IntoSeries for Array1<$t> {
                fn into_series(self) -> Series {
                    Series::new(self.iter().map(|&v| v as f64).collect())
                }
            }

            impl IntoSeries for ArrayView1<'_, $t> {
                fn into_series(self) -> Series {
                    Series::new(self.iter().map(|&v| v as f64).collect())
                }
            }
        };
    }

    impl_into_series_ndarray_cast!(f32);
    impl_into_series_ndarray_cast!(i32);
    impl_into_series_ndarray_cast!(i64);
}

// -- polars (feature-gated) ------------------------------------------------

#[cfg(feature = "polars")]
mod polars_impls {
    use super::{Categories, IntoCategories, IntoSeries, Series};
    use polars::prelude::*;

    /// Extracts numeric values from a Polars [`Series`](polars::prelude::Series),
    /// casting to `f64`. Null entries become `f64::NAN`.
    fn extract_numeric(series: &polars::prelude::Series) -> Vec<f64> {
        let ca = series
            .cast(&DataType::Float64)
            .unwrap_or_else(|_| {
                panic!(
                    "plotkit-polars: cannot cast series {:?} (dtype {:?}) to Float64",
                    series.name(),
                    series.dtype()
                )
            });
        let ca = ca
            .f64()
            .expect("cast to Float64 always yields f64 chunked array");
        ca.into_iter().map(|opt| opt.unwrap_or(f64::NAN)).collect()
    }

    impl IntoSeries for &polars::prelude::Series {
        /// Converts a borrowed Polars [`Series`](polars::prelude::Series) into
        /// a plotkit [`Series`].
        ///
        /// Supports numeric dtypes: `Float64`, `Float32`, `Int32`, `Int64`,
        /// `UInt32`, `UInt64`. Null values become `f64::NAN`.
        fn into_series(self) -> Series {
            Series::new(extract_numeric(self))
        }
    }

    impl IntoSeries for polars::prelude::Series {
        /// Converts an owned Polars [`Series`](polars::prelude::Series) into
        /// a plotkit [`Series`]. Delegates to the borrowed implementation.
        fn into_series(self) -> Series {
            (&self).into_series()
        }
    }

    impl IntoCategories for &polars::prelude::Series {
        /// Converts a borrowed Polars string [`Series`](polars::prelude::Series)
        /// into plotkit [`Categories`]. Null values become the string `"null"`.
        fn into_categories(self) -> Categories {
            let ca = self.str().unwrap_or_else(|_| {
                panic!(
                    "plotkit-polars: series {:?} (dtype {:?}) is not a string type",
                    self.name(),
                    self.dtype()
                )
            });
            let labels: Vec<String> = ca
                .into_iter()
                .map(|opt| opt.unwrap_or("null").to_owned())
                .collect();
            Categories::new(labels)
        }
    }
}

// -- f32 containers --------------------------------------------------------

impl IntoSeries for Vec<f32> {
    /// Converts a `Vec<f32>` into a [`Series`] by casting each element to `f64`.
    fn into_series(self) -> Series {
        Series::new(self.into_iter().map(|v| v as f64).collect())
    }
}

impl IntoSeries for &[f32] {
    /// Converts an `f32` slice into a [`Series`] by casting each element to `f64`.
    fn into_series(self) -> Series {
        Series::new(self.iter().map(|&v| v as f64).collect())
    }
}

// ---------------------------------------------------------------------------
// Categories
// ---------------------------------------------------------------------------

/// Categorical labels for bar charts, pie charts, and other discrete-axis
/// visualizations.
///
/// Use [`IntoCategories`] to convert common string collections into this
/// type automatically.
#[derive(Debug, Clone)]
pub struct Categories {
    /// The category labels.
    pub labels: Vec<String>,
}

impl Categories {
    /// Creates a new `Categories` from a vector of label strings.
    pub fn new(labels: Vec<String>) -> Self {
        Self { labels }
    }

    /// Returns the number of categories.
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// Returns `true` if there are no categories.
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
}

// ---------------------------------------------------------------------------
// IntoCategories
// ---------------------------------------------------------------------------

/// Trait for types that can be converted into [`Categories`].
pub trait IntoCategories {
    /// Converts this value into [`Categories`].
    fn into_categories(self) -> Categories;
}

impl IntoCategories for &[&str] {
    /// Converts a slice of string slices into [`Categories`].
    fn into_categories(self) -> Categories {
        Categories::new(self.iter().map(|s| (*s).to_owned()).collect())
    }
}

impl IntoCategories for Vec<String> {
    /// Zero-copy conversion from an owned `Vec<String>`.
    fn into_categories(self) -> Categories {
        Categories::new(self)
    }
}

impl IntoCategories for &[String] {
    /// Clones the string slice into [`Categories`].
    fn into_categories(self) -> Categories {
        Categories::new(self.to_vec())
    }
}

impl IntoCategories for Vec<&str> {
    /// Converts a vector of string slices into [`Categories`].
    fn into_categories(self) -> Categories {
        Categories::new(self.into_iter().map(|s| s.to_owned()).collect())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn series_from_vec_f64() {
        let s = vec![1.0, 2.0, 3.0].into_series();
        assert_eq!(s.len(), 3);
        assert_eq!(s.data, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn series_from_slice_f64() {
        let data: &[f64] = &[4.0, 5.0];
        let s = data.into_series();
        assert_eq!(s.data, vec![4.0, 5.0]);
    }

    #[test]
    fn series_from_vec_ref() {
        let v = vec![1.0, 2.0];
        let s = (&v).into_series();
        assert_eq!(s.data, vec![1.0, 2.0]);
    }

    #[test]
    fn series_from_array() {
        let s = [10.0, 20.0, 30.0].into_series();
        assert_eq!(s.data, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn series_from_array_ref() {
        let arr = [7.0, 8.0];
        let s = (&arr).into_series();
        assert_eq!(s.data, vec![7.0, 8.0]);
    }

    #[test]
    fn series_identity() {
        let original = Series::new(vec![1.0]);
        let s = original.into_series();
        assert_eq!(s.data, vec![1.0]);
    }

    #[test]
    fn series_from_range() {
        let s = (0..4).into_series();
        assert_eq!(s.data, vec![0.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    fn series_from_vec_i32() {
        let s = vec![1i32, 2, 3].into_series();
        assert_eq!(s.data, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn series_from_slice_i32() {
        let data: &[i32] = &[10, 20];
        let s = data.into_series();
        assert_eq!(s.data, vec![10.0, 20.0]);
    }

    #[test]
    fn series_from_vec_f32() {
        let s = vec![1.5f32, 2.5].into_series();
        assert_eq!(s.data, vec![1.5f64, 2.5]);
    }

    #[test]
    fn series_from_slice_f32() {
        let data: &[f32] = &[0.1, 0.2];
        let s = data.into_series();
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn series_empty() {
        let s = Series::new(vec![]);
        assert!(s.is_empty());
        assert_eq!(s.min(), None);
        assert_eq!(s.max(), None);
        assert_eq!(s.bounds(), None);
    }

    #[test]
    fn series_min_max_bounds() {
        let s = vec![3.0, 1.0, 4.0, 1.5, 9.0].into_series();
        assert_eq!(s.min(), Some(1.0));
        assert_eq!(s.max(), Some(9.0));
        assert_eq!(s.bounds(), Some((1.0, 9.0)));
    }

    #[test]
    fn series_min_max_ignores_nan() {
        let s = vec![f64::NAN, 2.0, f64::INFINITY, 1.0, f64::NEG_INFINITY].into_series();
        assert_eq!(s.min(), Some(1.0));
        assert_eq!(s.max(), Some(2.0));
    }

    #[test]
    fn categories_from_str_slice() {
        let cats: &[&str] = &["a", "b", "c"];
        let c = cats.into_categories();
        assert_eq!(c.labels, vec!["a", "b", "c"]);
    }

    #[test]
    fn categories_from_vec_string() {
        let c = vec!["x".to_string(), "y".to_string()].into_categories();
        assert_eq!(c.labels, vec!["x", "y"]);
    }

    #[test]
    fn categories_from_string_slice() {
        let v = vec!["p".to_string(), "q".to_string()];
        let c = v.as_slice().into_categories();
        assert_eq!(c.labels, vec!["p", "q"]);
    }

    #[test]
    fn categories_from_vec_str_ref() {
        let c = vec!["foo", "bar"].into_categories();
        assert_eq!(c.labels, vec!["foo", "bar"]);
        assert_eq!(c.len(), 2);
        assert!(!c.is_empty());
    }
}
