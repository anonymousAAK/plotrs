//! Axis tick generation using the Talbot/Wilkinson extended algorithm.
//!
//! This module implements the "An Extension of Wilkinson's Algorithm for Positioning Tick Labels
//! on Axes" algorithm (Talbot, Lin, Hanrahan, 2010). It produces aesthetically pleasing tick
//! positions by jointly optimizing four objectives: simplicity, coverage, density, and legibility.

use crate::scale::Scale;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A single tick mark with its position in data space and a formatted label.
#[derive(Debug, Clone)]
pub struct Tick {
    /// The position of this tick in data coordinates.
    pub value: f64,
    /// A human-readable label for this tick (e.g. "0.2", "100").
    pub label: String,
}

/// A set of tick positions with formatted labels.
#[derive(Debug, Clone)]
pub struct TickSet {
    /// Tick positions in data space.
    pub positions: Vec<f64>,
    /// Formatted string labels corresponding to each position.
    pub labels: Vec<String>,
}

impl TickSet {
    /// Converts this `TickSet` into a `Vec<Tick>`.
    pub fn into_ticks(self) -> Vec<Tick> {
        self.positions
            .into_iter()
            .zip(self.labels)
            .map(|(value, label)| Tick { value, label })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Generates optimal tick positions for an axis range.
///
/// Uses the Talbot/Wilkinson extended algorithm to find "nice" tick values
/// that balance simplicity, coverage, density, and legibility.
///
/// # Arguments
/// - `data_min` / `data_max`: the data range
/// - `target_count`: desired number of ticks (typically 5-10)
/// - `scale`: the axis scale (Linear, Log10, etc.)
///
/// # Returns
/// A `Vec<Tick>` with positions and formatted labels.
pub fn generate_ticks(
    data_min: f64,
    data_max: f64,
    target_count: usize,
    scale: &Scale,
) -> Vec<Tick> {
    let tick_set = match scale {
        Scale::Linear => generate_linear_ticks(data_min, data_max, target_count),
        Scale::Log10 => generate_log_ticks(data_min, data_max, target_count),
        Scale::SymLog { linthresh } => {
            generate_symlog_ticks(data_min, data_max, target_count, *linthresh)
        }
    };
    tick_set.into_ticks()
}

// ---------------------------------------------------------------------------
// Talbot/Wilkinson extended algorithm for linear scales
// ---------------------------------------------------------------------------

/// Preference-ordered "nice" step base numbers (Q sequence from the paper).
/// Earlier entries score higher on simplicity.
const Q: [f64; 6] = [1.0, 5.0, 2.0, 2.5, 4.0, 3.0];

/// Weights for the four scoring objectives.
const W_SIMPLICITY: f64 = 0.2;
const W_COVERAGE: f64 = 0.25;
const W_DENSITY: f64 = 0.35;
const W_LEGIBILITY: f64 = 0.2;

/// Generates optimally-spaced ticks for a linear (or symlog) axis.
///
/// Implements the Talbot/Wilkinson extended algorithm:
///   For each candidate base `q` in Q, for a range of exponents `k`, and for
///   a range of tick counts `j`, compute candidate tick sequences and score them.
///   Return the sequence with the highest weighted score.
fn generate_linear_ticks(data_min: f64, data_max: f64, target_count: usize) -> TickSet {
    // Handle degenerate ranges.
    if !data_min.is_finite() || !data_max.is_finite() {
        return make_tick_set(vec![0.0]);
    }

    let (dmin, dmax) = if (data_max - data_min).abs() < f64::EPSILON * 100.0 {
        // Degenerate: single-point range -- widen it.
        if data_min == 0.0 {
            (-1.0, 1.0)
        } else {
            let delta = data_min.abs() * 0.1;
            (data_min - delta, data_min + delta)
        }
    } else if data_min > data_max {
        (data_max, data_min)
    } else {
        (data_min, data_max)
    };

    let target = target_count.max(2) as f64;
    let range = dmax - dmin;

    let mut best_score = f64::NEG_INFINITY;
    let mut best_ticks: Option<Vec<f64>> = None;

    // Iterate over candidate base numbers.
    for (qi, &q) in Q.iter().enumerate() {
        // Iterate over candidate tick counts (j = number of ticks - 1, i.e. intervals).
        // We search a range around the target count.
        let j_min = 1_usize;
        let j_max = (target as usize * 3).max(12);

        for j in j_min..=j_max {
            let j_f = j as f64;

            // Density score for this tick count (before we know the exact positions).
            let density = density_score(j_f + 1.0, target, range);
            // Early skip: if the maximum possible total score can't beat the best, skip.
            let max_possible = W_SIMPLICITY + W_COVERAGE + W_DENSITY * density + W_LEGIBILITY;
            if max_possible < best_score {
                continue;
            }

            // Determine the magnitude (power of 10) for the step size.
            // step = q * 10^k, and we want j * step ~ range.
            let ideal_step = range / j_f;
            // k such that q * 10^k ~ ideal_step  =>  k ~ log10(ideal_step / q)
            let k_float = (ideal_step / q).log10().floor();

            // Search a small neighbourhood of k to find the best exponent.
            for k_offset in -2_i32..=2 {
                let k = k_float as i32 + k_offset;
                let step = q * 10.0_f64.powi(k);

                if step <= 0.0 || !step.is_finite() {
                    continue;
                }

                // We try a few different starting points to find the best coverage.
                // The paper iterates over `i` (the index offset for the starting tick).
                // We search offsets that keep the ticks close to covering [dmin, dmax].
                let i_min = ((dmin / step).ceil() - j_f) as i64;
                let i_max = (dmin / step).floor() as i64 + 1;

                for i in i_min..=i_max {
                    let tick_min = i as f64 * step;
                    let tick_max = tick_min + j_f * step;

                    // Ticks must span at least the data range (or come close).
                    if tick_max < dmax - step * 0.5 {
                        continue;
                    }
                    if tick_min > dmin + step * 0.5 {
                        continue;
                    }

                    // Generate the tick positions.
                    let num_ticks = j + 1;
                    let ticks: Vec<f64> = (0..num_ticks)
                        .map(|t| {
                            let v = tick_min + t as f64 * step;
                            // Round to cancel floating-point accumulation error.
                            snap_to_step(v, step)
                        })
                        .collect();

                    // Score this candidate.
                    let simplicity = simplicity_score(qi, &ticks);
                    let coverage = coverage_score(tick_min, tick_max, dmin, dmax);
                    let density = density_score(num_ticks as f64, target, range);
                    let legibility = legibility_score(&ticks);

                    let score = W_SIMPLICITY * simplicity
                        + W_COVERAGE * coverage
                        + W_DENSITY * density
                        + W_LEGIBILITY * legibility;

                    if score > best_score {
                        best_score = score;
                        best_ticks = Some(ticks);
                    }
                }
            }
        }
    }

    let ticks = best_ticks.unwrap_or_else(|| {
        // Ultimate fallback: evenly divide.
        let step = range / target;
        (0..=target as usize)
            .map(|i| dmin + i as f64 * step)
            .collect()
    });

    make_tick_set(ticks)
}

// ---------------------------------------------------------------------------
// Scoring functions
// ---------------------------------------------------------------------------

/// Simplicity score: rewards choosing earlier (more "round") entries in Q,
/// and gives a bonus when zero is included among the ticks.
fn simplicity_score(q_index: usize, ticks: &[f64]) -> f64 {
    let q_len = Q.len() as f64;
    // i ranges from 0 (best) to Q.len()-1 (worst). Map to [0, 1].
    let q_penalty = q_index as f64 / q_len;
    // Bonus for including zero.
    let zero_bonus = if ticks.iter().any(|&v| v.abs() < f64::EPSILON * 100.0) {
        1.0
    } else {
        0.0
    };
    1.0 - q_penalty + zero_bonus * 0.2
}

/// Coverage score: rewards tick ranges that tightly cover [dmin, dmax].
///
/// The score is based on the ratio `data_range / tick_range`. A perfect score
/// of 1.0 means the tick range exactly matches the data range. Overshoot
/// (tick_range > data_range) is penalized mildly. Undershoot (ticks that don't
/// span the data) returns 0.0 -- we never want ticks that miss part of the data.
fn coverage_score(tick_min: f64, tick_max: f64, dmin: f64, dmax: f64) -> f64 {
    let data_range = dmax - dmin;
    if data_range <= 0.0 {
        return 1.0;
    }
    // Hard reject: ticks must fully cover the data range.
    if tick_min > dmin + data_range * 0.001 || tick_max < dmax - data_range * 0.001 {
        return 0.0;
    }
    let tick_range = tick_max - tick_min;
    // coverage = 1 - 0.5 * ((tick_range - data_range) / data_range)^2
    // This gently penalizes overshoot while rewarding tight coverage.
    let overshoot_ratio = (tick_range - data_range) / data_range;
    (1.0 - 0.5 * overshoot_ratio * overshoot_ratio).max(0.0)
}

/// Density score: rewards tick counts close to the target. Uses a Gaussian-like
/// falloff centered on `target`.
fn density_score(num_ticks: f64, target: f64, _range: f64) -> f64 {
    let ratio = if target > 0.0 {
        num_ticks / target
    } else {
        1.0
    };
    // Score peaks at ratio = 1, falls off symmetrically.
    // Using 2 - max(ratio, 1/ratio) gives a nice [0,1] range for reasonable ratios.
    let raw = 2.0 - ratio.max(1.0 / ratio);
    raw.clamp(0.0, 1.0)
}

/// Legibility score: rewards ticks that produce short, clean formatted strings.
/// Penalizes very long labels (many decimal places) or scientific notation.
fn legibility_score(ticks: &[f64]) -> f64 {
    if ticks.is_empty() {
        return 1.0;
    }
    let total: f64 = ticks.iter().map(|&v| single_legibility(v)).sum();
    total / ticks.len() as f64
}

/// Legibility of a single tick value.
fn single_legibility(value: f64) -> f64 {
    let label = format_tick(value);
    let len = label.len();
    // Short labels are best. Penalize progressively.
    if len <= 3 {
        1.0
    } else if len <= 5 {
        0.9
    } else if len <= 7 {
        0.75
    } else if len <= 10 {
        0.5
    } else {
        0.3
    }
}

// ---------------------------------------------------------------------------
// Log-scale ticks
// ---------------------------------------------------------------------------

/// Generates ticks for a log10 scale. Places ticks at powers of 10, with
/// optional sub-decade ticks when the range is small.
fn generate_log_ticks(data_min: f64, data_max: f64, target_count: usize) -> TickSet {
    let lo = data_min.max(f64::EPSILON);
    let hi = data_max.max(lo);

    let log_lo = lo.log10().floor() as i32;
    let log_hi = hi.log10().ceil() as i32;

    let decades = (log_hi - log_lo) as usize;

    if decades <= 1 {
        // Very narrow log range: fall back to linear-style ticks within this range.
        return generate_linear_ticks(lo, hi, target_count);
    }

    let mut positions = Vec::new();

    if decades <= 3 {
        // Few decades: include sub-decade ticks at 2, 5.
        for exp in log_lo..=log_hi {
            let base = 10.0_f64.powi(exp);
            for &mult in &[1.0, 2.0, 5.0] {
                let val = base * mult;
                if val >= lo * 0.999 && val <= hi * 1.001 {
                    positions.push(val);
                }
            }
        }
    } else {
        // Many decades: only powers of 10.
        // If there are too many decades, skip some.
        let skip = ((decades as f64) / (target_count.max(2) as f64)).ceil() as i32;
        let skip = skip.max(1);
        let mut exp = log_lo;
        while exp <= log_hi {
            let val = 10.0_f64.powi(exp);
            if val >= lo * 0.999 && val <= hi * 1.001 {
                positions.push(val);
            }
            exp += skip;
        }
        // Always include the last power.
        let last = 10.0_f64.powi(log_hi);
        if positions.last().map_or(true, |&v| (v - last).abs() > f64::EPSILON)
            && last <= hi * 1.001 {
                positions.push(last);
        }
    }

    if positions.is_empty() {
        positions.push(lo);
        positions.push(hi);
    }

    make_tick_set(positions)
}

/// Generates minor tick positions for a log10 scale.
///
/// Minor ticks are placed at multiples 2, 3, 4, 5, 6, 7, 8, 9 of each power
/// of 10 within the given data range. These are the sub-decade ticks that give
/// log-scale plots their characteristic visual pattern.
///
/// Returns only positions (no labels), since minor ticks are typically drawn
/// without labels.
pub fn generate_log_minor_ticks(data_min: f64, data_max: f64) -> Vec<f64> {
    let lo = data_min.max(f64::EPSILON);
    let hi = data_max.max(lo);

    let log_lo = lo.log10().floor() as i32;
    let log_hi = hi.log10().ceil() as i32;

    let mut positions = Vec::new();

    for exp in log_lo..=log_hi {
        let base = 10.0_f64.powi(exp);
        for mult in 2..=9 {
            let val = base * mult as f64;
            if val >= lo * 0.999 && val <= hi * 1.001 {
                positions.push(val);
            }
        }
    }

    positions
}

/// Generates ticks for a symlog (symmetric log) scale.
///
/// Produces ticks that reflect the symmetry of the symlog transform: logarithmic
/// ticks for the positive and negative regions beyond `linthresh`, and linear
/// ticks in the `[-linthresh, linthresh]` region.
fn generate_symlog_ticks(data_min: f64, data_max: f64, target_count: usize, linthresh: f64) -> TickSet {
    // Guard: if linthresh is non-positive or non-finite, fall back to linear ticks.
    if linthresh <= 0.0 || !linthresh.is_finite() {
        return generate_linear_ticks(data_min, data_max, target_count);
    }

    let mut positions = Vec::new();

    // Always include zero if the range crosses it.
    if data_min <= 0.0 && data_max >= 0.0 {
        positions.push(0.0);
    }

    // Add +-linthresh markers if they fall within the range.
    if linthresh <= data_max && linthresh >= data_min {
        positions.push(linthresh);
    }
    if -linthresh >= data_min && -linthresh <= data_max {
        positions.push(-linthresh);
    }

    // Positive logarithmic region: powers of 10 beyond linthresh.
    if data_max > linthresh {
        let log_lo = linthresh.log10().ceil() as i32;
        let log_hi = data_max.abs().log10().ceil() as i32;
        for exp in log_lo..=log_hi {
            let val = 10.0_f64.powi(exp);
            if val > linthresh && val <= data_max * 1.001 {
                positions.push(val);
            }
        }
    }

    // Negative logarithmic region: negative powers of 10 beyond -linthresh.
    if data_min < -linthresh {
        let log_lo = linthresh.log10().ceil() as i32;
        let log_hi = data_min.abs().log10().ceil() as i32;
        for exp in log_lo..=log_hi {
            let val = -10.0_f64.powi(exp);
            if val < -linthresh && val >= data_min * 1.001 {
                positions.push(val);
            }
        }
    }

    // If the linear region is significant, add a few linear ticks within it.
    let lin_lo = data_min.max(-linthresh);
    let lin_hi = data_max.min(linthresh);
    if lin_hi > lin_lo {
        let lin_ticks = generate_linear_ticks(lin_lo, lin_hi, (target_count / 3).max(2));
        for &pos in &lin_ticks.positions {
            positions.push(pos);
        }
    }

    // Deduplicate and sort.
    positions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    positions.dedup_by(|a, b| (*a - *b).abs() < f64::EPSILON * 100.0);

    // If we ended up with too few ticks, fall back to linear.
    if positions.len() < 2 {
        return generate_linear_ticks(data_min, data_max, target_count);
    }

    make_tick_set(positions)
}

// ---------------------------------------------------------------------------
// Tick formatting
// ---------------------------------------------------------------------------

/// Formats a single tick value to a compact, human-readable string.
///
/// This is the public entry point used when custom tick positions are set
/// without explicit labels. Delegates to the internal `format_tick` function.
pub fn format_tick_value(value: f64) -> String {
    format_tick(value)
}

/// Formats a tick value to a compact string.
///
/// Uses fixed notation for values whose absolute magnitude is in [0.001, 999_999].
/// Uses scientific notation otherwise. Strips trailing zeros from the fractional part.
fn format_tick(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }

    let abs = value.abs();

    if (0.001..1_000_000.0).contains(&abs) {
        // Determine how many decimal places we need.
        // We want enough to distinguish this value, but not excessive.
        let decimals = needed_decimals(value);
        let formatted = format!("{:.prec$}", value, prec = decimals);
        strip_trailing_zeros(&formatted)
    } else {
        // Scientific notation.
        let formatted = format!("{:.6e}", value);
        clean_scientific(&formatted)
    }
}

/// Determines how many decimal places are needed to represent a value faithfully.
/// Returns at most 10 decimal places.
fn needed_decimals(value: f64) -> usize {
    let abs = value.abs();
    if abs == abs.floor() && abs < 1e15 {
        return 0;
    }
    // Try increasing precision until the round-trip is faithful.
    for d in 1..=10 {
        let factor = 10.0_f64.powi(d as i32);
        let rounded = (value * factor).round() / factor;
        if (rounded - value).abs() < f64::EPSILON * abs.max(1.0) * 10.0 {
            return d;
        }
    }
    10
}

/// Strips trailing zeros after a decimal point. E.g. "1.200" -> "1.2", "3.0" -> "3".
fn strip_trailing_zeros(s: &str) -> String {
    if !s.contains('.') {
        return s.to_string();
    }
    let trimmed = s.trim_end_matches('0');
    let trimmed = trimmed.trim_end_matches('.');
    trimmed.to_string()
}

/// Cleans scientific notation: "1.000000e0" -> "1e0", strips trailing zeros in mantissa.
fn clean_scientific(s: &str) -> String {
    if let Some(e_pos) = s.find('e') {
        let mantissa = &s[..e_pos];
        let exponent = &s[e_pos..]; // includes 'e'
        let cleaned_mantissa = strip_trailing_zeros(mantissa);
        format!("{}{}", cleaned_mantissa, exponent)
    } else {
        s.to_string()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Rounds a value to the nearest multiple of `step`, eliminating floating-point
/// accumulation errors such as `0.2 + 0.2 + 0.2 = 0.6000000000000001`.
///
/// Works by normalizing the step to remove its power-of-10 magnitude, computing
/// the snap in that normalized space, then scaling back. This ensures correct
/// behavior for both large steps (e.g. 200000) and tiny steps (e.g. 2e-11).
fn snap_to_step(value: f64, step: f64) -> f64 {
    if step == 0.0 {
        return value;
    }

    // Compute the integer multiple of step closest to value.
    let n = (value / step).round();
    let mut result = n * step;

    // Normalize: figure out how many digits the step's "significant part" needs.
    // E.g. step=0.2 -> magnitude=-1, mantissa=2.0
    //      step=200  -> magnitude=2, mantissa=2.0
    //      step=2.5  -> magnitude=0, mantissa=2.5
    let magnitude = step.abs().log10().floor() as i32;
    // Number of decimals needed for the mantissa (step / 10^magnitude).
    let mantissa = step.abs() / 10.0_f64.powi(magnitude);
    let mantissa_decimals = {
        let mut d = 0usize;
        for test_d in 0..=5 {
            let factor = 10.0_f64.powi(test_d as i32);
            let scaled = mantissa * factor;
            if (scaled - scaled.round()).abs() < 1e-6 {
                d = test_d;
                break;
            }
            d = test_d;
        }
        d
    };
    // Total number of fractional digits in step = mantissa_decimals - magnitude
    // (when magnitude is negative, we need more decimals).
    let total_decimals = (mantissa_decimals as i32 - magnitude).max(0) as u32;
    if total_decimals <= 15 {
        let factor = 10.0_f64.powi(total_decimals as i32);
        result = (result * factor).round() / factor;
    }

    // Snap very-near-zero results to exactly zero.
    if result.abs() < step.abs() * 1e-10 {
        0.0
    } else {
        result
    }
}

/// Creates a `TickSet` from a list of positions, generating labels via `format_tick`.
fn make_tick_set(positions: Vec<f64>) -> TickSet {
    let labels = positions.iter().map(|&v| format_tick(v)).collect();
    TickSet { positions, labels }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Helper to extract positions from the Vec<Tick> returned by generate_ticks.
    fn positions(ticks: &[Tick]) -> Vec<f64> {
        ticks.iter().map(|t| t.value).collect()
    }

    /// Helper to extract labels from the Vec<Tick> returned by generate_ticks.
    fn labels(ticks: &[Tick]) -> Vec<&str> {
        ticks.iter().map(|t| t.label.as_str()).collect()
    }

    /// Asserts that a Vec<Tick> has valid, sorted, non-empty entries.
    fn assert_nice(ticks: &[Tick]) {
        assert!(!ticks.is_empty(), "tick set should not be empty");
        for w in ticks.windows(2) {
            assert!(
                w[1].value >= w[0].value,
                "ticks must be sorted: {} came before {}",
                w[0].value,
                w[1].value
            );
        }
    }

    /// Asserts that the ticks cover the data range (first tick <= dmin, last tick >= dmax,
    /// or within one step of the boundary).
    fn assert_covers(ticks: &[Tick], dmin: f64, dmax: f64) {
        let first = ticks.first().unwrap().value;
        let last = ticks.last().unwrap().value;
        let step = if ticks.len() >= 2 {
            ticks[1].value - ticks[0].value
        } else {
            (dmax - dmin).abs().max(1.0)
        };
        assert!(
            first <= dmin + step * 0.01,
            "first tick {} should be <= data_min {} (step={})",
            first,
            dmin,
            step
        );
        assert!(
            last >= dmax - step * 0.01,
            "last tick {} should be >= data_max {} (step={})",
            last,
            dmax,
            step
        );
    }

    // -----------------------------------------------------------------------
    // Range [0, 10]
    // -----------------------------------------------------------------------

    #[test]
    fn range_0_10() {
        let ticks = generate_ticks(0.0, 10.0, 6, &Scale::Linear);
        assert_nice(&ticks);
        assert_covers(&ticks, 0.0, 10.0);
        // Should produce ticks with step 2: 0, 2, 4, 6, 8, 10.
        assert_eq!(positions(&ticks), vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
        assert_eq!(labels(&ticks), vec!["0", "2", "4", "6", "8", "10"]);
    }

    // -----------------------------------------------------------------------
    // Range [0, 1]
    // -----------------------------------------------------------------------

    #[test]
    fn range_0_1() {
        let ticks = generate_ticks(0.0, 1.0, 6, &Scale::Linear);
        assert_nice(&ticks);
        assert_covers(&ticks, 0.0, 1.0);
        assert_eq!(positions(&ticks), vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
        assert_eq!(labels(&ticks), vec!["0", "0.2", "0.4", "0.6", "0.8", "1"]);
    }

    // -----------------------------------------------------------------------
    // Range [-5, 5]
    // -----------------------------------------------------------------------

    #[test]
    fn range_neg5_pos5() {
        let ticks = generate_ticks(-5.0, 5.0, 6, &Scale::Linear);
        assert_nice(&ticks);
        assert_covers(&ticks, -5.0, 5.0);
        let pos = positions(&ticks);
        // The algorithm should include 0 and cover [-5, 5].
        assert!(
            pos.contains(&0.0),
            "ticks for [-5,5] should include zero: {:?}",
            pos
        );
        assert!(*pos.first().unwrap() <= -5.0);
        assert!(*pos.last().unwrap() >= 5.0);
    }

    // -----------------------------------------------------------------------
    // Range [0, 100]
    // -----------------------------------------------------------------------

    #[test]
    fn range_0_100() {
        let ticks = generate_ticks(0.0, 100.0, 6, &Scale::Linear);
        assert_nice(&ticks);
        assert_covers(&ticks, 0.0, 100.0);
        assert_eq!(positions(&ticks), vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
        assert_eq!(labels(&ticks), vec!["0", "20", "40", "60", "80", "100"]);
    }

    // -----------------------------------------------------------------------
    // Range [0, 1_000_000]
    // -----------------------------------------------------------------------

    #[test]
    fn range_0_1e6() {
        let ticks = generate_ticks(0.0, 1_000_000.0, 6, &Scale::Linear);
        assert_nice(&ticks);
        assert_covers(&ticks, 0.0, 1_000_000.0);
        // Should produce ticks at multiples of 200000.
        assert_eq!(
            positions(&ticks),
            vec![0.0, 200_000.0, 400_000.0, 600_000.0, 800_000.0, 1_000_000.0]
        );
    }

    // -----------------------------------------------------------------------
    // Range [0.001, 0.01]
    // -----------------------------------------------------------------------

    #[test]
    fn range_0001_001() {
        let ticks = generate_ticks(0.001, 0.01, 6, &Scale::Linear);
        assert_nice(&ticks);
        assert_covers(&ticks, 0.001, 0.01);
        let pos = positions(&ticks);
        let first = *pos.first().unwrap();
        let last = *pos.last().unwrap();
        assert!(first <= 0.001 + 1e-12);
        assert!(last >= 0.01 - 1e-12);
    }

    // -----------------------------------------------------------------------
    // Tick count is reasonable
    // -----------------------------------------------------------------------

    #[test]
    fn tick_count_reasonable() {
        for (lo, hi) in &[
            (0.0, 10.0),
            (0.0, 1.0),
            (-100.0, 100.0),
            (0.0, 0.005),
            (1.0, 2.0),
        ] {
            let ticks = generate_ticks(*lo, *hi, 6, &Scale::Linear);
            assert!(
                ticks.len() >= 3 && ticks.len() <= 15,
                "range [{}, {}] produced {} ticks (expected 3-15): {:?}",
                lo,
                hi,
                ticks.len(),
                positions(&ticks)
            );
        }
    }

    // -----------------------------------------------------------------------
    // Degenerate / edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn degenerate_same_min_max() {
        let ticks = generate_ticks(5.0, 5.0, 6, &Scale::Linear);
        assert!(!ticks.is_empty(), "should produce ticks even for degenerate range");
    }

    #[test]
    fn degenerate_zero_range() {
        let ticks = generate_ticks(0.0, 0.0, 6, &Scale::Linear);
        assert!(!ticks.is_empty());
    }

    #[test]
    fn reversed_range() {
        let ticks = generate_ticks(10.0, 0.0, 6, &Scale::Linear);
        assert_nice(&ticks);
        // Should be equivalent to [0, 10].
        assert!(ticks.first().unwrap().value <= 0.0 + 0.01);
        assert!(ticks.last().unwrap().value >= 10.0 - 0.01);
    }

    // -----------------------------------------------------------------------
    // Log-scale ticks
    // -----------------------------------------------------------------------

    #[test]
    fn log_ticks_basic() {
        let ticks = generate_ticks(1.0, 10000.0, 5, &Scale::Log10);
        assert_nice(&ticks);
        assert!(!ticks.is_empty());
        // All positions should be positive.
        for t in &ticks {
            assert!(t.value > 0.0, "log tick should be positive: {}", t.value);
        }
    }

    #[test]
    fn log_ticks_narrow() {
        // Narrow range within a single decade: falls back to linear.
        let ticks = generate_ticks(1.0, 5.0, 5, &Scale::Log10);
        assert!(!ticks.is_empty());
    }

    // -----------------------------------------------------------------------
    // Formatting
    // -----------------------------------------------------------------------

    #[test]
    fn format_zero() {
        assert_eq!(format_tick(0.0), "0");
    }

    #[test]
    fn format_integer() {
        assert_eq!(format_tick(42.0), "42");
        assert_eq!(format_tick(-7.0), "-7");
    }

    #[test]
    fn format_decimal() {
        assert_eq!(format_tick(0.5), "0.5");
        assert_eq!(format_tick(2.5), "2.5");
        assert_eq!(format_tick(0.25), "0.25");
    }

    #[test]
    fn format_no_trailing_zeros() {
        assert_eq!(format_tick(1.0), "1");
        assert_eq!(format_tick(10.0), "10");
        assert_eq!(format_tick(0.2), "0.2");
    }

    #[test]
    fn format_scientific() {
        let label = format_tick(1e-8);
        assert!(
            label.contains('e'),
            "very small numbers should use scientific notation: {}",
            label
        );
    }

    // -----------------------------------------------------------------------
    // SymLog scale dispatch
    // -----------------------------------------------------------------------

    #[test]
    fn symlog_ticks() {
        let ticks = generate_ticks(-100.0, 100.0, 6, &Scale::SymLog { linthresh: 1.0 });
        assert_nice(&ticks);
        let pos = positions(&ticks);
        assert!(
            pos.contains(&0.0),
            "symlog ticks for symmetric range should include zero: {:?}",
            pos
        );
    }

    // -----------------------------------------------------------------------
    // Strip trailing zeros helper
    // -----------------------------------------------------------------------

    #[test]
    fn strip_zeros() {
        assert_eq!(strip_trailing_zeros("1.200"), "1.2");
        assert_eq!(strip_trailing_zeros("3.0"), "3");
        assert_eq!(strip_trailing_zeros("100"), "100");
        assert_eq!(strip_trailing_zeros("0.00100"), "0.001");
    }

    // -----------------------------------------------------------------------
    // Scoring functions
    // -----------------------------------------------------------------------

    #[test]
    fn density_score_perfect() {
        // When num_ticks == target, score should be 1.0.
        let s = density_score(6.0, 6.0, 10.0);
        assert!((s - 1.0).abs() < 1e-10, "perfect density score should be 1.0, got {}", s);
    }

    #[test]
    fn density_score_degrades() {
        let s6 = density_score(6.0, 6.0, 10.0);
        let s12 = density_score(12.0, 6.0, 10.0);
        assert!(s6 > s12, "density should degrade as tick count diverges from target");
    }

    #[test]
    fn coverage_score_perfect() {
        let s = coverage_score(0.0, 10.0, 0.0, 10.0);
        assert!(
            (s - 1.0).abs() < 1e-10,
            "perfect coverage should be 1.0, got {}",
            s
        );
    }

    #[test]
    fn coverage_score_overshoot() {
        let s_tight = coverage_score(0.0, 10.0, 0.0, 10.0);
        let s_wide = coverage_score(-5.0, 15.0, 0.0, 10.0);
        assert!(
            s_tight > s_wide,
            "tighter coverage should score higher: {} vs {}",
            s_tight,
            s_wide
        );
    }

    #[test]
    fn simplicity_prefers_earlier_q() {
        let ticks_with_zero = vec![0.0, 1.0, 2.0];
        let s0 = simplicity_score(0, &ticks_with_zero); // q=1
        let s2 = simplicity_score(2, &ticks_with_zero); // q=2
        assert!(s0 > s2, "q=1 should score higher on simplicity than q=2");
    }

    // -----------------------------------------------------------------------
    // Large-range stress test
    // -----------------------------------------------------------------------

    #[test]
    fn large_range_no_panic() {
        let ticks = generate_ticks(0.0, 1e12, 6, &Scale::Linear);
        assert_nice(&ticks);
        assert!(!ticks.is_empty());
    }

    #[test]
    fn tiny_range_no_panic() {
        let ticks = generate_ticks(1e-10, 2e-10, 6, &Scale::Linear);
        assert_nice(&ticks);
        assert!(!ticks.is_empty());
    }

    // -----------------------------------------------------------------------
    // Negative range
    // -----------------------------------------------------------------------

    #[test]
    fn negative_range() {
        let ticks = generate_ticks(-100.0, -10.0, 6, &Scale::Linear);
        assert_nice(&ticks);
        assert_covers(&ticks, -100.0, -10.0);
        for t in &ticks {
            assert!(t.value <= 0.0, "ticks for negative range should be non-positive: {}", t.value);
        }
    }

    // -----------------------------------------------------------------------
    // TickSet -> Vec<Tick> conversion
    // -----------------------------------------------------------------------

    #[test]
    fn tick_set_into_ticks() {
        let ts = make_tick_set(vec![0.0, 5.0, 10.0]);
        let ticks = ts.into_ticks();
        assert_eq!(ticks.len(), 3);
        assert_eq!(ticks[0].value, 0.0);
        assert_eq!(ticks[0].label, "0");
        assert_eq!(ticks[1].value, 5.0);
        assert_eq!(ticks[1].label, "5");
        assert_eq!(ticks[2].value, 10.0);
        assert_eq!(ticks[2].label, "10");
    }

    // -----------------------------------------------------------------------
    // Log10 tick generation
    // -----------------------------------------------------------------------

    #[test]
    fn log_ticks_powers_of_10() {
        // Range spanning 4 decades: should produce ticks at powers of 10.
        let ticks = generate_ticks(1.0, 10_000.0, 7, &Scale::Log10);
        assert_nice(&ticks);
        let pos = positions(&ticks);
        // Must include 1, 10, 100, 1000, 10000 (or at least the endpoints).
        assert!(pos.contains(&1.0), "should include 10^0 = 1: {:?}", pos);
        assert!(pos.contains(&10.0), "should include 10^1 = 10: {:?}", pos);
        assert!(pos.contains(&100.0), "should include 10^2 = 100: {:?}", pos);
        assert!(pos.contains(&1000.0), "should include 10^3 = 1000: {:?}", pos);
        assert!(pos.contains(&10000.0), "should include 10^4 = 10000: {:?}", pos);
    }

    #[test]
    fn log_ticks_all_positive() {
        let ticks = generate_ticks(0.01, 1_000_000.0, 7, &Scale::Log10);
        for t in &ticks {
            assert!(t.value > 0.0, "log tick must be positive, got {}", t.value);
        }
    }

    #[test]
    fn log_ticks_large_range() {
        // 10 decades.
        let ticks = generate_ticks(1e-5, 1e5, 7, &Scale::Log10);
        assert_nice(&ticks);
        assert!(ticks.len() >= 3, "should have at least 3 ticks: {:?}", positions(&ticks));
    }

    #[test]
    fn log_ticks_small_values() {
        let ticks = generate_ticks(0.001, 0.1, 5, &Scale::Log10);
        assert!(!ticks.is_empty());
        for t in &ticks {
            assert!(t.value > 0.0);
        }
    }

    // -----------------------------------------------------------------------
    // Log minor ticks
    // -----------------------------------------------------------------------

    #[test]
    fn log_minor_ticks_basic() {
        let minor = generate_log_minor_ticks(1.0, 100.0);
        // Between 1 and 10: should have 2,3,4,5,6,7,8,9
        // Between 10 and 100: should have 20,30,40,50,60,70,80,90
        assert!(!minor.is_empty());
        assert!(minor.contains(&2.0), "should include 2: {:?}", minor);
        assert!(minor.contains(&5.0), "should include 5: {:?}", minor);
        assert!(minor.contains(&9.0), "should include 9: {:?}", minor);
        assert!(minor.contains(&20.0), "should include 20: {:?}", minor);
        assert!(minor.contains(&50.0), "should include 50: {:?}", minor);
        assert!(minor.contains(&90.0), "should include 90: {:?}", minor);
    }

    #[test]
    fn log_minor_ticks_all_positive() {
        let minor = generate_log_minor_ticks(0.01, 1000.0);
        for &v in &minor {
            assert!(v > 0.0, "minor tick must be positive, got {}", v);
        }
    }

    #[test]
    fn log_minor_ticks_sorted() {
        let minor = generate_log_minor_ticks(1.0, 10000.0);
        for w in minor.windows(2) {
            assert!(w[1] >= w[0], "minor ticks not sorted: {} before {}", w[0], w[1]);
        }
    }

    // -----------------------------------------------------------------------
    // SymLog tick generation (dedicated)
    // -----------------------------------------------------------------------

    #[test]
    fn symlog_ticks_include_zero_dedicated() {
        let ticks = generate_ticks(-100.0, 100.0, 7, &Scale::SymLog { linthresh: 1.0 });
        let pos = positions(&ticks);
        assert!(pos.contains(&0.0), "symlog ticks should include zero: {:?}", pos);
    }

    #[test]
    fn symlog_ticks_include_linthresh() {
        let ticks = generate_ticks(-1000.0, 1000.0, 7, &Scale::SymLog { linthresh: 10.0 });
        let pos = positions(&ticks);
        assert!(pos.contains(&10.0), "should include +linthresh=10: {:?}", pos);
        assert!(pos.contains(&-10.0), "should include -linthresh=-10: {:?}", pos);
    }

    #[test]
    fn symlog_ticks_sorted_dedicated() {
        let ticks = generate_ticks(-1000.0, 1000.0, 7, &Scale::SymLog { linthresh: 1.0 });
        assert_nice(&ticks);
    }

    #[test]
    fn symlog_ticks_positive_only() {
        let ticks = generate_ticks(0.1, 10000.0, 7, &Scale::SymLog { linthresh: 1.0 });
        assert!(!ticks.is_empty());
        assert_nice(&ticks);
    }

    #[test]
    fn symlog_ticks_negative_only() {
        let ticks = generate_ticks(-10000.0, -0.1, 7, &Scale::SymLog { linthresh: 1.0 });
        assert!(!ticks.is_empty());
        assert_nice(&ticks);
    }

    #[test]
    fn symlog_ticks_degenerate() {
        // Entirely within linear region.
        let ticks = generate_ticks(-0.5, 0.5, 5, &Scale::SymLog { linthresh: 1.0 });
        assert!(!ticks.is_empty());
    }
}
