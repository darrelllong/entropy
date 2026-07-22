//! DIEHARDER test 209 — dab_monobit2.
//!
//! Port of `dab_monobit2.c`. The reference test tries multiple block sizes,
//! computes a binomial chi-square p-value for each, then keeps only the most
//! extreme p-value with a Šidák multiple-test correction (cf. `evalMostExtreme()`
//! in `dab_dct.c`).
//!
//! Deliberate deviation from the C: `evalMostExtreme` maps low-side extremes to
//! p ≈ 1, which dieharder's harness flags as failure but this crate's one-sided
//! `p ≥ α` pass rule would report as PASS. Here each per-block p-value is folded
//! two-sided (`2·min(p, 1−p)`) *before* the Šidák correction, so both failure
//! directions map to small p while H₀ uniformity is preserved.
//!
//! # Author
//! David Bauer, *Dieharder* (2006), test `dab_monobit2`.

use crate::{
    math::{igamc, lgamma},
    result::TestResult,
};

const BLOCK_MAX: usize = 16;
const RMAX_BITS: usize = 32;
const GOFS_MIN_OBSERVED: f64 = 10.0;
const LN_HALF: f64 = -std::f64::consts::LN_2;

/// Run the enhanced monobit test.
///
/// # Author
/// David Bauer, Dieharder (2006), `dab_monobit2`.
pub fn monobit2(words: &[u32]) -> TestResult {
    if words.len() < 2 {
        return TestResult::insufficient("dieharder::monobit2", "not enough words");
    }

    let ntup = auto_ntuple(words.len());
    if ntup == 0 {
        return TestResult::insufficient(
            "dieharder::monobit2",
            "not enough samples for any block size",
        );
    }

    // This layout intentionally matches the C code's single flat buffer:
    // segment j starts at blens * ((2 << j) - 1).
    let mut counts = vec![0.0f64; RMAX_BITS * (2 << ntup)];
    let mut temp_count = vec![0u32; ntup];

    for (i, &word) in words.iter().enumerate() {
        let ones = word.count_ones();
        let mut t = 1usize;
        for j in 0..ntup {
            temp_count[j] += ones;
            if (t & i) != 0 && (t & (i.saturating_sub(1))) == 0 {
                let offset = RMAX_BITS * ((2 << j) - 1);
                counts[offset + temp_count[j] as usize] += 1.0;
                temp_count[j] = 0;
            }
            t <<= 1;
        }
    }

    let mut pvalues = Vec::with_capacity(ntup);
    for j in 0..ntup {
        let block_words = 2 << j;
        let kmax = RMAX_BITS * block_words;
        let nsamp = words.len() / block_words;
        let offset = RMAX_BITS * (block_words - 1);
        let p = chisq_binomial(&counts[offset..=offset + kmax], 0.5, kmax, nsamp);
        pvalues.push(p);
    }

    let p_value = eval_most_extreme(&pvalues);
    TestResult::with_note(
        "dieharder::monobit2",
        p_value,
        format!(
            "tsamples={}, ntuple={}, block_sizes=2..{}",
            words.len(),
            ntup,
            2usize << (ntup - 1)
        ),
    )
}

fn auto_ntuple(tsamples: usize) -> usize {
    let mut ntup = BLOCK_MAX;
    for j in 0..BLOCK_MAX {
        let block_words = 2usize << j;
        let nmax = RMAX_BITS * block_words;
        let nsamp = tsamples / block_words;
        if nsamp == 0 {
            ntup = j;
            break;
        }
        let mid = nmax / 2;
        let log_pdf =
            lgamma((nmax + 1) as f64) - lgamma((mid + 1) as f64) - lgamma((nmax - mid + 1) as f64)
                + (nmax as f64) * LN_HALF;
        let center_mass = log_pdf.exp();
        if (nsamp as f64) * center_mass < 20.0 {
            ntup = j;
            break;
        }
    }
    ntup
}

