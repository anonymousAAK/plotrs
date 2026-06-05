//! Axis scale transformations (data space → axes space).

/// A scale maps data values to the normalized [0, 1] range for axis display.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub enum Scale {
    /// Linear mapping from [min, max] to [0, 1].
    #[default]
    Linear,
    /// Base-10 logarithmic scale. Data must be positive.
    Log10,
    /// Symmetric log scale: linear near zero, logarithmic beyond ±linthresh.
    SymLog {
        /// The range (-linthresh, linthresh) is treated linearly.
        linthresh: f64,
    },
}

impl Scale {
    /// Applies the forward symlog function: sign(v) * (linthresh * (1 + log10(|v| / linthresh)))
    /// for |v| >= linthresh, and v otherwise.
    ///
    /// This produces a continuous, monotonically increasing function that is
    /// linear in [-linthresh, linthresh] and logarithmic outside that range.
    fn symlog(v: f64, linthresh: f64) -> f64 {
        let abs_v = v.abs();
        if abs_v <= linthresh {
            v
        } else {
            v.signum() * linthresh * (1.0 + (abs_v / linthresh).log10())
        }
    }

    /// Applies the inverse symlog function. Inverts [`Self::symlog`].
    fn symlog_inv(v: f64, linthresh: f64) -> f64 {
        let abs_v = v.abs();
        if abs_v <= linthresh {
            v
        } else {
            v.signum() * linthresh * 10.0_f64.powf(abs_v / linthresh - 1.0)
        }
    }

    /// Transforms a data value to the [0, 1] range given the axis [min, max].
    ///
    /// Values outside [min, max] will map outside [0, 1], which can be used for
    /// clipping or extrapolation by the caller.
    pub fn transform(&self, val: f64, min: f64, max: f64) -> f64 {
        match self {
            Scale::Linear => {
                if (max - min).abs() < f64::EPSILON {
                    0.5
                } else {
                    (val - min) / (max - min)
                }
            }
            Scale::Log10 => {
                let log_min = min.max(f64::EPSILON).log10();
                let log_max = max.max(f64::EPSILON).log10();
                let log_val = val.max(f64::EPSILON).log10();
                if (log_max - log_min).abs() < f64::EPSILON {
                    0.5
                } else {
                    (log_val - log_min) / (log_max - log_min)
                }
            }
            Scale::SymLog { linthresh } => {
                let s_min = Self::symlog(min, *linthresh);
                let s_max = Self::symlog(max, *linthresh);
                let s_val = Self::symlog(val, *linthresh);
                if (s_max - s_min).abs() < f64::EPSILON {
                    0.5
                } else {
                    (s_val - s_min) / (s_max - s_min)
                }
            }
        }
    }

    /// Inverse transform: maps a normalized [0, 1] value back to data space.
    pub fn inverse(&self, t: f64, min: f64, max: f64) -> f64 {
        match self {
            Scale::Linear => min + t * (max - min),
            Scale::Log10 => {
                let log_min = min.max(f64::EPSILON).log10();
                let log_max = max.max(f64::EPSILON).log10();
                10.0_f64.powf(log_min + t * (log_max - log_min))
            }
            Scale::SymLog { linthresh } => {
                let s_min = Self::symlog(min, *linthresh);
                let s_max = Self::symlog(max, *linthresh);
                let s_val = s_min + t * (s_max - s_min);
                Self::symlog_inv(s_val, *linthresh)
            }
        }
    }

