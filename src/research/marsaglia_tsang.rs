//! Marsaglia–Tsang "difficult tests" research implementations.
//!
//! This module currently implements the Gorilla test described in:
//!
//! George Marsaglia and Wai Wan Tsang, "Some difficult-to-pass tests of
//! randomness", Journal of Statistical Software 7(3), 2002
//! (`marsaglia2002difficult` in BIB.md).
//! [pubs/marsaglia-tsang-2002-difficult-tests.pdf]  The article's attached
//! C code is `gorilla()` and `ad32()` in `tuftests.c`.
//! [pubs/marsaglia-tsang-2002-tuftests.c]
//!
//! The paper's Gorilla test (pp. 5–6):
//! - selects one bit position from each 32-bit output word
//! - forms a bit stream of length 2^26 + 25
//! - counts how many 26-bit words are missing from the 2^26 overlapping windows
//! - compares the missing-word count to a normal approximation with
//!   mean 24,687,971 and standard deviation 4,170
//! - then applies an Anderson–Darling–Kolmogorov–Smirnov ("ADKS") test to
//!   the 32 per-bit p-values, to catch generators whose problem is collective
//!   non-uniformity across bit positions rather than one spectacularly bad bit
//!
//! The paper does not define ADKS.  `tuftests.c` computes the Anderson–Darling
//! statistic A₃₂ of the 32 p-values, with each product uᵢ(1 − u₃₁₋ᵢ) floored
//! at 10⁻³⁰ before its logarithm, and prints Pr(A₃₂ < A), labelled a KS test.
//! [`gorilla_aggregate_ad`] computes that statistic.  It converts the statistic
//! with [`crate::math::anderson_darling_cdf`], the Anderson–Darling
//! distribution the first author published two years later (Marsaglia and
//! Marsaglia 2004), rather than `tuftests.c`'s four-piece fit `ad32()` for
//! n = 32, which departs from that distribution by up to 0.0056 (at A = 1).
//! From the per-bit values printed on p. 6, this reproduces the ADKS printed
//! for KISS (0.115), SHR3 (0.937, which only the floor allows), LFIB4 (0.724,
//! where `ad32()` gives 0.727) and both congruential generators (1.000), to
//! within the four-decimal rounding of those inputs.  A Kolmogorov–Smirnov
//! test would give 0.052 for KISS and 0.587 for LFIB4.
//!
//! `tuftests.c` works in single precision and evaluates Φ with a three-term
//! rational approximation; this module uses double precision and
//! [`crate::math::normal_cdf`].  The reproduction above holds for the per-bit
//! values the paper prints, not for every z.  In single precision Φ(z) rounds
//! to 1 once z exceeds about 5.4, and the product it enters meets the 10⁻³⁰
//! floor, whereas this module's double-precision p-values saturate only
//! beyond |z| ≈ 8.3.  With SHR3's bits 2 and 31 at z = +6 and −6, for example,
//! `tuftests.c` gives A = 2.302 and ADKS 0.937, and this module gives
//! A = 1.439 and ADKS 0.808.
//!
//! Word use also departs from `tuftests.c`.  [`gorilla_all`] reads all 32 bit
//! positions from the same 2²⁶ + 25 words, whereas `gorilla()` in
//! `tuftests.c` calls the generator inside its loop over bit positions, so
//! each position gets 2²⁶ + 25 fresh numbers, 32 · (2²⁶ + 25) ≈ 2.15·10⁹ in
//! all.  Sharing words divides the generator output a test needs, and the
//! time to produce it, by 32; the `gorilla` binary holds its words at once,
//! which would otherwise take 8 GiB.  Under the null hypothesis the choice
//! changes nothing.  The words are iid uniform, so the bits at different
//! positions of one word are independent, and the 32 per-position bit
//! streams are independent whether or not they share words.  The 32
//! missing-word counts, and so the per-bit p-values the Anderson–Darling
//! aggregate treats as independent, have the same joint null distribution
//! as with fresh words.  The aggregate's null distribution is unchanged.
//! Under an alternative the schemes do differ: dependence between the bits
//! of one word correlates this module's per-position results, and the
//! generator is judged on one stretch of output rather than 32.  Results are
//! therefore not comparable with `tuftests.c`'s run for run.

