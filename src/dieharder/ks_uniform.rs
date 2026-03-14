//! DIEHARDER test 204 — rgb_kstest_test.
//!
//! Applies the Kolmogorov-Smirnov test directly to the floating-point output
//! of the generator, testing that values are uniformly distributed on [0, 1).
//! This is distinct from the NIST frequency test, which tests bit-level bias;
//! the Kolmogorov-Smirnov test is sensitive to non-uniformity in the continuous distribution.
//!
//! # Author
//! Robert G. Brown, *Dieharder* (2006), test `rgb_kstest_test`.

use crate::{math::ks_test, result::TestResult};

const N_SAMPLES: usize = 100_000;
const REPEATS: usize = 100;

/// Apply the Kolmogorov-Smirnov test for uniformity to the generator's float output.
///
/// # Author
/// Robert G. Brown, Dieharder (2006), `rgb_kstest_test`.
pub fn ks_uniform(words: &[u32]) -> TestResult {
    if words.len() < REPEATS * N_SAMPLES {
        return TestResult::insufficient("dieharder::ks_uniform", "not enough words");
    }

    let floats: Vec<f64> = words.iter().map(|&w| w as f64 / 4_294_967_296.0).collect();

    let mut outer_pvals: Vec<f64> = (0..REPEATS)
        .map(|rep| {
            let mut sample: Vec<f64> =
                floats[rep * N_SAMPLES..(rep + 1) * N_SAMPLES].to_vec();
            ks_test(&mut sample)
        })
        .collect();

    let p_value = ks_test(&mut outer_pvals);

    TestResult::with_note(
        "dieharder::ks_uniform",
        p_value,
        format!("n={N_SAMPLES}, repeats={REPEATS}"),
    )
}
