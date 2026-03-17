//! NIST SP 800-22 §2.8 — Overlapping Template Matching Test.
//!
//! Uses an all-ones template of length `m` (default m = 9).  Counts
//! overlapping occurrences in each of N = n/1032 blocks of M = 1032 bits,
//! bins the counts into 6 categories, and applies a chi-square test using
//! theoretical probabilities derived from a Markov chain model.
//!
//! Minimum recommended: n ≥ 10^6 (giving ≥ 968 blocks for m = 9, M = 1032).

use crate::{math::igamc, result::TestResult};

/// Run the overlapping template test.
///
/// The template is the all-ones pattern of length `m`.  Only `m = 9` with
/// `M = 1032` is supported: the theoretical Markov-chain probabilities in the
/// pi table (SP 800-22 §2.8.7, Table 4) are valid only for that pair.
/// Passing any other value of `m` returns an insufficient result.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.8.
pub fn overlapping_template(bits: &[u8], m: usize) -> TestResult {
    // The pi table below is derived from the Markov chain model for m = 9,
    // M = 1032 only (SP 800-22 §2.8.7, Table 4).  Using it for any other m
    // yields incorrect chi-square probabilities and false p-values.
    if m != 9 {
        return TestResult::insufficient(
            "nist::overlapping_template",
            "only m = 9 is supported; pi table is valid for m = 9, M = 1032 only",
        );
    }

    let n = bits.len();
    let big_m = 1032_usize; // SP 800-22 §2.8: M = 1032 for m = 9
    let num_blocks = n / big_m;

    if num_blocks < 5 {
        return TestResult::insufficient(
            "nist::overlapping_template",
            "n too small — need ≥ 5 blocks",
        );
    }

    let k = 5usize; // number of categories (0..=k, where k means "≥ k")

    // Theoretical probabilities for m = 9, M = 1032 (Table 4, SP 800-22 §2.8.7).
    let pi: [f64; 6] = [0.364091, 0.185659, 0.139381, 0.100571, 0.070432, 0.139865];

    let template: Vec<u8> = vec![1u8; m];

    let mut nu = [0usize; 6];
    for block in bits.chunks_exact(big_m).take(num_blocks) {
        let w = count_overlapping(block, &template);
        let idx = w.min(k);
        nu[idx] += 1;
    }

    let chi_sq: f64 = nu
        .iter()
        .zip(pi.iter())
        .map(|(&count, &p)| {
            let exp = num_blocks as f64 * p;
            (count as f64 - exp).powi(2) / exp
        })
        .sum();

    let p_value = igamc(k as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(
        "nist::overlapping_template",
        p_value,
        format!("n={n}, m={m}, N={num_blocks}, ν={nu:?}, χ²={chi_sq:.4}"),
    )
}

fn count_overlapping(block: &[u8], template: &[u8]) -> usize {
    let m = template.len();
    block.windows(m).filter(|&w| w == template).count()
}
