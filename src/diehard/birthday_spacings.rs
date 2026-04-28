//! DIEHARD Test 1 — Birthday Spacings Test.
//!
//! Chooses m = 512 "birthdays" from a year of n = 2²⁴ days.  The number j of
//! repeated spacings in each trial is asymptotically Poisson(λ = m³/(4n) = 2).
//! After 500 trials per bit offset, a chi-square test of the j histogram against
//! the Poisson(2) distribution gives one p-value.  Nine p-values (bit offsets
//! 0..=8) are combined with a final KS test.
//!
//! Faithful transcription of `diehard_birthdays.c` in Dieharder 3.31.1:
//!   - `intervals[0] = rand_uint[0]` (gap from day 0 to first birthday)
//!   - `intervals[m] = rand_uint[m] − rand_uint[m−1]`  (m = 1..M)
//!   - j counts interval values appearing more than once
//!   - chi-square on `js[]` histogram (tail bins with expected < 5 are excluded)
//!   - KS on the 9 chi-square p-values
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).
//! Source: `dieharder-3.31.1/libdieharder/diehard_birthdays.c`

use crate::{
    math::{igamc, ks_test},
    result::TestResult,
};

const M: usize = 512; // birthdays per trial (nms)
const YEAR: u32 = 1 << 24; // year size = 2^24 (nbits = 24)
const SAMPLES: usize = 500; // trials per bit-offset (tsamples)
const LAMBDA: f64 = 2.0; // m³/(4n) = 512³/(4×2^24) = 2

/// Run the birthday spacings test.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn birthday_spacings(words: &[u32]) -> TestResult {
    if words.len() < 9 * SAMPLES * M {
        return TestResult::insufficient("diehard::birthday_spacings", "not enough words");
    }

    // kmax: find the first k where expected count < 5, then add one extra slot.
    // Matches C code: `while binfreq > 5: kmax++; kmax++` starting from kmax=1.
    let kmax = {
        let mut k = 1usize;
        while (SAMPLES as f64) * poisson_pmf(k, LAMBDA) > 5.0 {
            k += 1;
        }
        k + 1
    };

    let mut p_values: Vec<f64> = Vec::with_capacity(9);
    let mut word_iter = words.iter().copied();

    for offset in 0..9usize {
        let mut js = vec![0u32; kmax];

        for _ in 0..SAMPLES {
            // Precondition (line 36) guarantees the iterator has enough words.
            let mut birthdays: Vec<u32> = (0..M)
                .map(|_| (word_iter.next().expect("birthday_spacings: word iterator exhausted (precondition failed)") >> offset) & (YEAR - 1))
                .collect();
            birthdays.sort_unstable();

            // Build M intervals (matching Marsaglia's diehard_birthdays.c):
            //   intervals[0] = rand_uint[0]
            //   intervals[m] = rand_uint[m] - rand_uint[m-1]  for m = 1..M
            let mut sorted_intervals: Vec<u32> = Vec::with_capacity(M);
            sorted_intervals.push(birthdays[0]);
            for w in birthdays.windows(2) {
                sorted_intervals.push(w[1] - w[0]);
            }
            sorted_intervals.sort_unstable();

            // Count j = number of distinct interval values that appear more than once.
            // In the reference C, an interval repeated 3 or 4 times still counts once.
            let mut k = 0usize;
            let mut m = 0usize;
            while m + 1 < sorted_intervals.len() {
                let mut mnext = m + 1;
                while mnext < sorted_intervals.len()
                    && sorted_intervals[m] == sorted_intervals[mnext]
                {
                    if mnext == m + 1 {
                        k += 1;
                    }
                    mnext += 1;
                }
                m = if mnext != m + 1 { mnext } else { m + 1 };
            }

            // Ignore j ≥ kmax (C: "BAD IDEA to bundle all the points from the tail")
            if k < kmax {
                js[k] += 1;
            }
        }

        // Chi-square against Poisson(λ=2); bins with expected < 5 are dropped.
        let chi_sq: f64 = js
            .iter()
            .enumerate()
            .filter(|&(k, _)| (SAMPLES as f64) * poisson_pmf(k, LAMBDA) >= 5.0)
            .map(|(k, &obs)| {
                let exp = (SAMPLES as f64) * poisson_pmf(k, LAMBDA);
                (obs as f64 - exp).powi(2) / exp
            })
            .sum();
        let df = js
            .iter()
            .enumerate()
            .filter(|&(k, _)| (SAMPLES as f64) * poisson_pmf(k, LAMBDA) >= 5.0)
            .count()
            .saturating_sub(1);

        let p = igamc(df as f64 / 2.0, chi_sq / 2.0);
        p_values.push(p);
    }

    // Final KS test on the 9 chi-square p-values.
    let p_value = ks_test(&mut p_values);

    TestResult::with_note(
        "diehard::birthday_spacings",
        p_value,
        format!("m={M}, year=2^24, samples={SAMPLES}"),
    )
}

/// Poisson PMF: P(X = k) for X ~ Poisson(lambda).
fn poisson_pmf(k: usize, lambda: f64) -> f64 {
    let mut term = (-lambda).exp();
    for i in 1..=k {
        term *= lambda / i as f64;
    }
    term
}
