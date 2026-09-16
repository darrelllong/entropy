//! Lempel–Ziv compressibility test.
//!
//! P. L'Ecuyer and R. Simard, "TestU01: A C Library for Empirical Testing of
//! Random Number Generators," *ACM Transactions on Mathematical Software*
//! 33(4), Article 22, 2007, §5.1, "Lempel-Ziv complexity", p. 17
//! (`lecuyer2007testu01` in BIB.md).  [pubs/lecuyer-simard-2007-testu01.pdf]
//! J. Ziv and A. Lempel, "Compression of individual sequences via
//! variable-rate coding," *IEEE Transactions on Information Theory* 24(5),
//! pp. 530–536, 1978.
//!
//! A replication forms a stream of n = 2^k bits from successive `s`-bit
//! fields of the words and counts W, the phrases of
//! its LZ78 parse: each phrase is the shortest prefix of the rest of the
//! stream not already a phrase, and a final partial phrase counts only if it
//! is a proper prefix of some phrase.  W is approximately normal, and
//! z = (W − μₖ)/σₖ is its standardised value, with μₖ and σₖ from
//! `LZ_MEAN_SD`.
//!
//! [`lempel_ziv_summary`] reports a two-sided normal p-value for Σz/√N over
//! N replications and a Kolmogorov–Smirnov test of the values Φ(z).
//!
//! # The mean and standard deviation of W
//!
//! No closed form is accurate at these lengths, so `LZ_MEAN_SD` holds
//! estimates from `examples/lz78_table.rs`: for each k, the sample mean and
//! standard deviation of W over independent replications on PCG64 streams,
//! 100 032 replications for k ≤ 20 and 10 000 above.  The standard error of
//! each mean is at most 0.01σₖ, and of each σₖ about 0.7%.

use super::strip_b;
use crate::{
    math::{erfc, ks_test, normal_cdf},
    result::TestResult,
    rng::Rng,
};
use std::f64::consts::SQRT_2;

/// (μₖ, σₖ), the mean and standard deviation of the LZ78 phrase count of 2^k
/// fair random bits, k = 3 … 28 (entries 0 … 2 unused).
const LZ_MEAN_SD: [(f64, f64); 29] = [
    (0.0, 0.0),
    (0.0, 0.0),
    (0.0, 0.0),
    (4.434, 0.496),         // k = 3, 100032 replications
    (7.641, 0.505),         // k = 4, 100032 replications
    (12.508, 0.631),        // k = 5, 100032 replications
    (20.783, 0.728),        // k = 6, 100032 replications
    (34.757, 0.781),        // k = 7, 100032 replications
    (58.888, 0.845),        // k = 8, 100032 replications
    (101.149, 0.927),       // k = 9, 100032 replications
    (176.018, 1.050),       // k = 10, 100032 replications
    (310.011, 1.219),       // k = 11, 100032 replications
    (552.009, 1.437),       // k = 12, 100032 replications
    (992.358, 1.734),       // k = 13, 100032 replications
    (1799.143, 2.115),      // k = 14, 100032 replications
    (3286.182, 2.629),      // k = 15, 100032 replications
    (6041.573, 3.276),      // k = 16, 100032 replications
    (11171.365, 4.169),     // k = 17, 100032 replications
    (20761.977, 5.297),     // k = 18, 100032 replications
    (38760.635, 6.763),     // k = 19, 100032 replications
    (72653.958, 8.768),     // k = 20, 100032 replications
    (136676.172, 11.420),   // k = 21, 10000 replications
    (257949.059, 14.843),   // k = 22, 10000 replications
    (488257.934, 19.612),   // k = 23, 10000 replications
    (926658.580, 25.127),   // k = 24, 10000 replications
    (1762966.211, 33.567),  // k = 25, 10000 replications
    (3361487.892, 43.893),  // k = 26, 10000 replications
    (6422496.089, 58.491),  // k = 27, 10000 replications
    (12293926.846, 78.164), // k = 28, 10000 replications
];

