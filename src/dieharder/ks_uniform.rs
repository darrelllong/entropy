//! DIEHARDER test 204 — rgb_kstest_test.
//!
//! Applies the Kolmogorov-Smirnov test directly to the floating-point output
//! of the generator, testing that values are uniformly distributed on [0, 1).
//! This implements the core `rgb_kstest_test.c` statistic: one KS test on one
//! vector of uniform deviates.
//!
//! # Author
//! Robert G. Brown, *Dieharder* (2006), test `rgb_kstest_test`.

use crate::{math::ks_test, result::TestResult};

/// Dieharder default is `tsamples = 1000`, but the core statistic is a single
/// KS test on one vector of uniform deviates. We use the full available word
/// stream here to keep the algorithm exact while avoiding a fake nested KS.
const MIN_SAMPLES: usize = 1_000;

/// Apply the Kolmogorov-Smirnov test for uniformity to the generator's float output.
///
/// # Author
/// Robert G. Brown, Dieharder (2006), `rgb_kstest_test`.
pub fn ks_uniform(words: &[u32]) -> TestResult {
    if words.len() < MIN_SAMPLES {
        return TestResult::insufficient("dieharder::ks_uniform", "not enough words");
    }

    let mut sample: Vec<f64> = words.iter().map(|&w| w as f64 / 4_294_967_296.0).collect();
    let p_value = ks_test(&mut sample);

    TestResult::with_note(
        "dieharder::ks_uniform",
        p_value,
        format!("tsamples={}", words.len()),
    )
}
