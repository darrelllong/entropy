//! DIEHARDER test 203 — rgb_lagged_sums.
//!
//! Sums samples drawn with a configurable lag between each.  If the generator
//! has autocorrelation at distance `lag`, the sums will deviate from the
//! normal distribution expected for independent samples.
//!
//! # Author
//! Robert G. Brown, *Dieharder* (2006), test `rgb_lagged_sums`.

use crate::{math::{ks_test, normal_cdf}, result::TestResult};

/// Number of values summed per window.
const WINDOW: usize = 1_000;

/// Run the lagged sums test with the given lag.
///
/// Each sum uses WINDOW values spaced `lag` apart, starting at non-overlapping
/// offsets so that consecutive sums are independent.  The starting offsets are
/// 0, WINDOW×lag, 2×WINDOW×lag, … ensuring no float is shared between sums.
///
/// # Author
/// Robert G. Brown, Dieharder (2006), `rgb_lagged_sums`.
pub fn lagged_sums(words: &[u32], lag: usize) -> TestResult {
    let lag = lag.max(1);
    // Each non-overlapping sum consumes WINDOW × lag floats.
    let stride = WINDOW * lag;
    let n_sums = words.len() / stride;

    if n_sums < 20 {
        return TestResult::insufficient("dieharder::lagged_sums", "not enough words");
    }

    let floats: Vec<f64> = words.iter().map(|&w| w as f64 / 4_294_967_296.0).collect();

    let mean = WINDOW as f64 / 2.0;
    let std_dev = (WINDOW as f64 / 12.0_f64).sqrt();

    let mut p_values: Vec<f64> = (0..n_sums)
        .map(|i| {
            // Sum WINDOW values at spacing `lag`, starting at non-overlapping offset.
            let base = i * stride;
            let sum: f64 = (0..WINDOW).map(|j| floats[base + j * lag]).sum();
            let z = (sum - mean) / std_dev;
            normal_cdf(z).clamp(1e-15, 1.0 - 1e-15)
        })
        .collect();

    let p_value = ks_test(&mut p_values);

    TestResult::with_note(
        "dieharder::lagged_sums",
        p_value,
        format!("lag={lag}, window={WINDOW}, n_sums={n_sums}"),
    )
}
