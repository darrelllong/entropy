//! DIEHARDER test 203 — rgb_lagged_sums.
//!
//! Sums samples drawn with a configurable lag between each. If the generator
//! has autocorrelation at distance `lag`, the sum will deviate from the
//! normal distribution expected for independent samples.
//!
//! # Author
//! Robert G. Brown, *Dieharder* (2006), test `rgb_lagged_sums`.

use crate::{math::erfc, result::TestResult};
use std::f64::consts::SQRT_2;

/// Run the lagged sums test with the given lag.
///
/// This implements the core `rgb_lagged_sums.c` statistic: one sum of samples
/// spaced `lag` apart, converted to a normal deviate. Dieharder then performs
/// an outer KS over many such p-values, but this crate reports the single-run
/// core statistic directly.
///
/// # Author
/// Robert G. Brown, Dieharder (2006), `rgb_lagged_sums`.
pub fn lagged_sums(words: &[u32], lag: usize) -> TestResult {
    let lag = lag.max(1);
    let stride = lag + 1;
    let tsamples = words.len() / stride;

    if tsamples < 1_000 {
        return TestResult::insufficient("dieharder::lagged_sums", "not enough words");
    }

    let sum: f64 = words
        .iter()
        .step_by(stride)
        .take(tsamples)
        .map(|&w| w as f64 / 4_294_967_296.0)
        .sum();

    let mean = tsamples as f64 / 2.0;
    let std_dev = (tsamples as f64 / 12.0_f64).sqrt();
    let z = (sum - mean) / std_dev;
    // Two-sided: a too-low sum (z << 0) is equally suspicious as a too-high one.
    let p_value = erfc(z.abs() / SQRT_2);

    TestResult::with_note(
        "dieharder::lagged_sums",
        p_value,
        format!("lag={lag}, tsamples={tsamples}, sum={sum:.4}, z={z:.4}"),
    )
}
