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
//! fields of the words and counts W, the phrases of its LZ78 parse: each
//! phrase is the shortest prefix of the rest of the stream not already a
//! phrase, and a final partial phrase counts only if it is a proper prefix of
//! some phrase.
//!
//! # The null law
//!
//! W is discrete, with F(w) = P(W ≤ w) and P(w) = P(W = w) taken from
//! `src/research/lz78_counts.rs`: exact for k ≤ 5, simulated above.
//! Each replication is mapped to
//!
//! U = F(W − 1) + V·P(W),
//!
//! with V uniform on (0, 1) from a generator seeded separately from the one
//! under test.  Under the tabulated law U is exactly uniform, where Φ(z) of a
//! standardised W is not: it takes only as many values as W does.  Over N
//! replications [`lempel_ziv_summary`] reports a Kolmogorov–Smirnov test of
//! the U and the two-sided normal p-value of Z = Σ Φ⁻¹(U)/√N, which is
//! exactly standard normal under the same law.
//!
//! A simulated table of M replications estimates F with a sup-norm error of
//! about 0.87/√M, against the 0.87/√N scale of the Kolmogorov–Smirnov
//! statistic and a mean shift of order √(N/M) in Z.  N is therefore limited
//! to M/100, which keeps both errors near a tenth of the test's own noise.
//! Larger N is reported as insufficient.  A simulated table gives one
//! pseudo-replication below its smallest and above its largest W, so a count
//! outside the table has probability 1/(M + 2) rather than 0.
//!
//! # Validation
//!
//! xoshiro256** streams, seeded apart from the PCG64 streams behind the
//! tables, rejected at 0.01 near the nominal rate in both statistics:
//!
//! | k | N | runs | Z | KS |
//! |---|---|---|---|---|
//! | 3 … 12 | 10 000 and 10 | 1 000 and 20 000 each | 0.7–1.6% | 0.8–1.2% |
//! | 17 | 10 000 | 300 | 0.33% | 1.33% |
//! | 20 | 1 000 | 2 000 | 1.30% | 1.25% |
//! | 20 | 10 000 | 200 | 1.50% | 1.00% |
//! | 22 | 10 000 | 100 | 1.00% | 2.00% |
//! | 23 | 1 000 | 150 | 0.67% | 0.67% |
//! | 25 | 1 000 | 80 | 1.25% | 2.50% |
//!
//! With tables a tenth as large, k = 20 at N = 1 000 had rejected 2.3% and
//! 2.5% of 400 runs; the larger table removed that excess.  The runs above
//! k = 20 are few, so they bound only gross miscalibration.

use super::{lz78_counts::PHRASE_COUNTS, strip_b};
use crate::{
    math::{erfc, ks_test, normal_quantile},
    result::TestResult,
    rng::{Pcg64, Rng},
};
use std::f64::consts::SQRT_2;

/// Stream of the PCG64 generator that draws V, "lzpit".
const PIT_STREAM: u128 = 0x6c_7a70_6974;

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
    /// Number of `s`-bit words drawn to assemble the stream.
    pub words: usize,
    /// Raw LZ78 phrase count W.
    pub phrase_count: usize,
    /// U = F(W − 1) + V·P(W), uniform under the table.
    pub uniform: f64,
    /// Φ⁻¹(U), standard normal under the table.
    pub z_score: f64,
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
    /// Number of replications requested.
    pub replications: usize,
    /// Seed of the generator that draws V.
    pub pit_seed: u64,
    /// Why no replication was run, when the table cannot support the request.
    pub unsupported: Option<String>,
    /// Z = Σ Φ⁻¹(U) / √N.
    pub z_sum_stat: f64,
    /// Two-sided normal p-value of `z_sum_stat`.
    pub z_sum_p_value: f64,
    /// Kolmogorov–Smirnov p-value for uniformity of the U.
    pub ks_p_value: f64,
    /// Kolmogorov–Smirnov distance of the U from Uniform(0, 1).
    pub ks_d: f64,
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

/// The LZ78 phrase count of `2^k` bits drawn from `rng`, and the number of
/// words drawn.
///
/// # Panics
/// Panics if `k` is outside `3..=28`, `s` is outside `1..=32`, or
/// `r + s > 32`.
pub fn phrase_count(rng: &mut impl Rng, k: usize, r: usize, s: usize) -> (usize, usize) {
    assert!((3..=28).contains(&k), "k must be in 3..=28");
    assert!(s > 0 && s <= 32, "s must be in 1..=32");
    assert!(r <= 32 && r + s <= 32, "r + s must be <= 32");

    let n_bits = 1usize << k;
    let words = n_bits.div_ceil(s);
    let blocks: Vec<u32> = (0..words).map(|_| strip_b(rng.next_u32(), r, s)).collect();
    (lz78_count_blocks(&blocks, n_bits, s), words)
}

/// The largest supported replication count for `k`: unlimited for an exact
/// table, a hundredth of the replications behind a simulated one, and zero
/// when there is no table.
pub fn max_replications(k: usize) -> usize {
    match PHRASE_COUNTS.iter().find(|t| t.k == k) {
        None => 0,
        Some(t) if t.exact => usize::MAX,
        Some(t) => (t.counts.iter().sum::<u64>() / 100) as usize,
    }
}

