//! DIEHARDER Kolmogorov–Smirnov uniformity test.
//!
//! Scales every word to [0, 1) and applies one Kolmogorov–Smirnov test of
//! uniformity to the whole sample.
//!
//! # Author
//! Robert G. Brown, *Dieharder: A Random Number Test Suite* (2004–2011).

use crate::{math::ks_test, result::TestResult};

/// Fewest words the test accepts; it uses every word it is given.
const MIN_SAMPLES: usize = 1_000;

/// Apply the Kolmogorov-Smirnov test for uniformity to the generator's float output.
///
/// # Author
/// Robert G. Brown, Dieharder (2006).
pub fn ks_uniform(words: &[u32]) -> TestResult {
    if words.len() < MIN_SAMPLES {
        return TestResult::insufficient("dieharder::ks_uniform", "not enough words");
    }

    let mut sample: Vec<f64> = words.iter().map(|&w| w as f64 / 4_294_967_296.0).collect();
    let p_value = ks_test(&mut sample);
    let d = crate::math::ks_statistic(&mut sample);

    TestResult::with_note(
        "dieharder::ks_uniform",
        p_value,
        format!("tsamples={}", words.len()),
    )
    .kolmogorov_smirnov(d, words.len())
}

#[cfg(test)]
mod tests {
    use super::{ks_uniform, MIN_SAMPLES};

    #[test]
    fn short_inputs_skip_and_constant_input_fails() {
        assert!(ks_uniform(&[]).skipped());
        assert!(ks_uniform(&vec![0; MIN_SAMPLES - 1]).skipped());
        let r = ks_uniform(&vec![0; MIN_SAMPLES]);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }
}
