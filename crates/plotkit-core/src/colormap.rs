//! Named colormap system for mapping scalar values to colors.
//!
//! This module provides a [`Colormap`] enum with popular scientific colormaps
//! backed by the [`colorous`] crate. Colormaps are used to map a continuous
//! scalar value in `[0, 1]` (or an arbitrary `[vmin, vmax]` range) to an RGBA
//! [`Color`].
//!
//! # Examples
//!
//! ```
//! use plotkit_core::colormap::Colormap;
//!
//! let cmap = Colormap::Viridis;
//! let color = cmap.map(0.5);
//! assert_eq!(color.a, 255);
//! ```

use crate::primitives::Color;

// ---------------------------------------------------------------------------
// Colormap enum
// ---------------------------------------------------------------------------

/// A named colormap that maps scalar values in `[0, 1]` to [`Color`].
///
/// Each variant corresponds to a well-known scientific or perceptual colormap
/// provided by the [`colorous`] crate. Use [`map`](Colormap::map) for raw
/// `[0, 1]` lookups, or [`map_value`](Colormap::map_value) to normalise from
/// an arbitrary `[vmin, vmax]` range first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Colormap {
    /// Perceptually uniform sequential (dark purple to yellow).
    Viridis,
    /// Perceptually uniform sequential (purple to yellow via pink).
    Plasma,
    /// Perceptually uniform sequential (black to yellow via red).
    Inferno,
    /// Perceptually uniform sequential (black to light yellow via magenta).
    Magma,
    /// Perceptually uniform sequential designed for color-vision deficiency.
    Cividis,
    /// Rainbow-like map with improved perceptual properties.
    Turbo,
    /// Diverging blue-white-red map.
    Coolwarm,
    /// Diverging multi-hue map.
    Spectral,
    /// Sequential single-hue blue map.
    Blues,
    /// Sequential single-hue red map.
    Reds,
    /// Sequential single-hue green map.
    Greens,
    /// Sequential single-hue grey map.
    Greys,
    /// Sequential multi-hue yellow-orange-red map.
    YlOrRd,
    /// Diverging red-blue map.
    RdBu,
}

