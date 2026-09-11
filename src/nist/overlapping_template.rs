//! NIST SP 800-22 §2.8 — Overlapping Template Matching Test.
//!
//! Uses an all-ones template of length `m` (default m = 9).  Counts
//! overlapping occurrences in each of N = n/1032 blocks of M = 1032 bits,
//! bins the counts into 6 categories, and applies a chi-square test against
//! the class probabilities SP 800-22 lists in §2.8.4 and §3.8.
//!
//! Minimum recommended: n ≥ 10^6 (giving ≥ 968 blocks for m = 9, M = 1032).
//!
//! # References
//! * A. Rukhin et al., *NIST SP 800-22 Rev. 1a*, 2010, §2.8 and §3.8.
//!   [pubs/NIST-SP-800-22r1a.pdf]
//! * K. Hamano and T. Kaneko, "The Correction of the Overlapping Template
//!   Matching Test Included in NIST Randomness Test Suite," *IEICE
//!   Transactions on Fundamentals of Electronics, Communications and Computer Sciences*
//!   E90-A(9), pp. 1788–1792, 2007.  [Source of the π values, per §3.8]

use crate::{math::igamc, result::TestResult};

/// Run the overlapping template test.
///
/// The template is the all-ones pattern of length `m`.  Only `m = 9` with
/// `M = 1032` is supported: the π table (SP 800-22 §2.8.4 step (4), §3.8) is
/// for that pair, where λ = (M − m + 1)/2^m = 2.  Passing any other value of
/// `m` returns an insufficient result.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.8.
pub fn overlapping_template(bits: &[u8], m: usize) -> TestResult {
    // The π table below holds only for m = 9, M = 1032 (SP 800-22 §2.8.4
    // step (4), §3.8).  Using it for any other m yields incorrect chi-square
    // probabilities and false p-values.
    if m != 9 {
        return TestResult::insufficient(
            "nist::overlapping_template",
            "only m = 9 is supported; pi table is valid for m = 9, M = 1032 only",
        );
    }

    let n = bits.len();
    let big_m = 1032_usize; // SP 800-22 §2.8: M = 1032 for m = 9
    let num_blocks = n / big_m;

    // The χ² approximation needs expected counts ≥ 5 in every category:
    // N·min(πᵢ) ≥ 5 with min(πᵢ) ≈ 0.0704 → N ≥ 72 blocks (n ≥ 74 304).
    // (NIST recommends n ≥ 10⁶, N = 968.)
    if num_blocks < 72 {
        return TestResult::insufficient(
            "nist::overlapping_template",
            "n too small — need ≥ 72 blocks of 1032 bits for valid χ² expected counts",
        );
    }

    let k = 5usize; // number of categories (0..=k, where k means "≥ k")

    // π₀…π₅ for m = 9, M = 1032, as SP 800-22 §2.8.4 step (4) lists them.
    // §3.8 prints the compound-Poisson formulas for P(U = u) with η = λ/2,
    // then says the "more accurate, but also more complicated, expressions"
    // of Hamano and Kaneko (2007) yield these values (it gives π₄ = 0.0704323).
    //
    // The §2.8.8 worked example predates them: its counts
    // ν = (329, 164, 150, 111, 78, 136) give the printed χ² = 8.965859 and
    // P-value = 0.110434 with the compound-Poisson values (π₀ = e^{−η} ≈
    // 0.367879 for η = 1), and χ² ≈ 7.949747 with the values below.
    let pi: [f64; 6] = [0.364091, 0.185659, 0.139381, 0.100571, 0.070432, 0.139865];

    let template: Vec<u8> = vec![1u8; m];

    let mut nu = [0usize; 6];
    for block in bits.chunks_exact(big_m) {
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
