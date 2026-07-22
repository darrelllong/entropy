//! TestU01 bit-string Hamming tests from `sstring.c` / `sstring.tex`.
//!
//! References:
//! - TestU01 1.2.3, `testu01/sstring.c`
//! - TestU01 user's guide, `testu01/sstring.tex`
//!
//! This module implements the core single-replication statistics for:
//! - `sstring_HammingCorr`
//! - `sstring_HammingIndep`
//!
//! It follows TestU01's bit extraction convention (`unif01_StripB` style:
//! keep the `s` most significant bits after dropping the first `r` bits)
//! and TestU01's `gofs_MinExpected = 10.0` lumping rule for the main
//! Hamming-independence chi-square.

use crate::{
    math::{erfc, igamc},
    result::TestResult,
    rng::Rng,
};
use std::f64::consts::{LN_2, SQRT_2};

const GOFS_MIN_EXPECTED: f64 = 10.0;

fn strip_b(word: u32, r: usize, s: usize) -> u32 {
    if r == 0 {
        word >> (32 - s)
    } else {
        (word << r) >> (32 - s)
    }
}

fn bit_chunks(rng: &mut impl Rng, r: usize, s: usize) -> impl Iterator<Item = (u32, usize)> + '_ {
    std::iter::from_fn(move || Some((strip_b(rng.next_u32(), r, s), s)))
}

fn binomial_probs(l: usize) -> Vec<f64> {
    let mut probs = vec![0.0; l + 1];
    probs[0] = (-(l as f64) * LN_2).exp();
    for k in 1..=l {
        probs[k] = probs[k - 1] * (l + 1 - k) as f64 / k as f64;
    }
    probs
}

fn next_block_weight(blocks: &mut impl Iterator<Item = (u32, usize)>, l: usize) -> Option<usize> {
    let mut remaining = l;
    let mut weight = 0usize;
    while remaining > 0 {
        let (chunk, width) = blocks.next()?;
        let take = remaining.min(width);
        let mask = if take == 32 {
            u32::MAX
        } else {
            (1u32 << take) - 1
        };
        weight += (chunk & mask).count_ones() as usize;
        remaining -= take;
    }
    Some(weight)
}

fn chi_square_pvalue(chi_square: f64, degrees_of_freedom: usize) -> f64 {
    if degrees_of_freedom == 0 {
        return f64::NAN;
    }
    igamc(degrees_of_freedom as f64 / 2.0, chi_square / 2.0)
}

fn chi_square(expected: &[f64], observed: &[u64]) -> f64 {
    expected
        .iter()
        .zip(observed)
        .filter(|(e, _)| **e > 0.0)
        .map(|(e, &o)| {
            let d = o as f64 - *e;
            d * d / *e
        })
        .sum()
}

fn lumped_chi_square(expected: &[f64], observed: &[u64], min_expected: f64) -> (f64, usize, usize) {
    let mut kept_expected = Vec::new();
    let mut kept_observed = Vec::new();
    let mut lumped_expected = 0.0;
    let mut lumped_observed = 0u64;
    let mut lumped_cells = 0usize;

    for (&e, &o) in expected.iter().zip(observed) {
        if e >= min_expected {
            kept_expected.push(e);
            kept_observed.push(o);
        } else {
            lumped_expected += e;
            lumped_observed += o;
            lumped_cells += 1;
        }
    }

    if lumped_expected >= min_expected || kept_expected.is_empty() {
        // Pool the weak cells into their own class.  If NO cell met the
        // threshold this yields a single class → dof 0 → NaN p-value, i.e.
        // an honest insufficient-data signal rather than an arbitrary
        // low-df fallback.
        kept_expected.push(lumped_expected);
        kept_observed.push(lumped_observed);
    } else {
        let last = kept_expected.len() - 1;
        kept_expected[last] += lumped_expected;
        kept_observed[last] += lumped_observed;
    }

    let classes = kept_expected.len();
    let dof = classes.saturating_sub(1);
    (
        chi_square(&kept_expected, &kept_observed),
        dof,
        lumped_cells,
    )
}

/// Outcome of a single `sstring_HammingCorr` replication.
#[derive(Debug, Clone)]
pub struct HammingCorrSummary {
    /// Number of L-bit blocks examined.
    pub n: usize,
    /// Leading bits dropped from each 32-bit word (TestU01 `r`).
    pub r: usize,
    /// Bits kept per word after the drop (TestU01 `s`).
    pub s: usize,
    /// Block length in bits (TestU01 `L`).
    pub l: usize,
    /// Estimated correlation between successive block Hamming weights.
    pub rho_hat: f64,
    /// Normal z-score, `rho_hat · √(n − 1)`.
    pub z_score: f64,
    /// Two-sided normal p-value of the z-score.
    pub p_value: f64,
}

