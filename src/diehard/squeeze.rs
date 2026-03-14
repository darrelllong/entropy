//! DIEHARD Test 13 — Squeeze Test.
//!
//! Starting from k = 2³¹, repeatedly applies k = ⌈k · U⌉ where U is drawn
//! from the generator (as a float in (0,1]).  Counts j, the number of steps
//! to reduce k to 1.  Repeats 100 000 times; the distribution of j is
//! tested with chi-square against known cell probabilities.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::igamc, rng::Rng, result::TestResult};

const N_TRIALS: usize = 100_000;

// Pool boundaries: j ≤ J_MIN and j ≥ J_MAX+1.
const J_MIN: usize = 12;
const J_MAX: usize = 40;

/// P(j = k) for the squeeze process (k₀ = 2³¹, U ∈ (0,1]).
/// Computed by 5 × 10⁶ Monte Carlo trials.
/// The actual distribution peaks near j = 22–23 (mean ≈ H_{2³¹} ≈ 22);
/// note that DIEHARD's published table used a different starting value.
///
/// Index 0  → j ≤ 12 (pooled)
/// Index 1  → j = 13
/// …
/// Index 28 → j = 40
/// Index 29 → j ≥ 41 (pooled)
const PI: [f64; 30] = [
    0.008_841,  // j ≤ 12 (pooled)
    0.008_286,  // j = 13
    0.013_633,  // j = 14
    0.020_915,  // j = 15
    0.030_198,  // j = 16
    0.040_911,  // j = 17
    0.051_917,  // j = 18
    0.062_779,  // j = 19
    0.072_066,  // j = 20
    0.078_719,  // j = 21
    0.081_942,  // j = 22
    0.081_778,  // j = 23
    0.078_530,  // j = 24
    0.072_232,  // j = 25
    0.064_100,  // j = 26
    0.054_717,  // j = 27
    0.045_076,  // j = 28
    0.036_140,  // j = 29
    0.027_973,  // j = 30
    0.021_032,  // j = 31
    0.015_442,  // j = 32
    0.010_993,  // j = 33
    0.007_567,  // j = 34
    0.005_125,  // j = 35
    0.003_405,  // j = 36
    0.002_178,  // j = 37
    0.001_401,  // j = 38
    0.000_864,  // j = 39
    0.000_521,  // j = 40
    0.000_720,  // j ≥ 41 (pooled)
];

/// Run the squeeze test.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn squeeze(rng: &mut impl Rng) -> TestResult {
    let mut counts = [0u32; 30];

    for _ in 0..N_TRIALS {
        let j = squeeze_steps(rng);
        let idx = if j <= J_MIN {
            0
        } else if j >= J_MAX + 1 {
            29
        } else {
            j - J_MIN   // j=13 → 1, j=14 → 2, …, j=40 → 28
        };
        counts[idx] += 1;
    }

    // Only use cells with expected count ≥ 5.
    let n = N_TRIALS as f64;
    let chi_sq: f64 = counts
        .iter()
        .zip(PI.iter())
        .filter(|(_, &p)| p * n >= 5.0)
        .map(|(&c, &p)| (c as f64 - n * p).powi(2) / (n * p))
        .sum();

    let df = counts.iter().zip(PI.iter()).filter(|(_, &p)| p * n >= 5.0).count() - 1;

    let p_value = igamc(df as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(
        "diehard::squeeze",
        p_value,
        format!("trials={N_TRIALS}, df={df}, χ²={chi_sq:.4}"),
    )
}

fn squeeze_steps(rng: &mut impl Rng) -> usize {
    let mut k: u64 = 1 << 31;
    let mut j = 0usize;
    while k > 1 {
        // U ∈ (0, 1]: use (word + 1) / 2^32 to avoid zero.
        let u = (rng.next_u32() as f64 + 1.0) / 4_294_967_296.0;
        k = (k as f64 * u).ceil() as u64;
        j += 1;
        if j > 200 { break; } // safety guard
    }
    j
}