fn chisq_binomial(observed: &[f64], prob: f64, kmax: usize, nsamp: usize) -> f64 {
    let mut chi_sq = 0.0;
    let mut ndof = 0usize;

    for (n, &obs) in observed.iter().take(kmax + 1).enumerate() {
        if obs > GOFS_MIN_OBSERVED {
            let expected = (nsamp as f64) * binomial_pdf(n, kmax, prob);
            let delta = obs - expected;
            chi_sq += delta * delta / expected;
            ndof += 1;
        }
    }

    let df = ndof.saturating_sub(1);
    if df == 0 {
        // Fewer than two populated cells.  With a nonzero chi-square this is a
        // wildly concentrated distribution — catastrophic evidence, not missing
        // data (the C reaches GSL's Q(0, x > 0) = 0 here).  A zero chi-square
        // means nothing was measurable at all.
        return if chi_sq > 0.0 { 0.0 } else { f64::NAN };
    }
    igamc(df as f64 / 2.0, chi_sq / 2.0)
}

fn binomial_pdf(k: usize, n: usize, prob: f64) -> f64 {
    let q = 1.0 - prob;
    let log_p = lgamma((n + 1) as f64) - lgamma((k + 1) as f64) - lgamma((n - k + 1) as f64)
        + (k as f64) * prob.ln()
        + ((n - k) as f64) * q.ln();
    log_p.exp()
}

/// Most-extreme p-value across blocks, two-sided.
///
/// Each per-block p is folded two-sided (`2·min(p, 1−p)`, uniform under H₀),
/// then the minimum fold is Šidák-corrected for the number of usable blocks.
/// NaN blocks (no measurable statistic) are excluded; all-NaN returns NaN so
/// the caller reports an insufficient-data result rather than a verdict.
fn eval_most_extreme(pvalues: &[f64]) -> f64 {
    let mut n = 0u32;
    let mut min_fold = f64::INFINITY;
    for &p in pvalues.iter().filter(|p| !p.is_nan()) {
        n += 1;
        min_fold = min_fold.min(2.0 * p.min(1.0 - p));
    }
    if n == 0 {
        return f64::NAN;
    }
    1.0 - (1.0 - min_fold).powi(n as i32)
}

#[cfg(test)]
mod tests {
    use super::{auto_ntuple, eval_most_extreme, monobit2};
    use crate::rng::{ConstantRng, Rng};

    #[test]
    fn eval_most_extreme_two_sided_sidak() {
        // Folds: 0.4, 0.2, 0.6 → min 0.2 → 1 − 0.8³ = 0.488.
        let p = eval_most_extreme(&[0.2, 0.9, 0.7]);
        assert!((p - 0.488).abs() < 1e-12);
        // BOTH extremes must map to small p — a p ≈ 1 block is a failure too.
        assert!(eval_most_extreme(&[1.0 - 1e-9, 0.5, 0.5]) < 1e-6);
        assert!(eval_most_extreme(&[1e-9, 0.5, 0.5]) < 1e-6);
        // NaN blocks are excluded; all-NaN yields NaN (→ SKIP), not a verdict.
        assert!((eval_most_extreme(&[f64::NAN, 0.5]) - 1.0).abs() < 1e-12);
        assert!(eval_most_extreme(&[f64::NAN, f64::NAN]).is_nan());
    }

    #[test]
    fn auto_ntuple_is_nonzero_for_dieharder_scale() {
        assert!(auto_ntuple(16_000_000) > 0);
    }

    /// A constant stream concentrates every block's bit-count in one bin;
    /// that must FAIL (p ≈ 0), not skip and not pass.
    #[test]
    fn monobit2_fails_constant_stream() {
        let mut rng = ConstantRng::new(0);
        let words = rng.collect_u32s(1_000_000);
        let result = monobit2(&words);
        assert!(!result.skipped(), "{result}");
        assert!(result.p_value < 0.01, "{result}");
    }
}