/// TestU01 `sstring_HammingCorr`: serial correlation between the Hamming
/// weights of `n` successive `l`-bit blocks drawn from `rng`.
///
/// # Panics
/// Panics if `n < 2`, `s` is outside `1..=32`, or `r + s > 32`.
pub fn hamming_corr(
    rng: &mut impl Rng,
    n: usize,
    r: usize,
    s: usize,
    l: usize,
) -> HammingCorrSummary {
    assert!(n >= 2, "n must be at least 2");
    assert!(s > 0 && s <= 32, "s must be in 1..=32");
    assert!(r + s <= 32, "r + s must be <= 32");
    let mut chunks = bit_chunks(rng, r, s);
    let mut prev = next_block_weight(&mut chunks, l).expect("insufficient stream");
    let mut sum = 0.0f64;
    let center = l as f64 / 2.0;
    for _ in 1..n {
        let cur = next_block_weight(&mut chunks, l).expect("insufficient stream");
        sum += (prev as f64 - center) * (cur as f64 - center);
        prev = cur;
    }
    let rho_hat = 4.0 * sum / ((n - 1) as f64 * l as f64);
    let z_score = rho_hat * ((n - 1) as f64).sqrt();
    let p_value = erfc(z_score.abs() / SQRT_2);
    HammingCorrSummary {
        n,
        r,
        s,
        l,
        rho_hat,
        z_score,
        p_value,
    }
}

/// Package a [`HammingCorrSummary`] as a [`TestResult`] named
/// `testu01::hamming_corr`.
pub fn hamming_corr_result(summary: &HammingCorrSummary) -> TestResult {
    TestResult::with_note(
        "testu01::hamming_corr",
        summary.p_value,
        format!(
            "n={}, r={}, s={}, L={}, rho_hat={:.6}, z={:.4}",
            summary.n, summary.r, summary.s, summary.l, summary.rho_hat, summary.z_score
        ),
    )
}

/// Outcome of a single `sstring_HammingIndep` replication.
#[derive(Debug, Clone)]
pub struct HammingIndepSummary {
    /// Number of (X, Y) block pairs examined.
    pub n: usize,
    /// Leading bits dropped from each 32-bit word (TestU01 `r`).
    pub r: usize,
    /// Bits kept per word after the drop (TestU01 `s`).
    pub s: usize,
    /// Block length in bits (TestU01 `L`).
    pub l: usize,
    /// Number of corner-block statistics computed (TestU01 `d`).
    pub d: usize,
    /// Chi-square over the (L+1)×(L+1) weight-pair table after lumping.
    pub main_chi_square: f64,
    /// Degrees of freedom of the main chi-square (classes − 1).
    pub main_dof: usize,
    /// Survival p-value of the main chi-square; NaN when `main_dof == 0`.
    pub main_p_value: f64,
    /// Number of low-expectation cells pooled by the
    /// `gofs_MinExpected = 10` lumping rule.
    pub lumped_cells: usize,
    /// Corner-block chi-square statistic for each `k` in `1..=d`.
    pub block_chi_square: Vec<f64>,
    /// Degrees of freedom (1 or 2) for each corner-block statistic.
    pub block_dof: Vec<usize>,
    /// Survival p-value for each corner-block statistic.
    pub block_p_value: Vec<f64>,
}

