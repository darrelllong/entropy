//! DIEHARD Test 2 — Overlapping 5-Permutations (OPERM5).
//!
//! Examines each overlapping window of 5 consecutive 32-bit integers from the
//! test sequence.  There are 5! = 120 possible orderings.  The test uses a
//! chi-square statistic based on a 120×120 covariance matrix (rank 99).
//!
//! The simplified chi-square used here follows the DIEHARD documentation:
//! compare observed frequencies to 1/120 expected frequency.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::igamc, result::TestResult};

const N_WORDS: usize = 1_000_000;
const N_PERMS: usize = 120;

/// Run the OPERM5 test.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn operm5(words: &[u32]) -> TestResult {
    let n = words.len().min(N_WORDS);
    if n < 10_000 {
        return TestResult::insufficient("diehard::operm5", "need ≥ 10 000 words");
    }

    // Count occurrences of each of the 120 orderings in every window of 5.
    let mut counts = [0u32; N_PERMS];
    for window in words[..n].windows(5) {
        let rank = perm5_rank(window);
        counts[rank] += 1;
    }

    let total = (n - 4) as f64;
    let expected = total / N_PERMS as f64;

    let chi_sq: f64 = counts
        .iter()
        .map(|&c| (c as f64 - expected).powi(2) / expected)
        .sum();

    // df = 119 (N_PERMS − 1)
    let p_value = igamc(119.0 / 2.0, chi_sq / 2.0);

    TestResult::with_note(
        "diehard::operm5",
        p_value,
        format!("n={n}, χ²={chi_sq:.4}"),
    )
}

/// Map a window of 5 u32 values to a rank in 0..120 (the lexicographic index
/// of the relative ordering).
fn perm5_rank(w: &[u32]) -> usize {
    // Compute the rank of the permutation induced by the ordering of w[0..5].
    let mut order = [0usize, 1, 2, 3, 4];
    order.sort_unstable_by_key(|&i| w[i]);

    // Convert permutation to factorial-number-system rank (Lehmer code).
    let mut rank = 0usize;
    let mut used = [false; 5];
    for (pos, &orig_idx) in order.iter().enumerate() {
        let count = order[pos..].iter().filter(|&&i| i < orig_idx && !used[i]).count();
        rank += count * factorial(4 - pos);
        used[orig_idx] = true;
    }
    rank.min(N_PERMS - 1)
}

const fn factorial(n: usize) -> usize {
    match n {
        0 => 1,
        1 => 1,
        2 => 2,
        3 => 6,
        4 => 24,
        _ => 120,
    }
}