/// Trie nodes that a parse of `n_bits` bits can need: the root plus the
/// largest possible phrase count.
///
/// Phrases are distinct non-empty strings whose lengths sum to at most
/// `n_bits`, so the count is largest when every string of length 1, then of
/// length 2, and so on, is used while they fit.
fn trie_reservation(n_bits: usize) -> usize {
    let (mut count, mut len, mut rem) = (0usize, 1u32, n_bits);
    loop {
        let available = 1usize.checked_shl(len).unwrap_or(usize::MAX);
        let fit = rem / len as usize;
        if fit <= available {
            return count + fit + 1;
        }
        count += available;
        rem -= available * len as usize;
        len += 1;
    }
}

#[derive(Debug, Clone)]
struct TrieNode {
    left: Option<usize>,
    right: Option<usize>,
}

/// Outcome of one Lempel-Ziv replication over a `2^k`-bit stream.
#[derive(Debug, Clone)]
pub struct LempelZivReplication {
    /// log2 of the bit-stream length (`n = 2^k`, `k` in `3..=28`).
    pub k: usize,
    /// Leading bits dropped from each 32-bit word.
    pub r: usize,
    /// Bits kept per word after the drop.
    pub s: usize,
    /// Number of `s`-bit words drawn to assemble the stream.
    pub words: usize,
    /// Standardised phrase count, `(phrase_count − μₖ) / σₖ`.
    pub z_score: f64,
    /// Raw LZ78 phrase count.
    pub phrase_count: usize,
}

/// Aggregate over `replications` Lempel-Ziv replications.
#[derive(Debug, Clone)]
pub struct LempelZivSummary {
    /// log2 of the per-replication stream length.
    pub k: usize,
    /// Leading bits dropped from each 32-bit word.
    pub r: usize,
    /// Bits kept per word after the drop.
    pub s: usize,
    /// Number of replications aggregated.
    pub replications: usize,
    /// Mean of the per-replication z-scores.
    pub z_mean: f64,
    /// Sum statistic `Σz / √N`, standard normal under the null.
    pub z_sum_stat: f64,
    /// Two-sided normal p-value of `z_sum_stat`.
    pub z_sum_p_value: f64,
    /// KS p-value for uniformity of `Φ(z)` across replications.
    pub z_ks_p_value: f64,
}

fn lz78_count_blocks(blocks: &[u32], n_bits: usize, s: usize) -> usize {
    let k_max = 1u32 << (s - 1);
    let mut nodes = Vec::with_capacity(trie_reservation(n_bits));
    nodes.push(TrieNode {
        left: None,
        right: None,
    });

    let mut block_index = 0usize;
    let mut y = blocks[block_index];
    let mut mask = k_max;
    let mut i = 0usize;
    let mut phrases = 0usize;

    while i < n_bits {
        let mut node = 0usize;
        loop {
            let take_left = (y & mask) == 0;
            let next_child = if take_left {
                nodes[node].left
            } else {
                nodes[node].right
            };
            let inserted = match next_child {
                Some(next) => {
                    node = next;
                    false
                }
                None => {
                    phrases += 1;
                    let next = nodes.len();
                    if take_left {
                        nodes[node].left = Some(next);
                    } else {
                        nodes[node].right = Some(next);
                    }
                    nodes.push(TrieNode {
                        left: None,
                        right: None,
                    });
                    node = next;
                    true
                }
            };

            i += 1;
            if i >= n_bits {
                if nodes[node].left.is_some() || nodes[node].right.is_some() {
                    phrases += 1;
                }
                break;
            }
            mask >>= 1;
            if mask == 0 {
                block_index += 1;
                y = blocks[block_index];
                mask = k_max;
            }
            if inserted {
                break;
            }
        }
    }

    phrases
}

