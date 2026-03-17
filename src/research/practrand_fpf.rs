//! PractRand FPF core test from `src/tests.cpp`.
//!
//! Reference:
//! - PractRand pre-0.95, `include/PractRand/Tests/FPF.h`
//! - PractRand pre-0.95, `src/tests.cpp` (`PractRand::Tests::FPF`)
//!
//! This ports the core counting/test logic:
//! - sample a stride-spaced LSB-first bitstream window
//! - interpret it as an FPF exponent/significand bucket
//! - apply PractRand's intra-platter truncation rule and G-test
//! - apply a grouped exponent-distribution G-test (`:cross`)
//!
//! It intentionally does not claim to reproduce PractRand's empirical
//! calibration tables or suspicion scores; p-values are from the asymptotic
//! chi-square law for the same core statistics.

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

#[derive(Debug, Clone)]
pub struct FpfConfig {
    pub stride_bits_l2: usize,
    pub sig_bits: usize,
    pub exp_bits: usize,
}

impl Default for FpfConfig {
    fn default() -> Self {
        Self {
            stride_bits_l2: 4,
            sig_bits: 14,
            exp_bits: 6,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FpfPlatterSummary {
    pub exponent: usize,
    pub effective_sig_bits: usize,
    pub chi_square: f64,
    pub dof: usize,
    pub p_value: f64,
    pub samples: usize,
}

#[derive(Debug, Clone)]
pub struct FpfSummary {
    pub total_bits: usize,
    pub stride_bits: usize,
    pub sig_bits: usize,
    pub max_exp: usize,
    pub samples: usize,
    pub platter_results: Vec<FpfPlatterSummary>,
    pub cross_chi_square: f64,
    pub cross_dof: usize,
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

fn bitstream_window_sample(window: u128, sig_bits: usize, max_exp: usize) -> (usize, usize) {
    let mut e = 0usize;
    while e < max_exp && ((window >> e) & 1) == 0 {
        e += 1;
    }
    if e < max_exp {
        let sig = ((window >> (e + 1)) & ((1u128 << sig_bits) - 1)) as usize;
        (e, sig)
    } else {
        let sig = ((window >> max_exp) & ((1u128 << sig_bits) - 1)) as usize;
        (max_exp, sig)
    }
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

pub fn fpf_test(rng: &mut impl Rng, total_bits: usize, config: &FpfConfig) -> FpfSummary {
    let stride_bits = 1usize << config.stride_bits_l2;
    let max_exp = (1usize << config.exp_bits) - 1;
    let footprint = config.sig_bits + max_exp;
    assert!(
        config.sig_bits > 0 && config.sig_bits <= 20,
        "sig_bits out of range"
    );
    assert!(
        footprint <= 128,
        "current FPF port supports footprints up to 128 bits"
    );
    assert!(
        total_bits >= footprint,
        "total_bits must cover at least one FPF footprint"
    );

    let mut plateau_counts = vec![vec![0u64; 1usize << config.sig_bits]; max_exp + 1];
    let mut exp_counts = vec![0u64; max_exp + 1];

    let mut current_word = 0u32;
    let mut bits_left = 0usize;

    let mut window = 0u128;
    for i in 0..footprint {
        window |= (next_lsb_bit(rng, &mut current_word, &mut bits_left) as u128) << i;
    }

    let mut consumed = footprint;
    let mut samples = 0usize;
    loop {
        let (e, sig) = bitstream_window_sample(window, config.sig_bits, max_exp);
        plateau_counts[e][sig] += 1;
        exp_counts[e] += 1;
        samples += 1;

        if consumed + stride_bits > total_bits {
            break;
        }
        window >>= stride_bits;
        for j in 0..stride_bits {
            let bit = next_lsb_bit(rng, &mut current_word, &mut bits_left) as u128;
            window |= bit << (footprint - stride_bits + j);
        }
        consumed += stride_bits;
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
        stride_bits,
        sig_bits: config.sig_bits,
        max_exp,
        samples,
        platter_results,
        cross_chi_square,
        cross_dof,
        cross_p_value,
    }
}

pub fn fpf_cross_result(summary: &FpfSummary) -> TestResult {
    TestResult::with_note(
        "practrand::fpf_cross",
        summary.cross_p_value,
        format!(
            "samples={}, stride_bits={}, sig_bits={}, max_exp={}, dof={}, chi2={:.4}",
            summary.samples,
            summary.stride_bits,
            summary.sig_bits,
            summary.max_exp,
            summary.cross_dof,
            summary.cross_chi_square
        ),
    )
}

pub fn fpf_platter_result(platter: &FpfPlatterSummary, summary: &FpfSummary) -> TestResult {
    TestResult::with_note(
        "practrand::fpf_platter",
        platter.p_value,
        format!(
            "samples={}, stride_bits={}, e={}, sig_bins=2^{}, dof={}, chi2={:.4}",
            summary.samples,
            summary.stride_bits,
            platter.exponent,
            platter.effective_sig_bits,
            platter.dof,
            platter.chi_square
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{bitstream_window_sample, fpf_test, FpfConfig};
    use crate::rng::ConstantRng;

    #[test]
    fn window_sample_uses_trailing_zero_exponent() {
        let window = 0b0110_1000u128;
        let (e, sig) = bitstream_window_sample(window, 3, 7);
        assert_eq!(e, 3);
        assert_eq!(sig, 0b110);
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
