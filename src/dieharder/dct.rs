//! DIEHARDER test 206 — dab_dct.
//!
//! Performs a Type-II Discrete Cosine Transform on blocks of raw 32-bit words
//! from the RNG.  For each block, the position of the maximum absolute DCT value
//! is recorded; a chi-square test checks that this position is uniformly
//! distributed over all block positions (primary method: tsamples > 5 × ntuple).
//!
//! Algorithm from `dieharder-3.31.1/libdieharder/dab_dct.c`:
//!   1. Rotate raw u32 words (rotAmount increases by rmax_bits/4 each quarter).
//!   2. DCT-II: X[k] = Σ x[j] cos(π(j+½)k/N)  (direct O(n²), n=256).
//!   3. Adjust DC component: X[0] -= N·(2³¹−½); X[0] /= √2.
//!   4. Record argmax |X[k]|.
//!   Chi-square on position counts; expected = tsamples/ntuple per position.
//!
//! # Author
//! David Bauer, *Dieharder* (2006), test `dab_dct`.
//! Source: `dieharder-3.31.1/libdieharder/dab_dct.c`

use crate::{math::igamc, result::TestResult};
use std::f64::consts::PI;

/// Block length (ntuple), must be a power of 2.
const NTUPLE: usize = 256;
/// Number of blocks (tsamples).  Must be > 5 × NTUPLE for the primary method.
const TSAMPLES: usize = 5_000;
/// Bit width of each generator word (rmax_bits = 32 for u32 output).
const RMAX_BITS: u32 = 32;

/// Run the DCT spectral test.
///
/// # Author
/// David Bauer, Dieharder (2006), `dab_dct`.
pub fn dct(words: &[u32]) -> TestResult {
    let needed = TSAMPLES * NTUPLE;
    if words.len() < needed {
        return TestResult::insufficient("dieharder::dct", "not enough words");
    }

    // v = 2^(rmax_bits−1).  DC mean for a block of N uniform u32 values is
    // N·(v − 0.5) since E[U32] ≈ 2^31 − 0.5.
    let v = 1u64 << (RMAX_BITS - 1);
    let mean_dc = NTUPLE as f64 * (v as f64 - 0.5);

    // positionCounts[k] counts how many blocks had position k as the |DCT| argmax.
    let mut position_counts = vec![0u64; NTUPLE];

    for j in 0..TSAMPLES {
        // rotAmount increases by rmax_bits/4 every TSAMPLES/4 blocks,
        // matching `if j != 0 && j % (tsamples/4) == 0 { rotAmount += rmax_bits/4; }`.
        let rot_amount = ((j / (TSAMPLES / 4)) as u32 * (RMAX_BITS / 4)) % RMAX_BITS;

        let block = &words[j * NTUPLE..(j + 1) * NTUPLE];

        // Compute DCT-II of the rotated raw words (as unsigned integers).
        let mut dct_vals = dct_ii_u32(block, rot_amount);

        // Adjust DC component: subtract block mean, then divide by √2.
        dct_vals[0] -= mean_dc;
        dct_vals[0] /= 2f64.sqrt();

        // Record the position of the maximum absolute DCT value.
        let max_pos = dct_vals
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        position_counts[max_pos] += 1;
    }

    // Chi-square for uniformity of position counts.
    // Expected count per position = TSAMPLES / NTUPLE.
    let expected = TSAMPLES as f64 / NTUPLE as f64;
    let chi_sq: f64 = position_counts
        .iter()
        .map(|&c| (c as f64 - expected).powi(2) / expected)
        .sum();
    let df = NTUPLE - 1;

    let p_value = igamc(df as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(
        "dieharder::dct",
        p_value,
        format!("ntuple={NTUPLE}, tsamples={TSAMPLES}, χ²={chi_sq:.4}"),
    )
}

/// Direct O(n²) DCT-II of raw u32 words with bit-rotation.
///
/// X[k] = Σ_{j=0}^{N-1} x[j] · cos(π(j+½)k/N)
///
/// where x[j] is the rotated word cast to f64.  Matches `fDCT2` in dab_dct.c.
fn dct_ii_u32(words: &[u32], rot_amount: u32) -> Vec<f64> {
    let n = words.len();
    let scale = PI / n as f64;

    let x: Vec<f64> = words
        .iter()
        .map(|&w| {
            let rotated = if rot_amount == 0 { w } else { w.rotate_left(rot_amount) };
            rotated as f64
        })
        .collect();

    (0..n)
        .map(|k| {
            x.iter()
                .enumerate()
                .map(|(j, &xj)| xj * ((j as f64 + 0.5) * k as f64 * scale).cos())
                .sum()
        })
        .collect()
}