/// U = F(w − 1) + v·P(w) under the table for `k`; NaN for a count an exact
/// table rules out.
fn randomized_pit(k: usize, w: usize, v: f64) -> f64 {
    let table = PHRASE_COUNTS
        .iter()
        .find(|t| t.k == k)
        .expect("a table for k");
    let total: u64 = table.counts.iter().sum();
    let pseudo = u64::from(!table.exact);
    let denominator = (total + 2 * pseudo) as f64;
    let w_max = table.w_min + table.counts.len() - 1;
    let (below, atom) = if w < table.w_min {
        (0, pseudo)
    } else if w > w_max {
        (total + pseudo, pseudo)
    } else {
        let i = w - table.w_min;
        (
            pseudo + table.counts[..i].iter().sum::<u64>(),
            table.counts[i],
        )
    };
    if atom == 0 && table.exact {
        return f64::NAN;
    }
    (below as f64 + v * atom as f64) / denominator
}

/// V uniform on (0, 1): 53 bits, offset by half a step from both ends.
fn open_unit(rng: &mut Pcg64) -> f64 {
    ((rng.next_u64() >> 11) as f64 + 0.5) * (1.0 / (1u64 << 53) as f64)
}

/// Run `replications` Lempel-Ziv replications on `rng`, drawing V from a
/// PCG64 generator seeded with `pit_seed`, and aggregate them.
///
/// A replication count the table cannot support runs nothing, and the
/// summary records why.
///
/// # Panics
/// Panics if `replications == 0`, or on the parameter violations of
/// [`phrase_count`].
pub fn lempel_ziv_summary(
    rng: &mut impl Rng,
    replications: usize,
    k: usize,
    r: usize,
    s: usize,
    pit_seed: u64,
) -> (Vec<LempelZivReplication>, LempelZivSummary) {
    assert!(replications > 0, "replications must be positive");
    let mut summary = LempelZivSummary {
        k,
        r,
        s,
        replications,
        pit_seed,
        unsupported: None,
        z_sum_stat: f64::NAN,
        z_sum_p_value: f64::NAN,
        ks_p_value: f64::NAN,
        ks_d: f64::NAN,
    };
    let limit = max_replications(k);
    if replications > limit {
        summary.unsupported = Some(if limit == 0 {
            format!("no phrase-count table for k={k}")
        } else {
            format!(
                "N={replications} exceeds {limit}, a hundredth of the simulated table for k={k}"
            )
        });
        return (Vec::new(), summary);
    }

    let mut v_rng = Pcg64::new(u128::from(pit_seed), PIT_STREAM);
    let reps: Vec<LempelZivReplication> = (0..replications)
        .map(|_| {
            let (phrase_count, words) = phrase_count(rng, k, r, s);
            let uniform = randomized_pit(k, phrase_count, open_unit(&mut v_rng));
            LempelZivReplication {
                words,
                phrase_count,
                uniform,
                z_score: normal_quantile(uniform),
            }
        })
        .collect();

    summary.z_sum_stat =
        reps.iter().map(|rep| rep.z_score).sum::<f64>() / (replications as f64).sqrt();
    summary.z_sum_p_value = erfc(summary.z_sum_stat.abs() / SQRT_2).min(1.0);
    let mut uniforms: Vec<f64> = reps.iter().map(|rep| rep.uniform).collect();
    summary.ks_p_value = if uniforms.iter().any(|u| u.is_nan()) {
        f64::NAN
    } else {
        ks_test(&mut uniforms)
    };
    summary.ks_d = crate::math::ks_statistic(&mut uniforms);
    (reps, summary)
}

fn parameters(summary: &LempelZivSummary) -> String {
    format!(
        "N={}, k={}, r={}, s={}, pit_seed={}",
        summary.replications, summary.k, summary.r, summary.s, summary.pit_seed
    )
}

/// The Z statistic from `summary` as a [`TestResult`] named
/// `testu01::lzw_sum`.
pub fn lempel_ziv_sum_result(summary: &LempelZivSummary) -> TestResult {
    match &summary.unsupported {
        Some(why) => TestResult::unsupported("testu01::lzw_sum", why),
        None => TestResult::with_note(
            "testu01::lzw_sum",
            summary.z_sum_p_value,
            format!("{}, Z={:.4}", parameters(summary), summary.z_sum_stat),
        )
        .normal(summary.z_sum_stat),
    }
}

