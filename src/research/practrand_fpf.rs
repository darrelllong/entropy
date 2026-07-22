//! PractRand FPF core test from `src/tests.cpp`.
//!
//! Reference:
//! - PractRand pre-0.95, `include/PractRand/Tests/FPF.h`
//! - PractRand pre-0.95, `src/tests.cpp` (`PractRand::Tests::FPF`)
//!
//! This ports the core bucketing/test logic:
//! - parse the LSB-first bitstream into FPF codewords: a run of zeros
//!   terminated by a stop bit (the geometric exponent, capped at `max_exp`),
//!   then `sig_bits` of significand
//! - apply PractRand's intra-platter truncation rule and G-test
//! - apply a grouped exponent-distribution G-test (`:cross`)
//!
//! Deliberate deviation from PractRand: upstream slides its sample window a
//! fixed 16-bit stride, so successive samples share examined bits whenever
//! the exponent is ≥ 2 — a dependence its empirical calibration tables absorb.
//! This port has no calibration tables, so it parses *disjoint* codewords
//! instead: samples are iid, the asymptotic chi-square/G law it quotes is
//! actually valid, and mean consumption (≈ 16 bits per sample at the default
//! `sig_bits = 14`) matches upstream's stride.  Suspicion scores are not
//! reproduced.

use crate::{math::igamc, result::TestResult, rng::Rng};

fn truncate_table_bits(counts: &mut [u64], probs: &mut [f64], old_bits: usize, new_bits: usize) {
    let ns = 1usize << new_bits;
    let os = 1usize << old_bits;
    for i in ns..os {
        let ni = i & (ns - 1);
        counts[ni] += counts[i];
        counts[i] = 0;
        probs[ni] += probs[i];
        probs[i] = 0.0;
    }
}

fn g_test(expected_probs: &[f64], observed: &[u64], total: usize) -> f64 {
    let total = total as f64;
    2.0 * expected_probs
        .iter()
        .zip(observed)
        .filter(|(p, &o)| **p > 0.0 && o > 0)
        .map(|(p, &o)| {
            let expected = total * *p;
            o as f64 * ((o as f64) / expected).ln()
        })
        .sum::<f64>()
}

fn chi_square_pvalue(chi_square: f64, dof: usize) -> f64 {
    if dof == 0 {
        return f64::NAN;
    }
    igamc(dof as f64 / 2.0, chi_square / 2.0)
}

/// Codeword-shape parameters for [`fpf_test`].
#[derive(Debug, Clone)]
pub struct FpfConfig {
    /// Significand width in bits (1..=20).
    pub sig_bits: usize,
    /// Exponent field width: exponents are capped at `2^exp_bits − 1`.
    pub exp_bits: usize,
}

impl Default for FpfConfig {
    fn default() -> Self {
        Self {
            sig_bits: 14,
            exp_bits: 6,
        }
    }
}

/// Per-exponent ("platter") G-test outcome over significand values.
#[derive(Debug, Clone)]
pub struct FpfPlatterSummary {
    /// Codeword exponent (leading-zero run length) this platter collects.
    pub exponent: usize,
    /// Significand bits actually tested after PractRand's truncation rule.
    pub effective_sig_bits: usize,
    /// G-test statistic over the `2^effective_sig_bits` significand bins.
    pub chi_square: f64,
    /// Degrees of freedom (bins − 1).
    pub dof: usize,
    /// Chi-square survival p-value of the G statistic; NaN when `dof == 0`.
    pub p_value: f64,
    /// Number of codewords that landed in this platter.
    pub samples: usize,
}

/// Full outcome of [`fpf_test`]: per-platter results plus the cross
/// (exponent-distribution) G-test.
#[derive(Debug, Clone)]
pub struct FpfSummary {
    /// Bit budget the test was given.
    pub total_bits: usize,
    /// Bits actually consumed by the parsed codewords (≤ `total_bits`).
    pub consumed_bits: usize,
    /// Configured significand width in bits.
    pub sig_bits: usize,
    /// Exponent cap, `2^exp_bits − 1`.
    pub max_exp: usize,
    /// Total number of disjoint codewords parsed.
    pub samples: usize,
    /// Intra-platter G-test results; platters whose expected sample count
    /// is too small are omitted.
    pub platter_results: Vec<FpfPlatterSummary>,
    /// Grouped G-test statistic over the exponent distribution.
    pub cross_chi_square: f64,
    /// Degrees of freedom of the cross test (merged cells − 1).
    pub cross_dof: usize,
    /// Chi-square survival p-value of the cross test; NaN when
    /// `cross_dof == 0`.
    pub cross_p_value: f64,
}

