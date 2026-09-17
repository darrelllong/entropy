//! NIST SP 800-22 §2.4 — Test for the Longest Run of Ones in a Block.
//!
//! Divides the sequence into M-bit blocks, finds the longest run of 1s in
//! each block, and compares the distribution against theoretical values via
//! a chi-square goodness-of-fit test.
//!
//! The parameters (M and the distribution categories) depend on n, following
//! the tables in SP 800-22 §2.4.2 and §2.4.4.  The class probabilities are
//! exact, from the recurrence in `longest_run_at_most`; §3.4 prints them to
//! four decimals.
//!
//! # References
//! * A. Rukhin et al., *NIST SP 800-22 Rev. 1a*, 2010, §2.4 and §3.4.
//!   [pubs/NIST-SP-800-22r1a.pdf]
//! * P. Revesz, *Random Walk in Random and Non-Random Environments*,
//!   World Scientific, 1990.  [Source §3.4 names for the class probabilities]

use crate::{math::chi2_pvalue, result::TestResult};

/// SP 800-22 §3.4's class probabilities for M = 128 (longest run ≤ 4, 5, …,
/// 8, ≥ 9), printed to four decimals, up to 6.4 × 10⁻⁵ from the exact values.
/// The test uses the exact values; the table explains the publication's
/// figures.
#[cfg(test)]
const PRINTED_M128: [f64; 6] = [0.1174, 0.2430, 0.2493, 0.1752, 0.1027, 0.1124];

/// SP 800-22 §3.4's class probabilities for M = 10 000 (longest run ≤ 10,
/// 11, …, 15, ≥ 16), printed to four decimals, up to 1.6 × 10⁻³ from the
/// exact values.  A fixed error δ in the cell probabilities adds about
/// N·Σ δ²/π to Pearson's statistic over N blocks, one unit by N ≈ 20 700 at
/// this block size, which is why the test does not use them.
#[cfg(test)]
const PRINTED_M10000: [f64; 7] = [0.0882, 0.2092, 0.2483, 0.1933, 0.1208, 0.0675, 0.0727];

/// P(longest run of ones in `m` fair bits ≤ `k`).
///
/// The probabilities of the current run length 0 … k are stepped bit by
/// bit: a 0 sends all the mass to run length 0 and a 1 moves each length up
/// by one, each with probability ½, and mass reaching k + 1 is dropped.
/// After m bits the remaining mass is the probability.
fn longest_run_at_most(m: usize, k: usize) -> f64 {
    let mut run = vec![0.0; k + 1];
    run[0] = 1.0;
    for _ in 0..m {
        let total: f64 = run.iter().sum();
        run.rotate_right(1);
        run[0] = total;
        run.iter_mut().for_each(|p| *p /= 2.0);
    }
    run.iter().sum()
}

/// Exact probabilities of the classes ≤ v, v + 1, …, v + k − 1, ≥ v + k for
/// the longest run of ones in an m-bit block.
fn class_probabilities(m: usize, v: usize, k: usize) -> Vec<f64> {
    let cdf: Vec<f64> = (v..v + k).map(|j| longest_run_at_most(m, j)).collect();
    let mut classes = vec![cdf[0]];
    classes.extend(cdf.windows(2).map(|w| w[1] - w[0]));
    classes.push(1.0 - cdf[k - 1]);
    classes
}

/// Run the longest-run-of-ones test.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.4.
pub fn longest_run(bits: &[u8]) -> TestResult {
    let n = bits.len();

    // Table 1: choose block size M and categories based on n.
    let (m, k, v_min) = if n < 128 {
        return TestResult::insufficient("nist::longest_run", "n < 128");
    } else if n < 6_272 {
        (8, 3, 1)
    } else if n < 750_000 {
        (128, 5, 4)
    } else {
        (10_000, 6, 10)
    };
    let pi = class_probabilities(m, v_min, k);

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
    .chi_square(chi_sq, k as f64)
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

    /// The M = 8 classes are the longest-run distribution over all 256 blocks.
    #[test]
    fn m8_probabilities_match_enumeration() {
        let mut counts = [0usize; 4];
        for byte in 0u32..256 {
            let block: Vec<u8> = (0..8).map(|i| ((byte >> i) & 1) as u8).collect();
            counts[longest_run_of_ones(&block).clamp(1, 4) - 1] += 1;
        }
        assert_eq!(counts, [55, 94, 59, 48]);
        for (count, p) in counts.into_iter().zip(class_probabilities(8, 1, 3)) {
            assert!((count as f64 / 256.0 - p).abs() < 1e-15);
        }
    }

    /// The recurrence against enumeration of all 2¹⁶ blocks of 16 bits.
    #[test]
    fn recurrence_matches_enumeration_at_sixteen_bits() {
        let mut at_most = [0u32; 17];
        for word in 0u32..1 << 16 {
            let block: Vec<u8> = (0..16).map(|i| ((word >> i) & 1) as u8).collect();
            let longest = longest_run_of_ones(&block);
            for (k, c) in at_most.iter_mut().enumerate() {
                *c += u32::from(longest <= k);
            }
        }
        for (k, &c) in at_most.iter().enumerate() {
            let want = f64::from(c) / 65_536.0;
            assert!((longest_run_at_most(16, k) - want).abs() < 1e-15, "k = {k}");
        }
    }

    /// SP 800-22 Appendix B's P-value 0.718945 for 10⁶ bits of e follows from
    /// the printed M = 10 000 table; the exact classes give 0.718366.
    #[test]
    fn appendix_b_figure_follows_from_the_printed_table() {
        let e = crate::nist::test_vectors::e_bits(1_000_000);
        let mut freq = [0usize; 7];
        for block in e.chunks_exact(10_000) {
            freq[longest_run_of_ones(block).clamp(10, 16) - 10] += 1;
        }
        let chi: f64 = freq
            .iter()
            .zip(PRINTED_M10000)
            .map(|(&c, p)| (c as f64 - 100.0 * p).powi(2) / (100.0 * p))
            .sum();
        assert!(
            (chi2_pvalue(chi, 6) - 0.718945).abs() < 1e-6,
            "{}",
            chi2_pvalue(chi, 6)
        );
        assert!((longest_run(&e).p_value - 0.718366).abs() < 1e-6);
    }

    fn max_gap(a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0, f64::max)
    }

    /// The printed M = 128 and M = 10 000 tables against the exact classes.
    #[test]
    fn printed_tables_against_exact_distribution() {
        let m128 = class_probabilities(128, 4, 5);
        let printed_gap = max_gap(&m128, &PRINTED_M128);
        assert!((6.3e-5..6.4e-5).contains(&printed_gap), "{printed_gap}");
        let m10000_gap = max_gap(&class_probabilities(10_000, 10, 6), &PRINTED_M10000);
        assert!((1.5e-3..1.6e-3).contains(&m10000_gap), "{m10000_gap}");
    }

    #[test]
    fn probability_tables_sum_to_one() {
        for pi in [
            class_probabilities(8, 1, 3),
            class_probabilities(128, 4, 5),
            class_probabilities(10_000, 10, 6),
        ] {
            let total: f64 = pi.iter().sum();
            assert!((total - 1.0).abs() < 1e-12, "Σ π = {total}");
        }
    }
}