/// TestU01 `sstring_HammingIndep`: independence of the Hamming weights of
/// successive block pairs — the main lumped chi-square over the weight-pair
/// table plus `d` corner-block statistics.
///
/// # Panics
/// Panics if `n < 20`, `s` is outside `1..=32`, `r + s > 32`, `d` is
/// outside `1..=8`, or `d > (l + 1) / 2`.
pub fn hamming_indep(
    rng: &mut impl Rng,
    n: usize,
    r: usize,
    s: usize,
    l: usize,
    d: usize,
) -> HammingIndepSummary {
    assert!(n as f64 >= 2.0 * GOFS_MIN_EXPECTED, "n must be >= 20");
    assert!(s > 0 && s <= 32, "s must be in 1..=32");
    assert!(r + s <= 32, "r + s must be <= 32");
    assert!((1..=8).contains(&d), "d must be in 1..=8");
    assert!(d <= l.div_ceil(2), "d must be <= (L + 1) / 2");

    let probs = binomial_probs(l);
    let width = l + 1;
    let mut counts = vec![0u64; width * width];
    let mut chunks = bit_chunks(rng, r, s);
    for _ in 0..n {
        let x = next_block_weight(&mut chunks, l).expect("insufficient stream");
        let y = next_block_weight(&mut chunks, l).expect("insufficient stream");
        counts[x * width + y] += 1;
    }

    let mut expected = vec![0.0f64; width * width];
    for i in 0..=l {
        for j in 0..=l {
            expected[i * width + j] = n as f64 * probs[i] * probs[j];
        }
    }
    let (main_chi_square, main_dof, lumped_cells) =
        lumped_chi_square(&expected, &counts, GOFS_MIN_EXPECTED);
    let main_p_value = chi_square_pvalue(main_chi_square, main_dof);

    let l2 = l / 2;
    let mut l1 = l / 2;
    if l % 2 == 1 {
        l1 += 1;
    }
    let mut block_chi_square = Vec::with_capacity(d);
    let mut block_dof = Vec::with_capacity(d);
    let mut block_p_value = Vec::with_capacity(d);
    for k in 1..=d {
        let mut xd0 = 0u64;
        let mut xd1 = 0u64;
        for i in 0..=l1 - k {
            for j in 0..=l1 - k {
                xd0 += counts[i * width + j];
            }
        }
        for i in l2 + k..=l {
            for j in l2 + k..=l {
                xd0 += counts[i * width + j];
            }
        }
        for i in 0..=l1 - k {
            for j in l2 + k..=l {
                xd1 += counts[i * width + j];
            }
        }
        for i in l2 + k..=l {
            for j in 0..=l1 - k {
                xd1 += counts[i * width + j];
            }
        }

        let tail_mass: f64 = probs[..=l1 - k].iter().sum();
        let nb_moyen = tail_mass * tail_mass * n as f64 * 2.0;
        let expected_block = [nb_moyen, nb_moyen, n as f64 - 2.0 * nb_moyen];
        let observed_block = [xd0, xd1, n as u64 - xd0 - xd1];
        let chi = chi_square(&expected_block, &observed_block);
        let dof = if (l % 2 == 1) && k == 1 { 1 } else { 2 };
        block_chi_square.push(chi);
        block_dof.push(dof);
        block_p_value.push(chi_square_pvalue(chi, dof));
    }

    HammingIndepSummary {
        n,
        r,
        s,
        l,
        d,
        main_chi_square,
        main_dof,
        main_p_value,
        lumped_cells,
        block_chi_square,
        block_dof,
        block_p_value,
    }
}

/// Package the main lumped chi-square from `summary` as a [`TestResult`]
/// named `testu01::hamming_indep_main`.
pub fn hamming_indep_main_result(summary: &HammingIndepSummary) -> TestResult {
    TestResult::with_note(
        "testu01::hamming_indep_main",
        summary.main_p_value,
        format!(
            "n={}, r={}, s={}, L={}, dof={}, lumped_cells={}, chi2={:.4}",
            summary.n,
            summary.r,
            summary.s,
            summary.l,
            summary.main_dof,
            summary.lumped_cells,
            summary.main_chi_square
        ),
    )
}

/// Package the `k`-th corner-block statistic (`k` in `1..=d`) as a
/// [`TestResult`] named `testu01::hamming_indep_block`.
///
/// # Panics
/// Panics if `k` is 0 or exceeds `summary.d` (index out of range).
pub fn hamming_indep_block_result(summary: &HammingIndepSummary, k: usize) -> TestResult {
    let idx = k - 1;
    TestResult::with_note(
        "testu01::hamming_indep_block",
        summary.block_p_value[idx],
        format!(
            "n={}, r={}, s={}, L={}, d={}, dof={}, chi2={:.4}",
            summary.n,
            summary.r,
            summary.s,
            summary.l,
            k,
            summary.block_dof[idx],
            summary.block_chi_square[idx]
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{binomial_probs, hamming_corr};
    use crate::rng::ConstantRng;

    #[test]
    fn binomial_probs_sum_to_one() {
        let probs = binomial_probs(12);
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12);
    }

    #[test]
    fn hamming_corr_rejects_constant_stream() {
        let mut rng = ConstantRng::new(0);
        let summary = hamming_corr(&mut rng, 128, 0, 8, 16);
        assert!(summary.p_value < 1e-6);
    }
}
