//! DIEHARDER GCD test.
//!
//! Applies Euclid's algorithm to pairs of 32-bit words, discarding pairs
//! with a zero, and scores two statistics:
//!
//! 1. The GCD: P(gcd = k) → 6/(π²k²) for large words.  The counts of
//!    k = 1 … K − 2, with larger GCDs in the last cell, are scored with a
//!    Pearson χ², K growing as √(pairs).
//! 2. The step count k of `while v ≠ 0 { (u, v) = (v, u mod v) }`, scored with
//!    a Pearson χ² against [`STEP_PROBABILITIES`].
//!
//! In each χ² the tail cells are pooled until every cell expects at least 5
//! pairs, so every pair is scored once.
//!
//! # The step-count law
//!
//! No closed form is known for 32-bit words.  [`STEP_PROBABILITIES`] is an
//! estimate from `examples/gcd_step_table.rs`: 5·10¹¹ pairs from PCG64 and
//! 5·10¹¹ from xoshiro256**, two unrelated generator designs whose counts
//! agree within sampling error (χ² = 42.0 on 37 degrees of freedom).  At that size the standard error of every
//! probability used at the test's 100 000 pairs is a negligible fraction of
//! the test's own sampling noise.
//!
//! # Calibration
//!
//! 20 000 separately seeded PCG64 streams gave p < 0.01 in 1.095% of the GCD
//! results and 1.085% of the step-count results (binomial standard deviation
//! 0.070%), and Kolmogorov–Smirnov p-values against uniformity of 0.65 and
//! 0.59.
//!
//! # Author
//! George Marsaglia and Wai Wan Tsang, "Some Difficult-to-pass Tests of
//! Randomness", *Journal of Statistical Software* 7(3), 2002,
//! <https://doi.org/10.18637/jss.v007.i03>.

use crate::{
    math::{chi_square_pooled_tails, igamc},
    result::TestResult,
    rng::Rng,
};
use std::f64::consts::PI;

/// Pairs drawn.
const N_PAIRS: usize = 100_000;

/// Size of the step-count table (k = 0..KTBLSIZE-1; k ≥ KTBLSIZE-1 lumped).
const KTBLSIZE: usize = 41;

/// P(k steps) for independent uniform nonzero 32-bit words, k = 0 … 40,
/// with k ≥ 40 in the last entry (see the module documentation).
#[rustfmt::skip]
pub const STEP_PROBABILITIES: [f64; KTBLSIZE] = [
    0.0,
    5.286000002532e-09,
    6.056700002901e-08,
    4.844950002321e-07,
    2.946393001411e-06,
    1.445050000692e-05,
    5.906462702829e-05,
    2.065249450989e-04,
    6.276912573007e-04,
    1.679825103805e-03,
    3.996315067914e-03,
    8.515906561079e-03,
    1.635256293283e-02,
    2.843217635762e-02,
    4.493705163252e-02,
    6.476610257202e-02,
    8.533581001588e-02,
    1.030018326953e-01,
    1.140692901406e-01,
    1.160421623676e-01,
    1.085313084480e-01,
    9.336725869072e-02,
    7.389565731240e-02,
    5.380160704577e-02,
    3.601897808125e-02,
    2.215832767161e-02,
    1.251355227499e-02,
    6.478457274103e-03,
    3.069824236470e-03,
    1.328563627636e-03,
    5.238620362509e-04,
    1.876686260899e-04,
    6.085164002915e-05,
    1.779029600852e-05,
    4.665563002235e-06,
    1.089485000522e-06,
    2.259970001083e-07,
    4.074600001952e-08,
    6.459000003094e-09,
    8.510000004076e-10,
    1.200000000575e-10,
];

