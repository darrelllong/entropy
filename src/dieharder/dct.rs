//! DIEHARDER test 206 — dab_dct.
//!
//! Applies the Discrete Cosine Transform (Type II) to blocks of the output and
//! tests the resulting spectral coefficients for uniformity.  Detects periodic
//! structure and spectral non-uniformity that is invisible to time-domain tests.
//! Bauer ranks this the second-best test in DIEHARDER for detecting bias.
//!
//! # Author
//! David Bauer, *Dieharder* (2006), test `dab_dct`.

use crate::{math::igamc, result::TestResult};
use std::f64::consts::PI;

const BLOCK_SIZE: usize = 512;
const N_BLOCKS: usize = 1_000;

/// Run the DCT spectral test.
///
/// # Author
/// David Bauer, Dieharder (2006), `dab_dct`.
pub fn dct(words: &[u32]) -> TestResult {
    let needed = N_BLOCKS * BLOCK_SIZE / 4 + 1;
    if words.len() < needed {
        return TestResult::insufficient("dieharder::dct", "not enough words");
    }

    // Convert to bytes then to floats in [−1, 1).
    let bytes: Vec<f64> = words
        .iter()
        .flat_map(|&w| w.to_le_bytes())
        .take(N_BLOCKS * BLOCK_SIZE)
        .map(|b| b as f64 / 128.0 - 1.0)
        .collect();

    // For each block, compute DCT-II, collect all AC coefficients (skip DC),
    // and chi-square test them against N(0, σ²) with σ² = BLOCK_SIZE/2.
    let mut all_coefs: Vec<f64> = Vec::with_capacity(N_BLOCKS * (BLOCK_SIZE - 1));

    for block_idx in 0..N_BLOCKS {
        let block = &bytes[block_idx * BLOCK_SIZE..(block_idx + 1) * BLOCK_SIZE];
        let coefs = dct_ii(block);
        // Skip DC component (index 0); use AC components.
        all_coefs.extend_from_slice(&coefs[1..]);
    }

    // Test: the AC coefficients of an i.i.d. uniform input should be normal
    // with zero mean.  For bytes mapped to (b/128 - 1), Var(x) ≈ 1/3.
    // For DCT-II (unnormalized), Var(X[k]) = Var(x) × Σ cos²(…) = Var(x) × N/2
    // = (1/3) × (BLOCK_SIZE / 2) = BLOCK_SIZE / 6.
    let sigma = (BLOCK_SIZE as f64 / 6.0).sqrt();
    let n_bins = 20usize;
    let bin_width = 6.0 * sigma / n_bins as f64;
    let lower = -3.0 * sigma;

    // Bin layout: counts[0] = left tail (c < lower), counts[1..=n_bins] = interior
    // bins, counts[n_bins+1] = right tail (c ≥ lower + n_bins×bin_width).
    // The interior index starts at 1, not 0, to keep the left tail in counts[0].
    let mut counts = vec![0u32; n_bins + 2]; // +2 for tails
    for &c in &all_coefs {
        let idx = if c < lower {
            0
        } else {
            let i = ((c - lower) / bin_width) as usize + 1;
            i.min(n_bins + 1)
        };
        counts[idx] += 1;
    }

    let total = all_coefs.len() as f64;
    let expected: Vec<f64> = {
        let mut e = vec![0.0f64; n_bins + 2];
        // Normal PDF integrated over each bin.
        use crate::math::normal_cdf;
        e[0] = normal_cdf(lower / sigma) * total; // left tail
        for i in 0..n_bins {
            let lo = lower + i as f64 * bin_width;
            let hi = lo + bin_width;
            e[i + 1] = (normal_cdf(hi / sigma) - normal_cdf(lo / sigma)) * total;
        }
        e[n_bins + 1] = (1.0 - normal_cdf(3.0)) * total; // right tail
        e
    };

    let chi_sq: f64 = counts
        .iter()
        .zip(expected.iter())
        .filter(|(_, &e)| e >= 5.0)
        .map(|(&c, &e)| (c as f64 - e).powi(2) / e)
        .sum();
    let df = counts.iter().zip(expected.iter()).filter(|(_, &e)| e >= 5.0).count() - 1;

    let p_value = igamc(df as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(
        "dieharder::dct",
        p_value,
        format!("block={BLOCK_SIZE}, N={N_BLOCKS}, χ²={chi_sq:.4}"),
    )
}

/// DCT-II of a real sequence: X[k] = Σ_{n=0}^{N−1} x[n] cos(π(n+½)k/N).
fn dct_ii(x: &[f64]) -> Vec<f64> {
    let n = x.len();
    let scale = PI / n as f64;
    (0..n)
        .map(|k| {
            x.iter()
                .enumerate()
                .map(|(j, &xj)| xj * ((j as f64 + 0.5) * k as f64 * scale).cos())
                .sum()
        })
        .collect()
}
