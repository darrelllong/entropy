//! DIEHARD Test 15 — Runs Test (floating-point / integer comparison).
//!
//! Counts ascending and descending monotone runs in successive 32-bit
//! integers from the generator.  Run lengths are binned into 6 categories
//! (≥6 pooled).  A quadratic form in the weak inverse of the known
//! covariance matrix gives a chi-square-like statistic with df = 6.
//!
//! This is repeated 10 times, yielding 10 p-values each for up-runs and
//! down-runs.  A final Kolmogorov-Smirnov test on each set of 10 p-values
//! produces the reported results.
//!
//! Covariance matrix and expected proportions from:
//! R.G.T. Grafton, "The Runs-Up and Runs-Down Tests", *Applied Statistics*
//! 30, Algorithm AS 157, 1981.  See also Knuth TAOCP Vol 2 §3.3.2.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{
    math::{igamc, ks_test},
    result::TestResult,
    rng::Rng,
};

const SEQ_LEN: usize = 10_000;
const REPEATS: usize = 10;
const RUN_MAX: usize = 6;

/// Pseudoinverse of the covariance matrix for runs-up (= runs-down), scaled
/// by n.  Source: Grafton 1981 (AS 157), Knuth TAOCP Vol 2, as reproduced in
/// Dieharder 3.31.1 diehard_runs.c.
const A: [[f64; RUN_MAX]; RUN_MAX] = [
    [4529.4, 9044.9, 13568.0, 18091.0, 22615.0, 27892.0],
    [9044.9, 18097.0, 27139.0, 36187.0, 45234.0, 55789.0],
    [13568.0, 27139.0, 40721.0, 54281.0, 67852.0, 83685.0],
    [18091.0, 36187.0, 54281.0, 72414.0, 90470.0, 111580.0],
    [22615.0, 45234.0, 67852.0, 90470.0, 113262.0, 139476.0],
    [27892.0, 55789.0, 83685.0, 111580.0, 139476.0, 172860.0],
];

/// Expected proportion of runs of length i+1 (i=0..5, where i=5 means ≥6).
const B: [f64; RUN_MAX] = [
    1.0 / 6.0,
    5.0 / 24.0,
    11.0 / 120.0,
    19.0 / 720.0,
    29.0 / 5040.0,
    1.0 / 840.0,
];

/// Run the runs test using the covariance-matrix quadratic form from DIEHARD.
///
/// Returns two `TestResult`s: one for up-runs, one for down-runs.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn runs_float_both(rng: &mut impl Rng) -> Vec<TestResult> {
    let mut up_pvals: Vec<f64> = Vec::with_capacity(REPEATS);
    let mut dn_pvals: Vec<f64> = Vec::with_capacity(REPEATS);

    for _ in 0..REPEATS {
        let (uv, dv) = runs_quad_form((0..SEQ_LEN).map(|_| rng.next_u32()));
        // igamc(3, v/2) = p-value for χ²(6).  df=6 for 6 bins is correct per
        // Grafton (1981) AS 157 §3: the covariance matrix has rank 6 because
        // the constraint Σcounts=n is absorbed into the pseudoinverse, not
        // subtracted as the usual -1 degree of freedom.
        up_pvals.push(igamc(3.0, uv / 2.0));
        dn_pvals.push(igamc(3.0, dv / 2.0));
    }

    let p_up = ks_test(&mut up_pvals);
    let p_dn = ks_test(&mut dn_pvals);

    vec![
        TestResult::with_note(
            "diehard::runs_up",
            p_up,
            format!("seq_len={SEQ_LEN}, repeats={REPEATS}, covariance-form"),
        ),
        TestResult::with_note(
            "diehard::runs_down",
            p_dn,
            format!("seq_len={SEQ_LEN}, repeats={REPEATS}, covariance-form"),
        ),
    ]
}

