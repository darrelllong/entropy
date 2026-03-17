//! DIEHARDER test 17 — marsaglia_tsang_gcd.
//!
//! Tests two signals from applying the Euclidean algorithm to pairs of random
//! 32-bit integers:
//!
//! 1. **GCD distribution**: P(gcd = k) = 6/(π²k²).  Tested with chi-square.
//! 2. **Step-count distribution**: the number of Euclidean steps follows a
//!    per-step distribution tabulated via `kprob[]` in the C source (41 bins,
//!    k=0..40; values ≥41 lumped into bin 40).  Tested with chi-square.
//!
//! The kprob[] table is the authentic empirical table from the published
//! Dieharder source (`marsaglia_tsang_gcd.c`), built from ~10^11 samples with
//! four independent high-quality RNGs (mt19937_1999, ranlxd2, gfsr4, taus2).
//! Bins with expected count < 5 are excluded from the chi-square (matching
//! Vtest_eval cutoff = 5.0 in the C source).
//!
//! # Author
//! George Marsaglia and Wai Wan Tsang, "Some Difficult-to-pass Tests of
//! Randomness", *Journal of Statistical Software* 7(3), 2002.
//! <https://doi.org/10.18637/jss.v007.i03>
//! Source: `dieharder-3.31.1/libdieharder/marsaglia_tsang_gcd.c`

use crate::{math::igamc, result::TestResult, rng::Rng};
use std::f64::consts::PI;

const N_PAIRS: usize = 100_000;

/// Size of the step-count table (k = 0..KTBLSIZE-1; k ≥ KTBLSIZE-1 lumped).
const KTBLSIZE: usize = 41;

/// Authentic kprob[] table from `marsaglia_tsang_gcd.c`.
/// Built from ~10^11 samples with mt19937_1999, ranlxd2, gfsr4, and taus2.
/// Index k = number of Euclidean steps; k ≥ 40 lumped into bin 40.
///
/// Source: `dieharder-3.31.1/libdieharder/marsaglia_tsang_gcd.c`, `kprob[KTBLSIZE]`.
#[rustfmt::skip]
const KPROB: [f64; KTBLSIZE] = [
    0.0,          5.39e-09,     6.077e-08,    4.8421e-07,   2.94869e-06,  1.443266e-05,
    5.908569e-05, 2.0658047e-04, 6.2764766e-04, 1.67993762e-03, 3.99620143e-03, 8.51629626e-03,
    1.635214339e-02, 2.843154488e-02, 4.493723812e-02, 6.476525706e-02, 8.533638862e-02, 1.030000214e-01,
    1.1407058851e-01, 1.1604146948e-01, 1.0853040184e-01, 9.336837411e-02, 7.389607162e-02, 5.380182248e-02,
    3.601960159e-02, 2.215902902e-02, 1.251328472e-02, 6.47884418e-03, 3.06981507e-03, 1.32828179e-03,
    5.2381841e-04, 1.8764452e-04, 6.084138e-05, 1.779885e-05, 4.66795e-06, 1.09504e-06,
    2.2668e-07,   4.104e-08,    6.42e-09,     8.4e-10,      1.4e-10,
];

/// Run both GCD tests (distribution and step counts); returns two `TestResult`s.
pub fn gcd_both(rng: &mut impl Rng) -> Vec<TestResult> {
    let gnorm = 6.0 / (PI * PI);

    // Dynamic GCD table size: gtblsize = sqrt(N_PAIRS * gnorm / 100).
    // Matches C: `gtblsize = sqrt((double)tsamples * gnorm / 100.0)`.
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

    // --- GCD chi-square ---
    // C: bins 0 and 1 are explicitly zeroed (set to 0.0/0.0) and excluded.
    // Expected for bin i (i ≥ 2): n * gnorm / i².
    // Tail bin (gtblsize-1): sum of n * gnorm / j² for j = gtblsize-1..100000.
    let gcd_expected: Vec<f64> = (0..gtblsize)
        .map(|i| {
            if i < 2 {
                0.0
            } else if i == gtblsize - 1 {
                // Tail: accumulate 6/(π²j²) for j from gtblsize-1 to 100000.
                (i..=100_000)
                    .map(|j| n * gnorm / (j as f64 * j as f64))
                    .sum()
            } else {
                n * gnorm / (i as f64 * i as f64)
            }
        })
        .collect();

    let gcd_chi_sq: f64 = gcd_counts
        .iter()
        .zip(gcd_expected.iter())
        .filter(|(_, &exp)| exp >= 5.0)
        .map(|(&obs, &exp)| (obs as f64 - exp).powi(2) / exp)
        .sum();
    let gcd_df = gcd_counts
        .iter()
        .zip(gcd_expected.iter())
        .filter(|(_, &exp)| exp >= 5.0)
        .count()
        .saturating_sub(1);
    let p_gcd = igamc(gcd_df as f64 / 2.0, gcd_chi_sq / 2.0);

    // --- Step-count chi-square ---
    // Uses KPROB[] table from published C source; bins with expected < 5.0 excluded.
    let step_chi_sq: f64 = step_counts
        .iter()
        .zip(KPROB.iter())
        .filter(|(_, &p)| p * n >= 5.0)
        .map(|(&obs, &p)| {
            let exp = p * n;
            (obs as f64 - exp).powi(2) / exp
        })
        .sum();
    let step_df = step_counts
        .iter()
        .zip(KPROB.iter())
        .filter(|(_, &p)| p * n >= 5.0)
        .count()
        .saturating_sub(1);
    let p_steps = igamc(step_df as f64 / 2.0, step_chi_sq / 2.0);

    vec![
        TestResult::with_note(
            "dieharder::gcd_distribution",
            p_gcd,
            format!("pairs={actual_pairs}, gtblsize={gtblsize}, χ²={gcd_chi_sq:.4}"),
        ),
        TestResult::with_note(
            "dieharder::gcd_step_counts",
            p_steps,
            format!("pairs={actual_pairs}, χ²={step_chi_sq:.4}"),
        ),
    ]
}

/// Run only the GCD distribution chi-square (backward-compatible single result).
pub fn gcd(rng: &mut impl Rng) -> TestResult {
    gcd_both(rng).remove(0)
}

/// Compute gcd(a, b) using the Euclidean algorithm; also return the step count.
///
/// Matches the C loop: `do { w = u%v; u = v; v = w; k++; } while(v>0)`.
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