use crate::math::{anderson_darling_cdf, ks_test, normal_cdf};

const GORILLA_WORD_BITS: usize = 26;
const GORILLA_WINDOWS: usize = 1 << GORILLA_WORD_BITS;
const GORILLA_STREAM_BITS: usize = GORILLA_WINDOWS + GORILLA_WORD_BITS - 1;
const GORILLA_MISSING_MEAN: f64 = 24_687_971.0;
const GORILLA_MISSING_STDDEV: f64 = 4_170.0;
/// Floor `tuftests.c` puts under each product uᵢ(1 − uₙ₋₁₋ᵢ) before taking
/// its logarithm, so that a p-value of exactly 0 or 1 cannot make A infinite.
const GORILLA_AD_PRODUCT_FLOOR: f64 = 1e-30;

fn bit_is_set(words: &[u32], word_index: usize, bit_position_from_msb: usize) -> bool {
    let shift = 31usize.saturating_sub(bit_position_from_msb);
    ((words[word_index] >> shift) & 1) != 0
}

fn set_seen(seen: &mut [u64], value: usize) -> bool {
    let idx = value / 64;
    let bit = value % 64;
    let mask = 1u64 << bit;
    let was_set = (seen[idx] & mask) != 0;
    if !was_set {
        seen[idx] |= mask;
    }
    !was_set
}

fn missing_words_for_bit(words: &[u32], bit_position_from_msb: usize, word_bits: usize) -> usize {
    assert!((1..=31).contains(&word_bits), "word_bits must be in 1..=31");
    let windows = 1usize << word_bits;
    let stream_bits = windows + word_bits - 1;
    assert!(
        words.len() >= stream_bits,
        "not enough source words for Gorilla stream"
    );
    assert!(bit_position_from_msb < 32, "bit position must be in 0..32");

    let mut seen = vec![0u64; windows.div_ceil(64)];
    let mask = windows - 1;
    let mut rolling = 0usize;
    let mut seen_count = 0usize;

    for idx in 0..stream_bits {
        let bit = usize::from(bit_is_set(words, idx, bit_position_from_msb));
        rolling = ((rolling << 1) & mask) | bit;
        if idx + 1 >= word_bits && set_seen(&mut seen, rolling) {
            seen_count += 1;
        }
    }
    windows - seen_count
}

/// One Marsaglia–Tsang Gorilla result for a single bit position.
#[derive(Debug, Clone)]
pub struct GorillaBitResult {
    /// Bit position tested, `0..32` counted from the most-significant bit.
    pub bit_position: usize,
    /// Number of 26-bit words absent from the 2^26 overlapping windows.
    pub missing_words: usize,
    /// Normal z-score of `missing_words` (mean 24 687 971, σ 4 170).
    pub z_score: f64,
    /// Upper-tail p-value `1 − Φ(z)`: small when too many words are missing.
    pub p_value: f64,
}

/// Run the full 32-bit-position Gorilla test.
///
/// Bit positions are numbered 0..31 from most-significant to least-significant,
/// matching the paper.
///
/// # Panics
/// Panics if `words` has fewer than 2^26 + 25 elements (the Gorilla
/// stream length).
pub fn gorilla_all(words: &[u32]) -> Vec<GorillaBitResult> {
    assert!(
        words.len() >= GORILLA_STREAM_BITS,
        "gorilla_all requires at least 2^26 + 25 source words"
    );
    (0..32)
        .map(|bit_position| {
            let missing_words = missing_words_for_bit(words, bit_position, GORILLA_WORD_BITS);
            let z_score = (missing_words as f64 - GORILLA_MISSING_MEAN) / GORILLA_MISSING_STDDEV;
            GorillaBitResult {
                bit_position,
                missing_words,
                z_score,
                // Upper-tail (one-sided) p-value: the test targets generators
                // with too many missing words (sparse coverage); over-uniform
                // generators (z << 0) pass silently.  Note the paper itself
                // reports the LOWER tail Φ(z) — failures there approach 1;
                // the 1 − Φ(z) flip adapts it to this crate's small-p-fails
                // convention (the Anderson–Darling aggregate is invariant
                // under p ↦ 1 − p).
                p_value: 1.0 - normal_cdf(z_score),
            }
        })
        .collect()
}

