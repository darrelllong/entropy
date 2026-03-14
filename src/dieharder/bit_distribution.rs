//! DIEHARDER test 200 — rgb_bitdist.
//!
//! For each n-bit word width (n = 1..=N), tests whether all 2ⁿ patterns
//! appear with equal frequency.  This is a generalisation of the monobit
//! test to wider patterns and is the most sensitive single test in DIEHARDER
//! for detecting bias — Brown reports that all known weak generators fail at
//! n ≤ 6 bits.
//!
//! # Author
//! Robert G. Brown, *Dieharder* (2006), test `rgb_bitdist`.

use crate::{math::igamc, result::TestResult};

/// Run the bit-distribution test for all word widths 1..=`max_bits`.
///
/// Returns a single `TestResult` whose p-value is the minimum (worst) over
/// all widths; the note details every individual p-value.
///
/// # Author
/// Robert G. Brown, Dieharder (2006), `rgb_bitdist`.
pub fn bit_distribution(words: &[u32], max_bits: usize) -> TestResult {
    let mut worst_p = 1.0f64;
    let mut notes = Vec::new();

    for n in 1..=max_bits.min(20) {
        if let Some(p) = test_width(words, n) {
            notes.push(format!("n={n} p={p:.4}"));
            if p < worst_p { worst_p = p; }
        }
    }

    TestResult::with_note(
        "dieharder::bit_distribution",
        worst_p,
        notes.join("; "),
    )
}

fn test_width(words: &[u32], n: usize) -> Option<f64> {
    let table_size = 1usize << n;
    let mask = (table_size - 1) as u32;

    // Extract all n-bit patterns (non-overlapping within each 32-bit word).
    let samples_per_word = 32 / n;
    let n_samples = words.len() * samples_per_word;

    if n_samples < 5 * table_size {
        return None; // not enough samples for a reliable chi-square
    }

    let mut counts = vec![0u32; table_size];
    for &w in words {
        for k in 0..samples_per_word {
            let pattern = ((w >> (k * n)) & mask) as usize;
            counts[pattern] += 1;
        }
    }

    let expected = n_samples as f64 / table_size as f64;
    let chi_sq: f64 = counts.iter().map(|&c| (c as f64 - expected).powi(2) / expected).sum();
    let df = table_size - 1;

    Some(igamc(df as f64 / 2.0, chi_sq / 2.0))
}
