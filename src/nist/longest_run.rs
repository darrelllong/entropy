//! NIST SP 800-22 §2.4 — Test for the Longest Run of Ones in a Block.
//!
//! Divides the sequence into M-bit blocks, finds the longest run of 1s in
//! each block, and compares the distribution against theoretical values via
//! a chi-square goodness-of-fit test.
//!
//! The parameters (M and the distribution categories) depend on n, following
//! the tables in SP 800-22 §2.4.2 and §2.4.4.
//!
//! # References
//! * A. Rukhin et al., *NIST SP 800-22 Rev. 1a*, 2010, §2.4 and §3.4.
//!   [pubs/NIST-SP-800-22r1a.pdf]
//! * P. Revesz, *Random Walk in Random and Non-Random Environments*,
//!   World Scientific, 1990.  [Source §3.4 names for the class probabilities]

use crate::{math::chi2_pvalue, result::TestResult};

/// Class probabilities for M = 8, K = 3 (longest run ≤ 1, 2, 3, ≥ 4): the
/// number of 8-bit blocks in each class over 256.  §3.4 prints them rounded
/// to 0.2148, 0.3672, 0.2305 and 0.1875; the §2.4.8 example's χ² = 4.882457
/// uses the exact values.
const PI_M8: [f64; 4] = [55.0 / 256.0, 94.0 / 256.0, 59.0 / 256.0, 48.0 / 256.0];

/// Class probabilities for M = 128, K = 5 (longest run ≤ 4, 5, …, 8, ≥ 9),
/// to the four decimals SP 800-22 §3.4 prints.
const PI_M128: [f64; 6] = [0.1174, 0.2430, 0.2493, 0.1752, 0.1027, 0.1124];

/// Class probabilities for M = 10 000, K = 6 (longest run ≤ 10, 11, …, 15,
/// ≥ 16), to the four decimals SP 800-22 §3.4 prints.
const PI_M10000: [f64; 7] = [0.0882, 0.2092, 0.2483, 0.1933, 0.1208, 0.0675, 0.0727];

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
        (8, 3, 1, &PI_M8)
    } else if n < 750_000 {
        (128, 5, 4, &PI_M128)
    } else {
        (10_000, 6, 10, &PI_M10000)
    };

    let num_blocks = n / m;
    let mut freq = vec![0usize; k + 1];

    for block in bits.chunks_exact(m) {
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

    let p_value = chi2_pvalue(chi_sq, k);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nist::test_vectors::bits;

    /// ε of the SP 800-22 §2.4.8 example (n = 128, M = 8).
    const SECTION_2_4_8_EPSILON: &str = concat!(
        "11001100000101010110110001001100111000000000001001",
        "00110101010001000100111101011010000000110101111100",
        "1100111001101101100010110010",
    );

    /// SP 800-22 §2.4.8: ν = (4, 9, 3, 0), χ² = 4.882457, P-value = 0.180609.
    /// The rounded §3.4 probabilities give χ² = 4.882605 and
    /// P-value = 0.180598, the figures §2.4.4 shows for the same counts.
    #[test]
    fn matches_section_2_4_8_example() {
        let r = longest_run(&bits(SECTION_2_4_8_EPSILON));
        assert!((r.p_value - 0.180609).abs() < 1e-6, "{r}");
        assert!(r.note.as_deref().unwrap().contains("χ²=4.8825"), "{r}");
    }

    /// The M = 8 table is the longest-run distribution over all 256 blocks.
    #[test]
    fn m8_probabilities_match_enumeration() {
        let mut counts = [0usize; 4];
        for byte in 0u32..256 {
            let block: Vec<u8> = (0..8).map(|i| ((byte >> i) & 1) as u8).collect();
            counts[longest_run_of_ones(&block).clamp(1, 4) - 1] += 1;
        }
        assert_eq!(counts, [55, 94, 59, 48]);
        for (count, p) in counts.into_iter().zip(PI_M8) {
            assert!((count as f64 / 256.0 - p).abs() < 1e-15);
        }
    }

    #[test]
    fn probability_tables_sum_to_one() {
        for pi in [&PI_M8[..], &PI_M128, &PI_M10000] {
            let total: f64 = pi.iter().sum();
            assert!((total - 1.0).abs() < 1e-12, "Σ π = {total}");
        }
    }
}