/// Run both GCD tests (distribution and step counts); returns two `TestResult`s.
pub fn gcd_both(rng: &mut impl Rng) -> Vec<TestResult> {
    let gnorm = 6.0 / (PI * PI);

    // GCD cells: √(pairs · 6/π² / 100), so the last scored single GCD still
    // expects about 100 pairs.
    let gtblsize = ((N_PAIRS as f64 * gnorm / 100.0).sqrt() as usize).max(3);
    let mut gcd_counts = vec![0u32; gtblsize];

    // Step-count bins: k = 0..40, values ≥ 40 lumped into bin 40.
    let mut step_counts = [0u32; KTBLSIZE];

    let mut actual_pairs = 0usize;

    for _ in 0..N_PAIRS {
        let u = rng.next_u32();
        let v = rng.next_u32();
        if u == 0 || v == 0 {
            continue;
        }
        actual_pairs += 1;
        let (g, k) = euclid_gcd_with_steps(u, v);

        // GCD bin: lump gcd ≥ gtblsize into bin gtblsize-1.
        let gcd_idx = (g as usize).min(gtblsize - 1);
        gcd_counts[gcd_idx] += 1;

        // Step-count bin: lump k ≥ KTBLSIZE-1 into bin KTBLSIZE-1.
        let step_idx = k.min(KTBLSIZE - 1);
        step_counts[step_idx] += 1;
    }

    let n = actual_pairs as f64;

    // GCD χ² over gcd = 1 … gtblsize − 2 and a last cell for every larger gcd:
    // cell k expects n·6/(π²k²) and the last cell the remainder, so every pair
    // is scored once.
    let head: Vec<f64> = (1..gtblsize - 1)
        .map(|k| n * gnorm / (k as f64 * k as f64))
        .collect();
    let mut gcd_expected = head.clone();
    gcd_expected.push(n - head.iter().sum::<f64>());
    let gcd_observed: Vec<f64> = gcd_counts[1..].iter().map(|&c| f64::from(c)).collect();
    let (gcd_chi_sq, gcd_df) =
        chi_square_pooled_tails(&gcd_observed, &gcd_expected, 5.0).unwrap_or((0.0, 0));
    let p_gcd = starved_or_pvalue(gcd_df, gcd_chi_sq);

    // Step-count χ² against the simulated law, the tails pooled until each
    // cell expects at least 5 pairs.
    let step_expected: Vec<f64> = STEP_PROBABILITIES.iter().map(|&p| p * n).collect();
    let step_observed: Vec<f64> = step_counts.iter().map(|&c| f64::from(c)).collect();
    let (step_chi_sq, step_df) =
        chi_square_pooled_tails(&step_observed, &step_expected, 5.0).unwrap_or((0.0, 0));
    let p_steps = starved_or_pvalue(step_df, step_chi_sq);

    // Note for a chi-square left without a degree of freedom.
    let starved = || {
        format!(
            "pairs={actual_pairs} of {N_PAIRS}: zero words left too few nonzero pairs for a chi-square"
        )
    };
    vec![
        TestResult::with_note(
            "dieharder::gcd_distribution",
            p_gcd,
            if gcd_df == 0 {
                starved()
            } else {
                format!("pairs={actual_pairs}, gtblsize={gtblsize}, χ²={gcd_chi_sq:.4}")
            },
        )
        .chi_square(gcd_chi_sq, gcd_df as f64),
        TestResult::with_note(
            "dieharder::gcd_step_counts",
            p_steps,
            if step_df == 0 {
                starved()
            } else {
                format!("pairs={actual_pairs}, χ²={step_chi_sq:.4}")
            },
        )
        .chi_square(step_chi_sq, step_df as f64),
    ]
}

/// P-value of a GCD-test chi-square, or 0 when zero words starved it.
///
/// A pair containing a zero word is discarded.  An honest generator loses a
/// pair that way with probability about 2/2³², and at `N_PAIRS` both tables
/// keep many scored cells, so a chi-square with no degree of freedom means
/// most words were zero: catastrophic evidence, reported as p = 0 rather than
/// as missing data.
fn starved_or_pvalue(df: usize, chi_sq: f64) -> f64 {
    if df == 0 {
        0.0
    } else {
        igamc(df as f64 / 2.0, chi_sq / 2.0)
    }
}

/// The GCD distribution χ² alone.
pub fn gcd(rng: &mut impl Rng) -> TestResult {
    gcd_both(rng).remove(0)
}

/// Compute gcd(a, b) using the Euclidean algorithm; also return the step count.
///
/// Counts the steps of `while b ≠ 0 { (a, b) = (b, a mod b) }`.
fn euclid_gcd_with_steps(mut a: u32, mut b: u32) -> (u32, usize) {
    let mut steps = 0usize;
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
        steps += 1;
    }
    (a, steps)
}

#[cfg(test)]
mod tests {
    use super::{gcd_both, STEP_PROBABILITIES};
    use crate::rng::ConstantRng;

    /// Every pair of an all-zero stream contains a zero word and is discarded,
    /// so neither chi-square keeps a degree of freedom.  That is a FAIL
    /// (p = 0), not missing data.
    #[test]
    fn all_zero_stream_fails() {
        for r in gcd_both(&mut ConstantRng::new(0)) {
            assert!(!r.skipped(), "{r}");
            assert_eq!(r.p_value, 0.0, "{r}");
            let note = r.note.as_deref().unwrap_or_default();
            assert!(note.contains("zero words"), "{note}");
        }
    }

    /// The table is a distribution, and k = 0 steps is impossible for
    /// nonzero words.
    #[test]
    fn step_probabilities_sum_to_one() {
        let sum: f64 = STEP_PROBABILITIES.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12, "sum = {sum}");
        assert_eq!(STEP_PROBABILITIES[0], 0.0);
    }

    /// u = v = 1 in every pair: gcd 1 after one step, both in cells neither
    /// chi-square scores, so every scored cell is empty.
    #[test]
    fn constant_nonzero_stream_fails() {
        for r in gcd_both(&mut ConstantRng::new(1)) {
            assert!(!r.skipped() && !r.passed(), "{r}");
        }
    }
}