/// Marsaglia and Tsang's second-stage "ADKS" check on the per-bit p-values.
#[derive(Debug, Clone, Copy)]
pub struct GorillaAggregate {
    /// Anderson–Darling statistic Aₙ of the `n` per-bit p-values, with each
    /// product floored at 10⁻³⁰ as in `tuftests.c`.
    pub statistic: f64,
    /// Pr(Aₙ < `statistic`), the value the paper prints as ADKS: near 1 when
    /// the per-bit p-values are far from uniform.  NaN for fewer than eight
    /// per-bit results, which [`crate::math::anderson_darling_cdf`] does not
    /// cover.
    pub adks: f64,
    /// `1 − adks`, small when the per-bit p-values are far from uniform, in
    /// this crate's small-p-fails convention.  For n = 32 it is
    /// within 3.5% of simulation, and on the high side, for 4 < A ≤ 12 (see
    /// [`crate::math::anderson_darling_cdf`]).
    pub p_value: f64,
}

/// Aggregate the per-bit results of [`gorilla_all`] as Marsaglia and Tsang
/// (2002, p. 6) do: an Anderson–Darling test of the p-values for uniformity
/// on [0, 1] (see the module docs).
///
/// A small `p_value` means the per-bit p-values are not uniformly
/// distributed, which indicates systematic non-randomness spread across bit
/// positions rather than an isolated bad bit.  Returns NaN in every field if
/// `results` is empty or holds a NaN p-value; with fewer than eight results
/// only `statistic` is defined.
pub fn gorilla_aggregate_ad(results: &[GorillaBitResult]) -> GorillaAggregate {
    let mut u: Vec<f64> = results.iter().map(|r| r.p_value).collect();
    if u.is_empty() || u.iter().any(|p| p.is_nan()) {
        return GorillaAggregate {
            statistic: f64::NAN,
            adks: f64::NAN,
            p_value: f64::NAN,
        };
    }
    u.sort_by(f64::total_cmp);
    let n = u.len();
    let log_sum: f64 = (0..n)
        .map(|i| {
            let product = (u[i] * (1.0 - u[n - 1 - i])).max(GORILLA_AD_PRODUCT_FLOOR);
            (2 * i + 1) as f64 * product.ln()
        })
        .sum();
    let statistic = -(n as f64) - log_sum / n as f64;
    let adks = anderson_darling_cdf(n, statistic);
    GorillaAggregate {
        statistic,
        adks,
        p_value: 1.0 - adks,
    }
}

/// Kolmogorov–Smirnov uniformity check on the per-bit p-values from
/// [`gorilla_all`], the aggregate this crate reported before
/// [`gorilla_aggregate_ad`].  Kept, deprecated, so code written against 0.5.0
/// still compiles.
///
/// Returns the two-sided KS p-value of the `p_value` fields
/// ([`crate::math::ks_test`]), which is not the paper's Anderson–Darling
/// ("ADKS") aggregate (see the module docs).
#[deprecated(note = "use gorilla_aggregate_ad, Marsaglia and Tsang's Anderson-Darling aggregate")]
pub fn gorilla_aggregate_ks(results: &[GorillaBitResult]) -> f64 {
    let mut pvals: Vec<f64> = results.iter().map(|r| r.p_value).collect();
    ks_test(&mut pvals)
}

#[cfg(test)]
mod tests {
    use super::{
        gorilla_aggregate_ad, missing_words_for_bit, GorillaBitResult, GORILLA_MISSING_MEAN,
        GORILLA_MISSING_STDDEV,
    };
    use crate::math::{ks_test, normal_cdf};