/// One replication: the LZ78 phrase count over `2^k` bits drawn from `rng`,
/// standardised by the simulated mean and standard deviation.
///
/// # Panics
/// Panics if `k` is outside `3..=28`, `s` is outside `1..=32`, or
/// `r + s > 32`.
pub fn lempel_ziv_replication(
    rng: &mut impl Rng,
    k: usize,
    r: usize,
    s: usize,
) -> LempelZivReplication {
    assert!((3..=28).contains(&k), "k must be in 3..=28");
    assert!(s > 0 && s <= 32, "s must be in 1..=32");
    assert!(r <= 32 && r + s <= 32, "r + s must be <= 32");

    let n_bits = 1usize << k;
    let words = n_bits.div_ceil(s);
    let mut blocks = Vec::with_capacity(words);
    for _ in 0..words {
        blocks.push(strip_b(rng.next_u32(), r, s));
    }
    let phrase_count = lz78_count_blocks(&blocks, n_bits, s);
    let (mean, sd) = LZ_MEAN_SD[k];
    let z_score = (phrase_count as f64 - mean) / sd;

    LempelZivReplication {
        k,
        r,
        s,
        words,
        z_score,
        phrase_count,
    }
}

/// Run `replications` Lempel-Ziv replications and aggregate their z-scores
/// (sum statistic plus KS uniformity check), returning the per-replication
/// results alongside the summary.
///
/// # Panics
/// Panics if `replications == 0`, or on the same parameter violations as
/// [`lempel_ziv_replication`].
pub fn lempel_ziv_summary(
    rng: &mut impl Rng,
    replications: usize,
    k: usize,
    r: usize,
    s: usize,
) -> (Vec<LempelZivReplication>, LempelZivSummary) {
    assert!(replications > 0, "replications must be positive");
    let reps: Vec<LempelZivReplication> = (0..replications)
        .map(|_| lempel_ziv_replication(rng, k, r, s))
        .collect();

    let z_sum: f64 = reps.iter().map(|rep| rep.z_score).sum();
    let z_mean = z_sum / replications as f64;
    let z_sum_stat = z_sum / (replications as f64).sqrt();
    // A guard only: `math::erfc` never exceeds 1 for a non-negative argument.
    let z_sum_p_value = erfc(z_sum_stat.abs() / SQRT_2).min(1.0);
    let mut uniforms: Vec<f64> = reps.iter().map(|rep| normal_cdf(rep.z_score)).collect();
    let z_ks_p_value = ks_test(&mut uniforms);

    (
        reps,
        LempelZivSummary {
            k,
            r,
            s,
            replications,
            z_mean,
            z_sum_stat,
            z_sum_p_value,
            z_ks_p_value,
        },
    )
}

/// Package the z-sum statistic from `summary` as a [`TestResult`] named
/// `testu01::lzw_sum`.
pub fn lempel_ziv_sum_result(summary: &LempelZivSummary) -> TestResult {
    TestResult::with_note(
        "testu01::lzw_sum",
        summary.z_sum_p_value,
        format!(
            "N={}, k={}, r={}, s={}, z_mean={:.4}, z_sum={:.4}",
            summary.replications,
            summary.k,
            summary.r,
            summary.s,
            summary.z_mean,
            summary.z_sum_stat
        ),
    )
}

