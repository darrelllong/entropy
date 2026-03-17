//! DIEHARDER test 209 — dab_monobit2.
//!
//! Faithful port of `dab_monobit2.c`. The reference test tries multiple block
//! sizes, computes a binomial chi-square p-value for each, then keeps only the
//! most extreme p-value with the same multiple-test correction used by
//! `evalMostExtreme()` in `dab_dct.c`.
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
        return f64::NAN;
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

fn eval_most_extreme(pvalues: &[f64]) -> f64 {
    let mut extreme = 1.0;
    let mut sign = 1;

    for &raw_p in pvalues {
        let mut p = raw_p;
        let mut cur_sign = -1;
        if p > 0.5 {
            p = 1.0 - p;
            cur_sign = 1;
        }
        if p < extreme {
            extreme = p;
            sign = cur_sign;
        }
    }

    let mut corrected = (1.0 - extreme).powi(pvalues.len() as i32);
    if sign == 1 {
        corrected = 1.0 - corrected;
    }
    corrected
}

#[cfg(test)]
mod tests {
    use super::{auto_ntuple, eval_most_extreme, monobit2};
    use crate::rng::{ConstantRng, Rng};

    #[test]
    fn eval_most_extreme_matches_reference_shape() {
        let p = eval_most_extreme(&[0.2, 0.9, 0.7]);
        assert!((p - 0.271).abs() < 1e-12);
    }

    #[test]
    fn auto_ntuple_is_nonzero_for_dieharder_scale() {
        assert!(auto_ntuple(16_000_000) > 0);
    }

    #[test]
    fn monobit2_returns_finite_pvalue_for_constant_stream() {
        let mut rng = ConstantRng::new(0);
        let words = rng.collect_u32s(1_000_000);
        let result = monobit2(&words);
        assert!(result.p_value.is_finite());
    }
}