fn next_lsb_bit(rng: &mut impl Rng, current_word: &mut u32, bits_left: &mut usize) -> u8 {
    if *bits_left == 0 {
        *current_word = rng.next_u32();
        *bits_left = 32;
    }
    let bit = (*current_word & 1) as u8;
    *current_word >>= 1;
    *bits_left -= 1;
    bit
}

/// Parse one FPF codeword from the LSB-first bitstream: a run of zeros
/// terminated by a stop bit (exponent, capped at `max_exp`, in which case no
/// stop bit is consumed), then `sig_bits` of significand.
/// Returns `(exponent, significand, bits_consumed)`.
fn parse_codeword(
    rng: &mut impl Rng,
    current_word: &mut u32,
    bits_left: &mut usize,
    sig_bits: usize,
    max_exp: usize,
) -> (usize, usize, usize) {
    let mut e = 0usize;
    while e < max_exp && next_lsb_bit(rng, current_word, bits_left) == 0 {
        e += 1;
    }
    let exp_bits_read = if e < max_exp { e + 1 } else { max_exp };
    let mut sig = 0usize;
    for j in 0..sig_bits {
        sig |= (next_lsb_bit(rng, current_word, bits_left) as usize) << j;
    }
    (e, sig, exp_bits_read + sig_bits)
}

fn grouped_tail_g_test(counts: &[u64], probs: &[f64], min_expected: f64) -> (f64, usize) {
    let total: usize = counts.iter().sum::<u64>() as usize;
    if total == 0 {
        return (f64::NAN, 0);
    }
    let mut merged_probs = Vec::new();
    let mut merged_counts = Vec::new();
    let mut run_prob = 0.0;
    let mut run_count = 0u64;
    for (&p, &c) in probs.iter().zip(counts) {
        run_prob += p;
        run_count += c;
        if run_prob * total as f64 >= min_expected {
            merged_probs.push(run_prob);
            merged_counts.push(run_count);
            run_prob = 0.0;
            run_count = 0;
        }
    }
    if run_prob > 0.0 {
        if let Some(last) = merged_probs.last_mut() {
            *last += run_prob;
        } else {
            merged_probs.push(run_prob);
        }
        if let Some(last) = merged_counts.last_mut() {
            *last += run_count;
        } else {
            merged_counts.push(run_count);
        }
    }
    let dof = merged_probs.len().saturating_sub(1);
    (g_test(&merged_probs, &merged_counts, total), dof)
}

/// Run the PractRand FPF test over `total_bits` bits drawn LSB-first from
/// `rng`, parsing disjoint codewords (see the module docs for the
/// deliberate deviation from upstream's sliding window).
///
/// # Panics
/// Panics if `config.sig_bits` is outside `1..=20` or `total_bits` cannot
/// hold one worst-case codeword (`2^exp_bits − 1 + sig_bits` bits).
pub fn fpf_test(rng: &mut impl Rng, total_bits: usize, config: &FpfConfig) -> FpfSummary {
    let max_exp = (1usize << config.exp_bits) - 1;
    // Longest possible codeword: max_exp zeros (no stop bit) + significand.
    let worst_codeword = max_exp + config.sig_bits;
    assert!(
        config.sig_bits > 0 && config.sig_bits <= 20,
        "sig_bits out of range"
    );
    assert!(
        total_bits >= worst_codeword,
        "total_bits must cover at least one worst-case FPF codeword"
    );

    let mut plateau_counts = vec![vec![0u64; 1usize << config.sig_bits]; max_exp + 1];
    let mut exp_counts = vec![0u64; max_exp + 1];

    let mut current_word = 0u32;
    let mut bits_left = 0usize;

    // Parse disjoint codewords while a worst-case codeword is guaranteed to
    // fit.  The stopping rule depends only on bits already consumed, never on
    // the codeword being parsed, so it introduces no sampling bias.
    let mut consumed = 0usize;
    let mut samples = 0usize;
    while consumed + worst_codeword <= total_bits {
        let (e, sig, used) = parse_codeword(
            rng,
            &mut current_word,
            &mut bits_left,
            config.sig_bits,
            max_exp,
        );
        plateau_counts[e][sig] += 1;
        exp_counts[e] += 1;
        samples += 1;
        consumed += used;
    }

    let mut platter_results = Vec::new();
    let intra_p = 1.0 / ((1usize << config.sig_bits) as f64);
    for e in 0..=max_exp {
        let expected =
            2f64.powi(-(e as i32 + 1 + if e == max_exp { -1 } else { 0 })) * samples as f64;
        let ebits_float = expected.log2() - 4.0;
        let mut ebits = (ebits_float * 0.75 + 0.1).floor() as isize;
        if ebits < 1 {
            continue;
        }
        if ebits as usize > config.sig_bits {
            ebits = config.sig_bits as isize;
        }
        let ebits = ebits as usize;
        let bins = 1usize << ebits;
        let mut counts = plateau_counts[e].clone();
        let mut probs = vec![intra_p; 1usize << config.sig_bits];
        if ebits < config.sig_bits {
            truncate_table_bits(&mut counts, &mut probs, config.sig_bits, ebits);
        }
        let chi = g_test(&probs[..bins], &counts[..bins], exp_counts[e] as usize);
        let dof = bins - 1;
        platter_results.push(FpfPlatterSummary {
            exponent: e,
            effective_sig_bits: ebits,
            chi_square: chi,
            dof,
            p_value: chi_square_pvalue(chi, dof),
            samples: exp_counts[e] as usize,
        });
    }

    let mut exp_probs = vec![0.0; max_exp + 1];
    for (e, p) in exp_probs.iter_mut().enumerate() {
        *p = 2f64.powi(-(e as i32 + 1 + if e == max_exp { -1 } else { 0 }));
    }
    let (cross_chi_square, cross_dof) = grouped_tail_g_test(&exp_counts, &exp_probs, 10.0);
    let cross_p_value = chi_square_pvalue(cross_chi_square, cross_dof);

    FpfSummary {
        total_bits,
        consumed_bits: consumed,
        sig_bits: config.sig_bits,
        max_exp,
        samples,
        platter_results,
        cross_chi_square,
        cross_dof,
        cross_p_value,
    }
}

