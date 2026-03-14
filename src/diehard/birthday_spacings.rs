//! DIEHARD Test 1 — Birthday Spacings Test.
//!
//! Chooses m = 512 "birthdays" from a year of n = 2²⁴ days (using 24-bit
//! subfields of each 32-bit word).  The number of collisions j in the sorted
//! spacings is Poisson(λ = m³/(4n)) = Poisson(2).  Repeats 500 times for
//! each of 9 bit-offset positions; the 9 p-values are tested with a final KS
//! test.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::ks_test, result::TestResult};

const M: usize = 512;       // birthdays per trial
const YEAR: u32 = 1 << 24;  // 2^24 days
const SAMPLES: usize = 500; // trials per bit-offset

/// Run the birthday spacings test.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn birthday_spacings(words: &[u32]) -> TestResult {
    if words.len() < 9 * SAMPLES * M {
        return TestResult::insufficient(
            "diehard::birthday_spacings",
            "not enough words",
        );
    }

    let mut p_values: Vec<f64> = Vec::with_capacity(9);
    let mut word_iter = words.iter().copied();

    for offset in 0..9usize {
        let mut counts = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            let mut birthdays: Vec<u32> = (0..M)
                .map(|_| (word_iter.next().unwrap_or(0) >> offset) & (YEAR - 1))
                .collect();
            birthdays.sort_unstable();

            // Count spacings that appear more than once.
            let spacings: Vec<u32> = birthdays.windows(2).map(|w| w[1] - w[0]).collect();
            let mut sorted_spacings = spacings.clone();
            sorted_spacings.sort_unstable();
            let collisions = sorted_spacings.windows(2).filter(|w| w[0] == w[1]).count();
            counts.push(collisions);
        }

        // The j's should be Poisson(2); convert to p-values via Poisson CDF.
        let mut pvals: Vec<f64> = counts
            .iter()
            .map(|&j| poisson_cdf(j, 2.0))
            .collect();
        p_values.push(ks_test(&mut pvals));
    }

    // Final Kolmogorov-Smirnov test on the 9 p-values from bit offsets.
    let p_value = ks_test(&mut p_values);

    TestResult::with_note(
        "diehard::birthday_spacings",
        p_value,
        format!("m={M}, year=2^24, samples={SAMPLES}"),
    )
}

/// Poisson CDF P(X ≤ k) for X ~ Poisson(lambda), using the exact formula.
fn poisson_cdf(k: usize, lambda: f64) -> f64 {
    let mut sum = 0.0f64;
    let mut term = (-lambda).exp();
    sum += term;
    for i in 1..=k {
        term *= lambda / i as f64;
        sum += term;
    }
    sum.min(1.0)
}