/// The Kolmogorov–Smirnov test from `summary` as a [`TestResult`] named
/// `testu01::lzw_ks`.
pub fn lempel_ziv_ks_result(summary: &LempelZivSummary) -> TestResult {
    match &summary.unsupported {
        Some(why) => TestResult::unsupported("testu01::lzw_ks", why),
        None => TestResult::with_note("testu01::lzw_ks", summary.ks_p_value, parameters(summary))
            .kolmogorov_smirnov(summary.ks_d, summary.replications),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        lempel_ziv_ks_result, lempel_ziv_sum_result, lempel_ziv_summary, lz78_count_blocks,
        max_replications, phrase_count, randomized_pit, trie_reservation, PHRASE_COUNTS,
    };
    use crate::rng::{ConstantRng, Rng, Xorshift32};

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
            let (count, drawn) = phrase_count(&mut rng, k, r, s);
            assert_eq!(words, drawn, "k = {k}, r = {r}, s = {s}");
            assert_eq!(phrases, count, "k = {k}, r = {r}, s = {s}");
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
            let (reps, summary) = lempel_ziv_summary(&mut rng, replications, k, r, s, 1);
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

    /// Counts of W over every string of 2^k bits, one `k`-bit-wide field per
    /// string.
    fn enumerate(k: usize) -> Vec<(usize, u64)> {
        let bits = 1usize << k;
        let mut hist = std::collections::BTreeMap::new();
        for x in 0..1u64 << bits {
            let word = (x << (32 - bits)) as u32;
            *hist
                .entry(lz78_count_blocks(&[word], bits, 32))
                .or_insert(0u64) += 1;
        }
        hist.into_iter().collect()
    }

    fn table(k: usize) -> Vec<(usize, u64)> {
        let t = PHRASE_COUNTS.iter().find(|t| t.k == k).unwrap();
        (t.w_min..).zip(t.counts.iter().copied()).collect()
    }

    /// The exact tables for k = 3 and 4 are the enumeration.
    #[test]
    fn exact_tables_match_enumeration() {
        for k in [3, 4] {
            assert_eq!(enumerate(k), table(k), "k = {k}");
        }
    }

    /// The same for k = 5, 2³² strings (minutes in release).
    #[test]
    #[ignore]
    fn exact_table_matches_enumeration_at_k5() {
        assert_eq!(enumerate(5), table(5));
    }

    /// Tables run from k = 3 upward without gaps, with nonzero end cells.
    #[test]
    fn tables_are_contiguous() {
        for (i, t) in PHRASE_COUNTS.iter().enumerate() {
            assert_eq!(t.k, i + 3);
            assert_eq!(t.exact, t.k <= 5);
            assert!(
                t.counts[0] > 0 && *t.counts.last().unwrap() > 0,
                "k = {}",
                t.k
            );
        }
    }

    /// Consecutive counts map onto adjacent intervals that tile [0, 1], so a
    /// uniform V makes U uniform.  A simulated table's pseudo-replications
    /// cover the ends; a count an exact table excludes is NaN.
    #[test]
    fn randomized_pit_tiles_the_unit_interval() {
        for t in PHRASE_COUNTS {
            let (lo, hi) = (t.w_min, t.w_min + t.counts.len() - 1);
            let mut edge = randomized_pit(t.k, lo, 0.0);
            for w in lo..=hi {
                assert!(
                    (randomized_pit(t.k, w, 0.0) - edge).abs() < 1e-15,
                    "k = {}",
                    t.k
                );
                edge = randomized_pit(t.k, w, 1.0);
            }
            if t.exact {
                assert_eq!(randomized_pit(t.k, lo, 0.0), 0.0);
                assert!((edge - 1.0).abs() < 1e-15, "k = {}", t.k);
                assert!(randomized_pit(t.k, lo - 1, 0.5).is_nan());
                assert!(randomized_pit(t.k, hi + 1, 0.5).is_nan());
            } else {
                assert_eq!(
                    randomized_pit(t.k, lo - 1, 1.0),
                    randomized_pit(t.k, lo, 0.0)
                );
                assert_eq!(randomized_pit(t.k, hi + 1, 0.0), edge);
                assert!((randomized_pit(t.k, hi + 9, 1.0) - 1.0).abs() < 1e-15);
                assert_eq!(randomized_pit(t.k, 0, 0.0), 0.0);
            }
        }
    }

    /// Exact tables admit any N; simulated ones a hundredth of their size.
    #[test]
    fn replication_limits() {
        assert_eq!(max_replications(3), usize::MAX);
        assert_eq!(max_replications(6), 10_000);
        assert_eq!(max_replications(17), 10_000);
        assert_eq!(max_replications(28), 1_000);
        assert_eq!(max_replications(29), 0);
    }

    /// An unsupported N draws nothing and reports insufficient data.
    #[test]
    fn unsupported_replications() {
        let mut rng = Xorshift32::new(XORSHIFT_SEED);
        let (reps, summary) = lempel_ziv_summary(&mut rng, 10_001, 6, 0, 32, 1);
        assert!(reps.is_empty());
        assert!(lempel_ziv_sum_result(&summary).is_unsupported());
        assert!(lempel_ziv_ks_result(&summary).is_unsupported());
        let mut fresh = Xorshift32::new(XORSHIFT_SEED);
        assert_eq!(fresh.next_u32(), rng.next_u32());
    }

    /// A stream of zero bits is far less complex than the table allows.
    #[test]
    fn zero_stream_fails() {
        let (_, summary) = lempel_ziv_summary(&mut ConstantRng::new(0), 20, 10, 0, 32, 1);
        assert!(lempel_ziv_ks_result(&summary).failed());
        assert!(lempel_ziv_sum_result(&summary).failed());
    }
}