/// Package the cross (exponent-distribution) G-test from `summary` as a
/// [`TestResult`] named `practrand::fpf_cross`.
pub fn fpf_cross_result(summary: &FpfSummary) -> TestResult {
    TestResult::with_note(
        "practrand::fpf_cross",
        summary.cross_p_value,
        format!(
            "samples={}, sig_bits={}, max_exp={}, dof={}, chi2={:.4}",
            summary.samples,
            summary.sig_bits,
            summary.max_exp,
            summary.cross_dof,
            summary.cross_chi_square
        ),
    )
}

/// Package one platter's intra-platter G-test as a [`TestResult`] named
/// `practrand::fpf_platter`.
pub fn fpf_platter_result(platter: &FpfPlatterSummary, summary: &FpfSummary) -> TestResult {
    TestResult::with_note(
        "practrand::fpf_platter",
        platter.p_value,
        format!(
            "samples={}, e={}, sig_bins=2^{}, dof={}, chi2={:.4}",
            summary.samples,
            platter.exponent,
            platter.effective_sig_bits,
            platter.dof,
            platter.chi_square
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{fpf_test, parse_codeword, FpfConfig};
    use crate::rng::{ConstantRng, Rng};

    /// Emits a fixed word forever — enough to test codeword parsing.
    struct FixedWordRng(u32);
    impl Rng for FixedWordRng {
        fn next_u32(&mut self) -> u32 {
            self.0
        }
    }

    #[test]
    fn parse_codeword_reads_exponent_stop_bit_then_significand() {
        // LSB-first stream from 0b101100: bits 0,0,1 (e=2 with stop bit),
        // then 1,0,1 → sig = 0b101 = 5.  Consumed = 3 + 3.
        let mut rng = FixedWordRng(0b101_100);
        let (mut word, mut left) = (0u32, 0usize);
        let (e, sig, used) = parse_codeword(&mut rng, &mut word, &mut left, 3, 7);
        assert_eq!(e, 2);
        assert_eq!(sig, 5);
        assert_eq!(used, 6);
    }

    #[test]
    fn parse_codeword_caps_exponent_without_stop_bit() {
        // All-zero stream: exponent saturates at max_exp (no stop bit read),
        // then sig_bits of zeros.
        let mut rng = ConstantRng::new(0);
        let (mut word, mut left) = (0u32, 0usize);
        let (e, sig, used) = parse_codeword(&mut rng, &mut word, &mut left, 3, 7);
        assert_eq!(e, 7);
        assert_eq!(sig, 0);
        assert_eq!(used, 10);
    }

    #[test]
    fn codewords_are_disjoint_and_consumption_is_tracked() {
        let mut rng = ConstantRng::new(u32::MAX); // stream of ones: e=0 always
        let summary = fpf_test(&mut rng, 1 << 12, &FpfConfig::default());
        // Every codeword is 1 + 14 = 15 bits.
        assert_eq!(summary.consumed_bits, summary.samples * 15);
        assert!(summary.consumed_bits <= summary.total_bits);
    }

    #[test]
    fn constant_stream_has_some_fpf_signal() {
        let mut rng = ConstantRng::new(0);
        let summary = fpf_test(&mut rng, 1 << 18, &FpfConfig::default());
        assert!(
            summary.cross_p_value < 1e-6
                || summary.platter_results.iter().any(|r| r.p_value < 1e-6)
        );
    }
}