    // Per-bit p-values Φ(z) printed by Marsaglia and Tsang (2002, p. 6), bits
    // 0 to 31, above the ADKS value for each generator.
    /// KISS.
    const KISS: [f64; 32] = [
        0.6330, 0.2903, 0.6350, 0.7377, 0.1342, 0.6095, 0.1959, 0.3699, 0.4194, 0.9699, 0.3807,
        0.4496, 0.9106, 0.9100, 0.4753, 0.8187, 0.3225, 0.2455, 0.7300, 0.9907, 0.0483, 0.8786,
        0.3932, 0.9093, 0.0975, 0.2096, 0.5962, 0.3991, 0.2822, 0.4591, 0.6845, 0.1816,
    ];
    /// SHR3, `jsr ^= jsr << 13; jsr ^= jsr >> 17; jsr ^= jsr << 5`.
    const SHR3: [f64; 32] = [
        0.1301, 0.9562, 1.0000, 0.5639, 0.3534, 0.5053, 0.5459, 0.7153, 0.6989, 0.5975, 0.5579,
        0.2299, 0.4949, 0.4399, 0.3033, 0.2713, 0.4054, 0.0514, 0.9929, 0.5981, 0.6724, 0.3801,
        0.2743, 0.0367, 0.5239, 0.5100, 0.1128, 0.8865, 0.8057, 0.8623, 0.9569, 0.0000,
    ];
    /// LFIB4, x(n) = x(n−256) + x(n−179) + x(n−119) + x(n−55) mod 2³².
    const LFIB4: [f64; 32] = [
        0.7726, 0.6625, 0.8484, 0.6311, 0.5161, 0.4235, 0.3163, 0.0502, 0.0928, 0.6614, 0.0078,
        0.2021, 0.6616, 0.0149, 0.5762, 0.5736, 0.4923, 0.6725, 0.5489, 0.1335, 0.8364, 0.2657,
        0.0169, 0.7038, 0.5774, 0.7989, 0.6508, 0.4192, 0.2158, 0.6698, 0.8185, 0.2468,
    ];
    /// x(n) = 69069·x(n−1) mod 2³² + 91, a prime modulus.
    const LCG_PRIME_MODULUS: [f64; 32] = [
        0.0309, 0.0211, 0.0260, 0.0150, 0.0181, 0.0002, 0.0202, 0.0162, 0.3523, 0.0018, 0.0013,
        0.4848, 0.0345, 0.0044, 0.0008, 0.0076, 0.0227, 0.0012, 0.0018, 0.0743, 0.0821, 0.0001,
        0.0118, 0.4472, 0.3204, 0.5452, 0.3325, 0.4709, 0.7966, 0.5493, 0.0068, 0.3143,
    ];
    /// x(n) = 214013·x + 2531011 mod 2³², the paper's "gcc's rand function".
    const GCC_RAND: [f64; 32] = [
        0.6429, 0.0000, 0.0000, 0.0000, 0.0000, 0.0000, 0.0000, 1.0000, 1.0000, 1.0000, 1.0000,
        1.0000, 1.0000, 1.0000, 1.0000, 1.0000, 1.0000, 1.0000, 1.0000, 1.0000, 1.0000, 1.0000,
        1.0000, 1.0000, 1.0000, 1.0000, 1.0000, 1.0000, 1.0000, 1.0000, 1.0000, 1.0000,
    ];

    fn bit_results(p_values: &[f64]) -> Vec<GorillaBitResult> {
        p_values
            .iter()
            .enumerate()
            .map(|(bit_position, &p_value)| GorillaBitResult {
                bit_position,
                missing_words: 0,
                z_score: f64::NAN,
                p_value,
            })
            .collect()
    }

