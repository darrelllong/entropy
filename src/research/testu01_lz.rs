//! TestU01 Lempel-Ziv compression test core statistic.
//!
//! This ports the core of `scomp_LempelZiv` from TestU01 1.2.3:
//! - the exact `LZ78` trie walk over a bit stream assembled via `unif01_StripB`
//! - the official empirical `LZMu` / `LZSigma` tables for `n = 2^k`, `3 <= k <= 28`
//!
//! The full TestU01 post-processing runs a goodness-of-fit battery over the
//! normalized observations. This module exposes the exact per-replication
//! normalized scores and a lightweight summary, but does not claim to
//! reproduce TestU01's entire reporting layer.

use crate::{
    math::{erfc, ks_test, normal_cdf},
    result::TestResult,
    rng::Rng,
};
use std::f64::consts::SQRT_2;

const LZ_MU: [f64; 29] = [
    0.0, 0.0, 0.0, 4.44, 7.64, 12.5, 20.8, 34.8, 58.9, 101.1, 176.0, 310.0, 551.9, 992.3, 1799.0,
    3286.2, 6041.5, 11171.5, 20761.8, 38760.4, 72654.0, 136677.0, 257949.0, 488257.0, 926658.0,
    1762965.0, 3361490.0, 6422497.0, 12293930.0,
];

const LZ_SIGMA: [f64; 29] = [
    0.0, 0.0, 0.0, 0.49, 0.51, 0.62, 0.75, 0.78, 0.86, 0.94, 1.03, 1.19, 1.43, 1.68, 2.09, 2.46,
    3.36, 4.2, 5.4, 6.8, 9.1, 10.9, 14.7, 19.1, 25.2, 33.5, 44.546, 58.194, 75.513,
];

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
    /// Leading bits dropped from each 32-bit word (TestU01 `r`).
    pub r: usize,
    /// Bits kept per word after the drop (TestU01 `s`).
    pub s: usize,
    /// Number of `s`-bit words drawn to assemble the stream.
    pub words: usize,
    /// Normalized phrase count, `(phrase_count − LZMu[k]) / LZSigma[k]`.
    pub z_score: f64,
    /// Raw LZ78 phrase count.
    pub phrase_count: usize,
}

/// Aggregate over `replications` Lempel-Ziv replications.
#[derive(Debug, Clone)]
pub struct LempelZivSummary {
    /// log2 of the per-replication stream length.
    pub k: usize,
    /// Leading bits dropped from each 32-bit word (TestU01 `r`).
    pub r: usize,
    /// Bits kept per word after the drop (TestU01 `s`).
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

fn strip_b(word: u32, r: usize, s: usize) -> u32 {
    if r == 0 {
        word >> (32 - s)
    } else {
        (word << r) >> (32 - s)
    }
}

fn lz78_count_blocks(blocks: &[u32], n_bits: usize, s: usize) -> usize {
    let k_max = 1u32 << (s - 1);
    let mut nodes = Vec::with_capacity(n_bits / 4 + 1);
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

/// One `scomp_LempelZiv` replication: the LZ78 phrase count over `2^k` bits
/// drawn from `rng`, normalized by the official `LZMu`/`LZSigma` tables.
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
    assert!(r + s <= 32, "r + s must be <= 32");

    let n_bits = 1usize << k;
    let words = n_bits.div_ceil(s);
    let mut blocks = Vec::with_capacity(words);
    for _ in 0..words {
        blocks.push(strip_b(rng.next_u32(), r, s));
    }
    let phrase_count = lz78_count_blocks(&blocks, n_bits, s);
    let z_score = (phrase_count as f64 - LZ_MU[k]) / LZ_SIGMA[k];

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
    let z_sum_p_value = erfc(z_sum_stat.abs() / SQRT_2);
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
    use super::{lz78_count_blocks, strip_b};

    #[test]
    fn strip_b_uses_most_significant_bits_like_testu01() {
        let word = 0xDEAD_BEEF;
        assert_eq!(0xD, strip_b(word, 0, 4));
        assert_eq!(0xE, strip_b(word, 4, 4));
    }

    #[test]
    fn lz78_counts_constant_zero_stream_reasonably() {
        let blocks = vec![0u32; 8];
        let phrases = lz78_count_blocks(&blocks, 16, 2);
        assert!(phrases >= 4);
        assert!(phrases <= 8);
    }
}
