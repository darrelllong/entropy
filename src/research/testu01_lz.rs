//! TestU01 Lempel-Ziv compression test core statistic.
//!
//! This ports the core of `scomp_LempelZiv` from TestU01 1.2.3:
//! - the exact `LZ78` trie walk over a bit stream assembled via `unif01_StripB`
//! - the official empirical `LZMu` / `LZSigma` tables for `n = 2^k`, `3 <= k <= 28`
//!
//! Both follow `scomp.c`: the walk matches `LZ78` step by step, including
//! its end-of-stream rule and the ⌈2^k/s⌉ words each replication draws, and
//! the tables match digit for digit.  The tests pin phrase counts and
//! generator calls against TestU01 1.2.3 itself.
//!
//! The full TestU01 post-processing runs a goodness-of-fit battery over the
//! normalized observations. This module exposes the exact per-replication
//! normalized scores and a lightweight summary, but does not claim to
//! reproduce TestU01's entire reporting layer.  In particular, for `N > 1`
//! TestU01 reports the right tail `1 − Φ(Σz/√N)` of the sum statistic
//! (`sres_GetNormalSumStat` in `testu01/sres.c`) and applies its active
//! empirical-distribution tests to the values `Φ(z)` (`gofw_ActiveTests2`
//! in `probdist/gofw.c`).  [`lempel_ziv_summary`] reports a two-sided normal
//! p-value for the sum and a two-sided Kolmogorov–Smirnov p-value instead.
//!
//! # References
//! * P. L'Ecuyer and R. Simard, "TestU01: A C Library for Empirical Testing
//!   of Random Number Generators," *ACM Transactions on Mathematical
//!   Software* 33(4), Article 22, 2007, §5.1,
//!   "Lempel-Ziv complexity", p. 17 (`lecuyer2007testu01` in BIB.md).
//!   [pubs/lecuyer-simard-2007-testu01.pdf]
//! * J. Ziv and A. Lempel, "Compression of individual sequences via
//!   variable-rate coding," *IEEE Transactions on Information Theory* 24(5),
//!   pp. 530–536, 1978.  [LZ78; cited from the 2007 paper's reference list.]
//! * TestU01 1.2.3 source (`testu01-source` in BIB.md): `testu01/scomp.c`
//!   (`scomp_LempelZiv`, `LZ78`, and the `LZMu` and `LZSigma` tables) and
//!   `testu01/unif01.c` (`unif01_StripB`).
//!   [pubs/TestU01-2009-57e98bf33880.tar.gz]
//!
//! # Author
//! Pierre L'Ecuyer and Richard Simard (TestU01); Darrell Long (Rust port).

use super::strip_b;
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

/// Reservation factor applied to `LZ_MU[k]` by [`trie_reservation`].
const TRIE_RESERVE_FACTOR: f64 = 9.0 / 8.0;

/// Trie nodes reserved for an `n_bits`-bit stream: `LZ_MU[k] · 9/8 + 2` for
/// `n_bits = 2^k`, a multiple of the expected phrase count.
///
/// The trie holds the root plus one node per inserted phrase, and inserted
/// phrases are distinct non-empty strings whose lengths sum to at most
/// `n_bits`.  No stream can therefore need more nodes than one plus the
/// number of shortest distinct strings that fit in `n_bits` bits.  The
/// reservation ⌊9/8 · `LZ_MU[k]`⌋ + 2 covers that worst case plus the root
/// for every `k` in the table, checked k by k: at k = 3 with no slack (five
/// phrases, six nodes reserved), and with room to spare for k ≥ 4, where the
/// worst case stays under 1.05 · `LZ_MU[k]`.  Lengths that are not a power of two
/// (tests only) use the entry for ⌊log2 n_bits⌋ and let the vector grow.
fn trie_reservation(n_bits: usize) -> usize {
    let k = n_bits.checked_ilog2().map_or(0, |k| k as usize);
    let mu = LZ_MU.get(k).copied().unwrap_or(0.0);
    (mu * TRIE_RESERVE_FACTOR) as usize + 2
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
    assert!(r <= 32 && r + s <= 32, "r + s must be <= 32");

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
    use super::{
        lempel_ziv_replication, lempel_ziv_summary, lz78_count_blocks, trie_reservation, LZ_MU,
    };
    use crate::rng::{Rng, Xorshift32};

    /// Marsaglia's example xorshift32 seed; any non-zero seed would do.
    const XORSHIFT_SEED: u32 = 2_463_534_242;

    /// Worst-case LZ78 phrase count for an `n`-bit string: take every
    /// distinct string of length 1, then of length 2, and so on, while they
    /// fit in `n` bits.
    fn max_lz78_phrases(n: usize) -> usize {
        let (mut count, mut len, mut rem) = (0usize, 1usize, n);
        loop {
            let avail = 1usize << len;
            let fit = rem / len;
            if fit <= avail {
                return count + fit;
            }
            count += avail;
            rem -= avail * len;
            len += 1;
        }
    }

    /// Regression: the trie reserved `n_bits/4 + 1` nodes (256 MiB at
    /// k = 25, 2 GiB at k = 28).  The reservation now tracks `LZ_MU` and must
    /// still hold the root plus the worst-case phrase count.
    #[test]
    fn trie_reservation_tracks_lz_mu_and_covers_worst_case() {
        for (k, &mu) in LZ_MU.iter().enumerate().skip(3) {
            let n = 1usize << k;
            let reserve = trie_reservation(n);
            assert!(
                reserve > max_lz78_phrases(n),
                "k = {k}: {reserve} nodes cannot hold the worst case"
            );
            assert!(
                (reserve as f64) < 1.2 * mu + 2.0,
                "k = {k}: {reserve} nodes over-reserve"
            );
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
    /// the statistic.  Reference: an independent Python LZ78 counter over the
    /// same `unif01_StripB` bit stream, built on a set of phrase strings
    /// rather than a trie.  TestU01 1.2.3's `scomp_LempelZiv` gives the same
    /// counts and generator calls for all three.
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

    /// Phrase counts and generator calls from TestU01 1.2.3 itself: the
    /// library built from pubs/TestU01-2009-57e98bf33880.tar.gz, running
    /// `scomp_LempelZiv` on this Xorshift32 stream through
    /// `unif01_CreateExternGenBits`, with `swrite_Counters` printing each
    /// replication's phrase count.  Replications continue one stream, each
    /// drawing ⌈2^k/s⌉ words.
    #[test]
    fn summary_phrase_counts_match_testu01() {
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