    /// From the per-bit values printed on p. 6 the aggregate reproduces each
    /// printed ADKS to within 10⁻³, the slack the four-decimal rounding of
    /// the inputs allows.  The statistic and Pr(A₃₂ < A) are also pinned to
    /// `tuftests.c`'s computation [pubs/marsaglia-tsang-2002-tuftests.c],
    /// redone in double precision and converted by ADinf + errfix from the
    /// attachments `ADinf.c` [pubs/marsaglia-marsaglia-2004-ADinf.c] and
    /// `AnDarl.c` [pubs/marsaglia-marsaglia-2004-AnDarl.c]; the two
    /// congruential generators' statistics exceed 30, where ADinf is 1.
    #[test]
    fn aggregate_reproduces_the_adks_values_printed_in_the_paper() {
        for (name, table, printed, statistic, adks) in [
            (
                "KISS",
                &KISS,
                0.115,
                0.362_114_004_714_591_43,
                0.115_507_015_767_071_01,
            ),
            (
                "SHR3",
                &SHR3,
                0.937,
                2.301_865_320_405_241_3,
                0.936_544_387_575_710_7,
            ),
            (
                "LFIB4",
                &LFIB4,
                0.724,
                1.178_105_090_677_540_5,
                0.724_337_845_482_796_6,
            ),
            (
                "69069",
                &LCG_PRIME_MODULUS,
                1.000,
                44.109_952_347_440_41,
                1.0,
            ),
            ("gcc", &GCC_RAND, 1.000, 1_318.812_100_133_806_6, 1.0),
        ] {
            let aggregate = gorilla_aggregate_ad(&bit_results(table));
            assert!(
                (aggregate.statistic - statistic).abs() <= 1e-12 * statistic,
                "{name}: A = {}",
                aggregate.statistic
            );
            assert!(
                (aggregate.adks - adks).abs() < 1e-12,
                "{name}: Pr(A < z) = {}",
                aggregate.adks
            );
            assert!(
                (aggregate.adks - printed).abs() < 1e-3,
                "{name}: printed {printed}"
            );
            assert_eq!(1.0 - aggregate.adks, aggregate.p_value, "{name}");
        }

        // SHR3's bits 2 and 31 print as 1.0000 and 0.0000.  Taken as exact,
        // they make one product 0, and only the floor keeps A finite; without
        // it Pr(A < z) would be 1, not 0.937.
        let unfloored: f64 = {
            let mut u = SHR3.to_vec();
            u.sort_by(f64::total_cmp);
            (0..32)
                .map(|i| (2 * i + 1) as f64 * (u[i] * (1.0 - u[31 - i])).ln())
                .sum()
        };
        assert_eq!(f64::NEG_INFINITY, unfloored);

        // A Kolmogorov–Smirnov test on the same values gives Pr(D < d) = 0.052
        // for KISS and 0.587 for LFIB4, not the printed 0.115 and 0.724.
        for (table, ks_cdf) in [(&KISS, 0.052), (&LFIB4, 0.587)] {
            let mut values = table.to_vec();
            let got = 1.0 - ks_test(&mut values);
            assert!((got - ks_cdf).abs() < 1e-3, "Pr(D < d) = {got}");
        }
    }

    /// `tuftests.c` stores Φ(z) in a float, which is 1 above z ≈ 5.4; this
    /// module's p-values are doubles.  SHR3's printed values with bits 2 and
    /// 31 at z = +6 and −6: `tuftests.c`'s float path, redone in C, gives
    /// A = 2.302 and ADKS 0.937 (like the printed 1.0000 and 0.0000), and a
    /// double-precision replica gives the values pinned here.
    #[test]
    fn aggregate_departs_from_single_precision_beyond_z_5_4() {
        assert!((normal_cdf(5.3) as f32) < 1.0);
        assert_eq!(1.0, normal_cdf(5.5) as f32);
        let mut results = bit_results(&SHR3.map(|phi| 1.0 - phi));
        for (bit, z) in [(2, 6.0), (31, -6.0)] {
            results[bit].z_score = z;
            results[bit].p_value = 1.0 - normal_cdf(z);
        }
        let aggregate = gorilla_aggregate_ad(&results);
        assert!(
            (aggregate.statistic - 1.439_24).abs() < 1e-4,
            "A = {}",
            aggregate.statistic
        );
        assert!(
            (aggregate.adks - 0.808_295).abs() < 1e-4,
            "Pr(A < z) = {}",
            aggregate.adks
        );
    }

