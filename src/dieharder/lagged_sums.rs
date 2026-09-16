//! DIEHARDER lagged sums test.
//!
//! Scales words to [0, 1) and sums every (lag + 1)-th one, skipping `lag`
//! words between samples.  For t independent uniforms the sum has mean t/2
//! and variance t/12, so z = (sum − t/2)/√(t/12) is approximately standard
//! normal, and the p-value is two-sided, erfc(|z|/√2).  A generator with
//! correlation at that lag moves the sum.
//!
//! # Author
//! Robert G. Brown, *Dieharder: A Random Number Test Suite* (2004–2011).

use crate::{math::erfc, result::TestResult};
use std::f64::consts::SQRT_2;

/// Run the lagged sums test with the given lag.
///
/// # Author
/// Robert G. Brown, Dieharder (2006).
pub fn lagged_sums(words: &[u32], lag: usize) -> TestResult {
    // lag = 0 sums every word.
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

#[cfg(test)]
mod tests {
    use super::lagged_sums;

    #[test]
    fn short_inputs_skip_and_constant_input_fails() {
        for lag in [0, 1, 100] {
            let needed = 1_000 * (lag + 1);
            assert!(lagged_sums(&[], lag).skipped());
            assert!(lagged_sums(&vec![0; needed - 1], lag).skipped());
            let r = lagged_sums(&vec![0; needed], lag);
            assert!(!r.skipped() && !r.passed(), "lag {lag}: {r}");
        }
    }
}