/// Package the KS uniformity check from `summary` as a [`TestResult`]
/// named `testu01::lzw_ks`.
pub fn lempel_ziv_ks_result(summary: &LempelZivSummary) -> TestResult {
    TestResult::with_note(
        "testu01::lzw_ks",
        summary.z_ks_p_value,
        format!(
            "N={}, k={}, r={}, s={}",
            summary.replications, summary.k, summary.r, summary.s
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{lempel_ziv_replication, lempel_ziv_summary, lz78_count_blocks, trie_reservation};
    use crate::rng::{Rng, Xorshift32};

    /// Marsaglia's example xorshift32 seed; any non-zero seed would do.
    const XORSHIFT_SEED: u32 = 2_463_534_242;

    /// The reservation for tiny streams, counted by hand: 2 bits hold "0"
    /// and "1"; 7 bits hold them and two 2-bit phrases; 10 bits hold them and
    /// all four 2-bit phrases.  The root adds one node.
    #[test]
    fn trie_reservation_is_the_worst_case_plus_the_root() {
        assert_eq!(trie_reservation(1), 2);
        assert_eq!(trie_reservation(2), 3);
        assert_eq!(trie_reservation(7), 5);
        assert_eq!(trie_reservation(10), 7);
        for k in 3..=28 {
            let n = 1usize << k;
            assert!(trie_reservation(n) < n, "k = {k}");
        }
    }

    /// LZ78 parses derived by hand (MSB first), pinning the end-of-stream
    /// rule: a trailing partial phrase counts only if its trie node has a
    /// child.
    /// - `0100110`: 0 | 1 | 00 | 11 | 0…, and "0" has the child "00": 5.
    /// - `01001111`: 0 | 1 | 00 | 11 | 11…, and "11" is a leaf: 4.
    /// - `01001101`: 0 | 1 | 00 | 11 | 01, a phrase ending with the stream: 5.
    #[test]
    fn lz78_hand_parses_pin_end_of_stream_rule() {
        // One 8-bit field per string; the 7-bit stream ignores the last bit.
        assert_eq!(5, lz78_count_blocks(&[0b0100_1100], 7, 8));
        assert_eq!(4, lz78_count_blocks(&[0b0100_1111], 8, 8));
        assert_eq!(5, lz78_count_blocks(&[0b0100_1101], 8, 8));
        // The same strings split across 4-bit fields.
        assert_eq!(5, lz78_count_blocks(&[0b0100, 0b1100], 7, 4));
        assert_eq!(4, lz78_count_blocks(&[0b0100, 0b1111], 8, 4));
        assert_eq!(5, lz78_count_blocks(&[0b0100, 0b1101], 8, 4));
    }

    /// Pins words drawn and phrase counts for Xorshift32 streams, including
    /// `s` that does not divide `2^k`, so a change to the trie cannot change
    /// the statistic.  The counts were checked with a separate LZ78 counter
    /// built on a set of phrase strings rather than a trie.
    #[test]
    fn replication_phrase_counts_match_independent_replica() {
        for (k, r, s, words, phrases) in [
            (10, 0, 30, 35, 176),
            (13, 3, 7, 1171, 989),
            (16, 1, 31, 2115, 6044),
        ] {
            let mut rng = Xorshift32::new(XORSHIFT_SEED);
            let rep = lempel_ziv_replication(&mut rng, k, r, s);
            assert_eq!(words, rep.words, "k = {k}, r = {r}, s = {s}");
            assert_eq!(phrases, rep.phrase_count, "k = {k}, r = {r}, s = {s}");
        }
    }

    /// Replications continue one stream, each drawing ⌈2^k/s⌉ words; phrase
    /// counts pinned.
    #[test]
    fn summary_replications_continue_one_stream() {
        for (replications, k, r, s, phrases, calls) in [
            (3, 12, 5, 9, &[556usize, 552, 554][..], 1368),
            (2, 13, 3, 7, &[989usize, 992][..], 2342),
        ] {
            let mut rng = Xorshift32::new(XORSHIFT_SEED);
            let (reps, summary) = lempel_ziv_summary(&mut rng, replications, k, r, s);
            let counts: Vec<usize> = reps.iter().map(|rep| rep.phrase_count).collect();
            assert_eq!(phrases, &counts[..], "k = {k}, r = {r}, s = {s}");
            assert_eq!(replications, summary.replications);
            let mut fresh = Xorshift32::new(XORSHIFT_SEED);
            for _ in 0..calls {
                fresh.next_u32();
            }
            assert_eq!(fresh.next_u32(), rng.next_u32(), "k = {k}: {calls} calls");
        }
    }

    #[test]
    fn lz78_counts_constant_zero_stream_reasonably() {
        let blocks = vec![0u32; 8];
        let phrases = lz78_count_blocks(&blocks, 16, 2);
        assert!(phrases >= 4);
        assert!(phrases <= 8);
    }
}