    /// This crate's per-bit p-values are 1 − Φ(z), the paper's Φ(z).  Aₙ is
    /// unchanged by u ↦ 1 − u, floor included.
    #[test]
    fn aggregate_is_invariant_under_reflected_p_values() {
        for table in [&KISS, &SHR3, &LFIB4] {
            let direct = gorilla_aggregate_ad(&bit_results(table));
            let reflected: Vec<f64> = table.iter().map(|p| 1.0 - p).collect();
            let reflected = gorilla_aggregate_ad(&bit_results(&reflected));
            assert!((direct.statistic - reflected.statistic).abs() < 1e-12 * direct.statistic);
            assert!((direct.adks - reflected.adks).abs() < 1e-12);
        }
    }

    #[test]
    fn aggregate_without_p_values_is_nan() {
        assert!(gorilla_aggregate_ad(&[]).p_value.is_nan());
        // Seven results: A is defined, its distribution is not.
        let few = gorilla_aggregate_ad(&bit_results(&KISS[..7]));
        assert!(few.statistic.is_finite());
        assert!(few.adks.is_nan());
        assert!(few.p_value.is_nan());
        let mut results = bit_results(&KISS);
        results[3].p_value = f64::NAN;
        let aggregate = gorilla_aggregate_ad(&results);
        assert!(aggregate.statistic.is_nan());
        assert!(aggregate.adks.is_nan());
        assert!(aggregate.p_value.is_nan());
    }

    /// The aggregate is an exact two-sided KS test on the `p_value` fields;
    /// z-scores play no part.  Reference from R 4.2.0,
    /// `ks.test(((0:31) + 0.5)^2 / 1024, "punif", exact = TRUE)`:
    /// D = 0.265380859375, p = 0.01768314058013587.  Reflecting every p-value
    /// to 1 − p gives the same D and p in R.
    #[test]
    #[allow(deprecated)]
    fn deprecated_ks_aggregate_is_exact_ks_on_per_bit_p_values() {
        let results = |reflect: bool| -> Vec<GorillaBitResult> {
            (0..32)
                .map(|bit_position| {
                    let u = (bit_position as f64 + 0.5) / 32.0;
                    let p = u * u;
                    GorillaBitResult {
                        bit_position,
                        missing_words: 0,
                        z_score: f64::NAN,
                        p_value: if reflect { 1.0 - p } else { p },
                    }
                })
                .collect()
        };
        for reflect in [false, true] {
            let p = super::gorilla_aggregate_ks(&results(reflect));
            assert!(
                (p - 0.017_683_140_580_135_87).abs() < 1e-9,
                "reflect = {reflect}: p = {p}"
            );
        }
    }

    #[test]
    fn alternating_bit_stream_misses_all_but_two_patterns_for_small_word_size() {
        let words = (0..20)
            .map(|i| if i % 2 == 0 { 0xaaaa_aaaa } else { 0x5555_5555 })
            .collect::<Vec<_>>();
        let missing = missing_words_for_bit(&words, 0, 3);
        assert_eq!(6, missing);
    }

    #[test]
    fn constant_zero_stream_hits_exactly_one_pattern_for_small_word_size() {
        let words = vec![0u32; 20];
        let missing = missing_words_for_bit(&words, 5, 4);
        assert_eq!(15, missing);
    }

    #[test]
    fn gorilla_p_value_uses_upper_tail_for_excess_missing_words() {
        let z_score = 3.0;
        let result = GorillaBitResult {
            bit_position: 0,
            missing_words: (GORILLA_MISSING_MEAN + 3.0 * GORILLA_MISSING_STDDEV) as usize,
            z_score,
            p_value: 1.0 - normal_cdf(z_score),
        };
        assert!(
            result.p_value < 0.01,
            "excess missing words should yield a small p-value"
        );
    }
}