    /// Returns true if this scale requires strictly positive data values.
    pub fn requires_positive(&self) -> bool {
        matches!(self, Scale::Log10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-12;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < TOL
    }

    // -----------------------------------------------------------------------
    // Linear
    // -----------------------------------------------------------------------

    #[test]
    fn linear_basic() {
        let s = Scale::Linear;
        assert!(approx_eq(s.transform(0.0, 0.0, 10.0), 0.0));
        assert!(approx_eq(s.transform(5.0, 0.0, 10.0), 0.5));
        assert!(approx_eq(s.transform(10.0, 0.0, 10.0), 1.0));
    }

    #[test]
    fn linear_negative_range() {
        let s = Scale::Linear;
        assert!(approx_eq(s.transform(-5.0, -10.0, 0.0), 0.5));
    }

    #[test]
    fn linear_degenerate_range() {
        let s = Scale::Linear;
        assert!(approx_eq(s.transform(5.0, 5.0, 5.0), 0.5));
    }

    #[test]
    fn linear_inverse_roundtrip() {
        let s = Scale::Linear;
        let min = -3.0;
        let max = 7.0;
        for &val in &[-3.0, 0.0, 2.5, 7.0] {
            let t = s.transform(val, min, max);
            let recovered = s.inverse(t, min, max);
            assert!(approx_eq(recovered, val), "roundtrip failed for {val}");
        }
    }

    // -----------------------------------------------------------------------
    // Log10
    // -----------------------------------------------------------------------

    #[test]
    fn log10_basic() {
        let s = Scale::Log10;
        assert!(approx_eq(s.transform(1.0, 1.0, 1000.0), 0.0));
        assert!(approx_eq(s.transform(1000.0, 1.0, 1000.0), 1.0));
        // 10^1.5 ≈ 31.62 is the midpoint in log space between 1 and 1000
        let mid = 10.0_f64.powf(1.5);
        assert!(approx_eq(s.transform(mid, 1.0, 1000.0), 0.5));
    }

    #[test]
    fn log10_degenerate_range() {
        let s = Scale::Log10;
        assert!(approx_eq(s.transform(5.0, 5.0, 5.0), 0.5));
    }

    #[test]
    fn log10_clamps_non_positive() {
        let s = Scale::Log10;
        // Non-positive values should be clamped to EPSILON internally
        let t = s.transform(-1.0, 1.0, 100.0);
        assert!(t.is_finite());
    }

    #[test]
    fn log10_inverse_roundtrip() {
        let s = Scale::Log10;
        let min = 1.0;
        let max = 10000.0;
        for &val in &[1.0, 10.0, 100.0, 1000.0, 10000.0] {
            let t = s.transform(val, min, max);
            let recovered = s.inverse(t, min, max);
            assert!(
                (recovered - val).abs() < 1e-6,
                "roundtrip failed for {val}: got {recovered}"
            );
        }
    }

    #[test]
    fn log10_requires_positive() {
        assert!(Scale::Log10.requires_positive());
        assert!(!Scale::Linear.requires_positive());
        assert!(!Scale::SymLog { linthresh: 1.0 }.requires_positive());
    }

    // -----------------------------------------------------------------------
    // SymLog
    // -----------------------------------------------------------------------

    #[test]
    fn symlog_zero_maps_correctly() {
        let s = Scale::SymLog { linthresh: 1.0 };
        // Zero should be at the expected normalized position for a symmetric range.
        let t = s.transform(0.0, -100.0, 100.0);
        assert!(approx_eq(t, 0.5), "zero should map to 0.5 for symmetric range, got {t}");
    }

    #[test]
    fn symlog_linear_region() {
        let s = Scale::SymLog { linthresh: 10.0 };
        // Within [-linthresh, linthresh] the transform is linear.
        // symlog(5, 10) = 5, symlog(-5, 10) = -5
        let min = -10.0;
        let max = 10.0;
        let t_neg5 = s.transform(-5.0, min, max);
        let t_0 = s.transform(0.0, min, max);
        let t_5 = s.transform(5.0, min, max);
        // Should be evenly spaced in this region.
        assert!(approx_eq(t_0, 0.5));
        assert!(approx_eq(t_5 - t_0, t_0 - t_neg5));
    }

    #[test]
    fn symlog_continuity_at_threshold() {
        // The function should be continuous at ±linthresh.
        let linthresh = 2.0;
        let just_below = linthresh - 1e-14;
        let at_thresh = linthresh;
        let s_below = Scale::symlog(just_below, linthresh);
        let s_at = Scale::symlog(at_thresh, linthresh);
        assert!(
            (s_at - s_below).abs() < 1e-10,
            "discontinuity at +linthresh: {s_below} vs {s_at}"
        );

        let s_below_neg = Scale::symlog(-just_below, linthresh);
        let s_at_neg = Scale::symlog(-at_thresh, linthresh);
        assert!(
            (s_at_neg - s_below_neg).abs() < 1e-10,
            "discontinuity at -linthresh: {s_below_neg} vs {s_at_neg}"
        );
    }

    #[test]
    fn symlog_monotonic() {
        let linthresh = 1.0;
        let vals: Vec<f64> = (-50..=50).map(|i| i as f64 * 0.5).collect();
        for w in vals.windows(2) {
            let a = Scale::symlog(w[0], linthresh);
            let b = Scale::symlog(w[1], linthresh);
            assert!(
                b >= a,
                "symlog not monotonic: symlog({}) = {a}, symlog({}) = {b}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn symlog_inverse_roundtrip() {
        let s = Scale::SymLog { linthresh: 1.0 };
        let min = -1000.0;
        let max = 1000.0;
        let test_vals = [
            -1000.0, -100.0, -10.0, -1.0, -0.5, 0.0, 0.5, 1.0, 10.0, 100.0, 1000.0,
        ];
        for &val in &test_vals {
            let t = s.transform(val, min, max);
            let recovered = s.inverse(t, min, max);
            assert!(
                (recovered - val).abs() < 1e-8,
                "symlog roundtrip failed for {val}: got {recovered} (t={t})"
            );
        }
    }

    #[test]
    fn symlog_inverse_roundtrip_asymmetric() {
        let s = Scale::SymLog { linthresh: 5.0 };
        let min = -20.0;
        let max = 500.0;
        for &val in &[-20.0, -5.0, 0.0, 5.0, 50.0, 500.0] {
            let t = s.transform(val, min, max);
            let recovered = s.inverse(t, min, max);
            assert!(
                (recovered - val).abs() < 1e-8,
                "symlog roundtrip failed for {val}: got {recovered}"
            );
        }
    }

    #[test]
    fn symlog_degenerate_range() {
        let s = Scale::SymLog { linthresh: 1.0 };
        assert!(approx_eq(s.transform(5.0, 5.0, 5.0), 0.5));
    }

    #[test]
    fn symlog_odd_symmetry() {
        // symlog(-v) == -symlog(v) for all v (odd function).
        let linthresh = 3.0;
        for &v in &[0.0, 1.0, 3.0, 10.0, 100.0] {
            let pos = Scale::symlog(v, linthresh);
            let neg = Scale::symlog(-v, linthresh);
            assert!(
                approx_eq(neg, -pos),
                "symlog is not odd-symmetric for v={v}: symlog({v})={pos}, symlog(-{v})={neg}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Boundary / edge-case tests
    // -----------------------------------------------------------------------

    #[test]
    fn transform_at_boundaries() {
        for scale in &[
            Scale::Linear,
            Scale::Log10,
            Scale::SymLog { linthresh: 1.0 },
        ] {
            let (min, max) = match scale {
                Scale::Log10 => (1.0, 100.0),
                _ => (-10.0, 10.0),
            };
            let t_min = scale.transform(min, min, max);
            let t_max = scale.transform(max, min, max);
            assert!(
                approx_eq(t_min, 0.0),
                "{scale:?}: transform(min) should be 0.0, got {t_min}"
            );
            assert!(
                approx_eq(t_max, 1.0),
                "{scale:?}: transform(max) should be 1.0, got {t_max}"
            );
        }
    }

    #[test]
    fn inverse_at_boundaries() {
        for scale in &[
            Scale::Linear,
            Scale::Log10,
            Scale::SymLog { linthresh: 1.0 },
        ] {
            let (min, max) = match scale {
                Scale::Log10 => (1.0, 100.0),
                _ => (-10.0, 10.0),
            };
            let recovered_min = scale.inverse(0.0, min, max);
            let recovered_max = scale.inverse(1.0, min, max);
            assert!(
                (recovered_min - min).abs() < 1e-8,
                "{scale:?}: inverse(0) should be {min}, got {recovered_min}"
            );
            assert!(
                (recovered_max - max).abs() < 1e-8,
                "{scale:?}: inverse(1) should be {max}, got {recovered_max}"
            );
        }
    }
}