impl Colormap {
    /// Maps a value `t` in `[0, 1]` to a fully opaque [`Color`].
    ///
    /// Values outside `[0, 1]` are clamped. The mapping uses the underlying
    /// [`colorous`] gradient's `eval_continuous` method for smooth
    /// interpolation across the full colour range.
    pub fn map(&self, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);
        let c = self.gradient().eval_continuous(t);
        Color::rgb(c.r, c.g, c.b)
    }

    /// Normalises `val` from `[vmin, vmax]` to `[0, 1]`, then maps to a
    /// [`Color`].
    ///
    /// When `vmin == vmax` (zero-width range), the result is `map(0.5)`.
    pub fn map_value(&self, val: f64, vmin: f64, vmax: f64) -> Color {
        let t = if (vmax - vmin).abs() < f64::EPSILON {
            0.5
        } else {
            (val - vmin) / (vmax - vmin)
        };
        self.map(t)
    }

    /// Batch-maps a slice of values, auto-detecting `[vmin, vmax]` from the
    /// finite values in `vals`.
    ///
    /// Non-finite values (`NaN`, `Inf`) are excluded from the min/max
    /// calculation but are still mapped (they clamp to 0 or 1 depending on
    /// sign).
    pub fn map_values(&self, vals: &[f64]) -> Vec<Color> {
        let (vmin, vmax) = finite_bounds(vals);
        vals.iter().map(|&v| self.map_value(v, vmin, vmax)).collect()
    }

    /// Returns the [`colorous::Gradient`] for this colormap variant.
    fn gradient(&self) -> colorous::Gradient {
        match self {
            Colormap::Viridis => colorous::VIRIDIS,
            Colormap::Plasma => colorous::PLASMA,
            Colormap::Inferno => colorous::INFERNO,
            Colormap::Magma => colorous::MAGMA,
            Colormap::Cividis => colorous::CIVIDIS,
            Colormap::Turbo => colorous::TURBO,
            Colormap::Coolwarm => colorous::COOL,
            Colormap::Spectral => colorous::SPECTRAL,
            Colormap::Blues => colorous::BLUES,
            Colormap::Reds => colorous::REDS,
            Colormap::Greens => colorous::GREENS,
            Colormap::Greys => colorous::GREYS,
            Colormap::YlOrRd => colorous::YELLOW_ORANGE_RED,
            Colormap::RdBu => colorous::RED_BLUE,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns `(min, max)` of the finite values in `vals`.
///
/// Falls back to `(0.0, 1.0)` when the slice is empty or contains no finite
/// values.
fn finite_bounds(vals: &[f64]) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &v in vals {
        if v.is_finite() {
            if v < lo {
                lo = v;
            }
            if v > hi {
                hi = v;
            }
        }
    }
    if lo.is_finite() && hi.is_finite() {
        (lo, hi)
    } else {
        (0.0, 1.0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viridis_at_zero_is_dark_purple() {
        let c = Colormap::Viridis.map(0.0);
        // colorous VIRIDIS at 0.0 is approximately (68, 1, 84).
        assert_eq!(c.r, 68);
        assert_eq!(c.g, 1);
        assert_eq!(c.b, 84);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn viridis_at_one_is_yellow() {
        let c = Colormap::Viridis.map(1.0);
        // colorous VIRIDIS at 1.0 is approximately (253, 231, 37).
        assert_eq!(c.r, 253);
        assert_eq!(c.g, 231);
        assert_eq!(c.b, 37);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn viridis_midpoint_is_teal() {
        let c = Colormap::Viridis.map(0.5);
        // The midpoint should be a teal/green colour.
        assert!(c.g > c.r, "midpoint green channel should exceed red");
        assert!(c.g > c.b, "midpoint green channel should exceed blue");
        assert_eq!(c.a, 255);
    }

    #[test]
    fn map_clamps_below_zero() {
        let c = Colormap::Viridis.map(-0.5);
        let c0 = Colormap::Viridis.map(0.0);
        assert_eq!(c, c0);
    }

    #[test]
    fn map_clamps_above_one() {
        let c = Colormap::Viridis.map(1.5);
        let c1 = Colormap::Viridis.map(1.0);
        assert_eq!(c, c1);
    }

    #[test]
    fn map_value_normalises_correctly() {
        let c = Colormap::Viridis.map_value(50.0, 0.0, 100.0);
        let expected = Colormap::Viridis.map(0.5);
        assert_eq!(c, expected);
    }

    #[test]
    fn map_value_equal_bounds_gives_midpoint() {
        let c = Colormap::Plasma.map_value(5.0, 5.0, 5.0);
        let expected = Colormap::Plasma.map(0.5);
        assert_eq!(c, expected);
    }

    #[test]
    fn map_values_batch() {
        let vals = vec![0.0, 50.0, 100.0];
        let colors = Colormap::Viridis.map_values(&vals);
        assert_eq!(colors.len(), 3);
        assert_eq!(colors[0], Colormap::Viridis.map(0.0));
        assert_eq!(colors[1], Colormap::Viridis.map(0.5));
        assert_eq!(colors[2], Colormap::Viridis.map(1.0));
    }

    #[test]
    fn map_values_empty_slice() {
        let colors = Colormap::Viridis.map_values(&[]);
        assert!(colors.is_empty());
    }

    #[test]
    fn map_values_single_element() {
        let colors = Colormap::Viridis.map_values(&[42.0]);
        assert_eq!(colors.len(), 1);
        // Single element: vmin == vmax, so t = 0.5.
        assert_eq!(colors[0], Colormap::Viridis.map(0.5));
    }

    #[test]
    fn all_colormaps_produce_opaque_colors() {
        let all = [
            Colormap::Viridis,
            Colormap::Plasma,
            Colormap::Inferno,
            Colormap::Magma,
            Colormap::Cividis,
            Colormap::Turbo,
            Colormap::Coolwarm,
            Colormap::Spectral,
            Colormap::Blues,
            Colormap::Reds,
            Colormap::Greens,
            Colormap::Greys,
            Colormap::YlOrRd,
            Colormap::RdBu,
        ];
        for cmap in &all {
            for &t in &[0.0, 0.25, 0.5, 0.75, 1.0] {
                let c = cmap.map(t);
                assert_eq!(c.a, 255, "{cmap:?} at t={t} should be fully opaque");
            }
        }
    }

    #[test]
    fn plasma_endpoints_differ() {
        let lo = Colormap::Plasma.map(0.0);
        let hi = Colormap::Plasma.map(1.0);
        assert_ne!(lo, hi, "plasma endpoints should be different colors");
    }

    #[test]
    fn finite_bounds_skips_nan() {
        let vals = vec![f64::NAN, 1.0, 5.0, f64::NAN];
        let (lo, hi) = finite_bounds(&vals);
        assert!((lo - 1.0).abs() < f64::EPSILON);
        assert!((hi - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn finite_bounds_all_nan_fallback() {
        let vals = vec![f64::NAN, f64::NAN];
        let (lo, hi) = finite_bounds(&vals);
        assert!((lo - 0.0).abs() < f64::EPSILON);
        assert!((hi - 1.0).abs() < f64::EPSILON);
    }
}