/// Backward-compatible single-result wrapper.
///
/// Combines the up-runs and down-runs KS p-values with a Bonferroni bound
/// (valid under their dependence — both directions come from the same
/// sequences); a bare min doubled the false-failure rate at small α.
/// Prefer [`runs_float_both`] (used by `run_all`) for the uncombined report.
pub fn runs_float(words: &[u32]) -> TestResult {
    // This wrapper uses the first SEQ_LEN*REPEATS words from the slice.
    let needed = SEQ_LEN * REPEATS;
    if words.len() < needed {
        return TestResult::insufficient("diehard::runs_up_down", "not enough words");
    }
    let (mut up_pvals, mut dn_pvals): (Vec<f64>, Vec<f64>) = (0..REPEATS)
        .map(|rep| {
            let slice = &words[rep * SEQ_LEN..(rep + 1) * SEQ_LEN];
            runs_quad_form(slice.iter().copied())
        })
        .map(|(uv, dv)| (igamc(3.0, uv / 2.0), igamc(3.0, dv / 2.0)))
        .unzip();
    let p = (2.0 * ks_test(&mut up_pvals).min(ks_test(&mut dn_pvals))).min(1.0);
    TestResult::with_note(
        "diehard::runs_up_down",
        p,
        format!("seq_len={SEQ_LEN}, repeats={REPEATS}, covariance-form (Bonferroni)"),
    )
}

/// Compute the quadratic form statistic for up-runs and down-runs in one
/// sequence of 32-bit words.
///
/// Both entry points count through this function: [`runs_float_both`] passes
/// words drawn from the generator as the loop asks for them, and
/// [`runs_float`] passes a slice.  Returns (uv, dv) where
/// p = igamc(3.0, v/2.0) for each direction; an empty sequence has no
/// statistic and gives NaN for both.
fn runs_quad_form(words: impl IntoIterator<Item = u32>) -> (f64, f64) {
    let mut words = words.into_iter();
    let Some(first) = words.next() else {
        return (f64::NAN, f64::NAN);
    };
    let mut upruns = [0usize; RUN_MAX];
    let mut downruns = [0usize; RUN_MAX];
    let mut ucount = 1usize;
    let mut dcount = 1usize;
    let mut n = 1usize;
    let mut last = first;

    for next in words {
        n += 1;
        if next > last {
            ucount += 1;
            if ucount > RUN_MAX {
                ucount = RUN_MAX;
            }
            downruns[dcount - 1] += 1;
            dcount = 1;
        } else {
            dcount += 1;
            if dcount > RUN_MAX {
                dcount = RUN_MAX;
            }
            upruns[ucount - 1] += 1;
            ucount = 1;
        }
        last = next;
    }

    // Closing convention from diehard_runs.c: the final partial run direction
    // is determined by comparing the last output with the first.
    if last > first {
        downruns[dcount - 1] += 1;
    } else {
        upruns[ucount - 1] += 1;
    }

    (quadratic_form(&upruns, n), quadratic_form(&downruns, n))
}

/// v = Σᵢⱼ (counts[i] − n·b[i]) · (counts[j] − n·b[j]) · A[i][j] / n
fn quadratic_form(counts: &[usize; RUN_MAX], n: usize) -> f64 {
    let nf = n as f64;
    let mut v = 0.0f64;
    for i in 0..RUN_MAX {
        for j in 0..RUN_MAX {
            v += (counts[i] as f64 - nf * B[i]) * (counts[j] as f64 - nf * B[j]) * A[i][j];
        }
    }
    v / nf
}

#[cfg(test)]
mod tests {
    use super::{runs_float, runs_float_both, REPEATS, SEQ_LEN};
    use crate::rng::{ConstantRng, Mt19937, Rng};

    /// Both entry points count the same words the same way: the slice
    /// wrapper's Bonferroni p is twice the smaller of the two KS p-values the
    /// generator path reports for the same stream.
    #[test]
    fn slice_and_generator_paths_agree() {
        let words = Mt19937::new(5489).collect_u32s(SEQ_LEN * REPEATS);
        let both = runs_float_both(&mut Mt19937::new(5489));
        let want = (2.0 * both[0].p_value.min(both[1].p_value)).min(1.0);
        assert_eq!(runs_float(&words).p_value.to_bits(), want.to_bits());
    }

    #[test]
    fn short_inputs_skip() {
        assert!(runs_float(&[]).skipped());
        assert!(runs_float(&vec![0; SEQ_LEN * REPEATS - 1]).skipped());
    }

    /// A constant sequence never rises: every step closes a length-1 up-run
    /// and the single down-run never closes.
    #[test]
    fn constant_input_fails() {
        let r = runs_float(&vec![0; SEQ_LEN * REPEATS]);
        assert!(!r.skipped() && !r.passed(), "{r}");
        for r in runs_float_both(&mut ConstantRng::new(0)) {
            assert!(!r.skipped() && !r.passed(), "{r}");
        }
    }
}
