//! NIST SP 800-22 §2.4 — Test for the Longest Run of Ones in a Block.
//!
//! Divides the sequence into M-bit blocks, finds the longest run of 1s in
//! each block, and compares the distribution against theoretical values via
//! a chi-square goodness-of-fit test.
//!
//! The parameters (M and the distribution categories) depend on n, following
//! Table 1 of SP 800-22 §2.4.4.

use crate::{math::igamc, result::TestResult};

/// Run the longest-run-of-ones test.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.4.
pub fn longest_run(bits: &[u8]) -> TestResult {
    let n = bits.len();

    // Table 1: choose block size M and categories based on n.
    let (m, k, v_min, pi): (usize, usize, usize, &[f64]) = if n < 128 {
        return TestResult::insufficient("nist::longest_run", "n < 128");
    } else if n < 6_272 {
        (8, 3, 1, &[0.2148, 0.3672, 0.2305, 0.1875])
    } else if n < 750_000 {
        (128, 5, 4, &[0.1174, 0.2430, 0.2493, 0.1752, 0.1027, 0.1124])
    } else {
        (
            10_000,
            6,
            10,
            &[0.0882, 0.2092, 0.2483, 0.1933, 0.1208, 0.0675, 0.0727],
        )
    };

    let num_blocks = n / m;
    let mut freq = vec![0usize; k + 1];

    for block in bits.chunks_exact(m).take(num_blocks) {
        let longest = longest_run_of_ones(block);
        let idx = if longest < v_min {
            0
        } else if longest >= v_min + k {
            k
        } else {
            longest - v_min
        };
        freq[idx] += 1;
    }

    let chi_sq: f64 = freq
        .iter()
        .zip(pi.iter())
        .map(|(&count, &p)| {
            let expected = num_blocks as f64 * p;
            (count as f64 - expected).powi(2) / expected
        })
        .sum();

    let p_value = igamc(k as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(
        "nist::longest_run",
        p_value,
        format!("n={n}, M={m}, N={num_blocks}, χ²={chi_sq:.4}"),
    )
}

fn longest_run_of_ones(block: &[u8]) -> usize {
    let mut max_run = 0usize;
    let mut cur_run = 0usize;
    for &b in block {
        if b == 1 {
            cur_run += 1;
            if cur_run > max_run {
                max_run = cur_run;
            }
        } else {
            cur_run = 0;
        }
    }
    max_run
}
