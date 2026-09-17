//! DIEHARD runs up and runs down test.
//!
//! Counts ascending and descending runs in a sequence of 10 000 successive
//! 32-bit words, with run lengths 1 to 5 counted separately and 6 or more
//! pooled.  A rise extends the open up-run and closes the open down-run; a
//! fall or a tie does the reverse.  When the sequence ends both open runs are
//! counted, so every word lies in exactly one up-run and one down-run.
//!
//! With counts c and n words, the quadratic form
//! V = Σᵢⱼ (cᵢ − n·bᵢ)(cⱼ − n·bⱼ)·aᵢⱼ / n, where b holds the expected
//! proportions of runs of each length and A the inverse of their covariance,
//! is asymptotically χ²(6).  Ten sequences give ten p-values in each
//! direction, and a Kolmogorov–Smirnov test of each ten is reported.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).
//! Donald E. Knuth, *The Art of Computer Programming*, Vol. 2, §3.3.2, and
//! R. G. T. Grafton, "Algorithm AS 157: The runs-up and runs-down tests",
//! *Applied Statistics* 30 (1981), for A, b and the χ²(6) law.

use crate::{
    math::{igamc, ks_test},
    result::TestResult,
    rng::Rng,
};

const SEQ_LEN: usize = 10_000;
const REPEATS: usize = 10;
const RUN_MAX: usize = 6;

/// Inverse covariance of the run counts, scaled by n (Knuth, TAOCP Vol. 2,
/// §3.3.2; Grafton 1981).
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
        // igamc(3, v/2) is the upper tail of χ²(6): the run counts do not sum
        // to a fixed total, so no degree of freedom is lost.
        up_pvals.push(igamc(3.0, uv / 2.0));
        dn_pvals.push(igamc(3.0, dv / 2.0));
    }

    let p_up = ks_test(&mut up_pvals);
    let p_dn = ks_test(&mut dn_pvals);
    let (d_up, d_dn) = (
        crate::math::ks_statistic(&mut up_pvals),
        crate::math::ks_statistic(&mut dn_pvals),
    );

    vec![
        TestResult::with_note(
            "diehard::runs_up",
            p_up,
            format!("seq_len={SEQ_LEN}, repeats={REPEATS}, covariance-form"),
        )
        .kolmogorov_smirnov(d_up, REPEATS),
        TestResult::with_note(
            "diehard::runs_down",
            p_dn,
            format!("seq_len={SEQ_LEN}, repeats={REPEATS}, covariance-form"),
        )
        .kolmogorov_smirnov(d_dn, REPEATS),
    ]
}

/// Runs test on a slice, as one result.
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
    let smaller = ks_test(&mut up_pvals).min(ks_test(&mut dn_pvals));
    TestResult::with_note(
        "diehard::runs_up_down",
        (2.0 * smaller).min(1.0),
        format!("seq_len={SEQ_LEN}, repeats={REPEATS}, covariance-form (Bonferroni)"),
    )
    .with_statistic("smaller p-value", smaller, None, "Bonferroni bound")
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
    match run_counts(words) {
        Some((upruns, downruns, n)) => (quadratic_form(&upruns, n), quadratic_form(&downruns, n)),
        None => (f64::NAN, f64::NAN),
    }
}

/// Up-run counts, down-run counts (lengths 1 to 5, then 6 or more) and the
/// sequence length.
type RunCounts = ([usize; RUN_MAX], [usize; RUN_MAX], usize);

/// Count the up-runs and down-runs of one sequence; `None` if it is empty.
///
/// A rise extends the open up-run and closes the open down-run; a fall or a
/// tie does the reverse.  When the sequence ends, both open runs are closed
/// and counted, so every word lies in exactly one up-run and one down-run.
fn run_counts(words: impl IntoIterator<Item = u32>) -> Option<RunCounts> {
    let mut words = words.into_iter();
    let mut last = words.next()?;
    let mut upruns = [0usize; RUN_MAX];
    let mut downruns = [0usize; RUN_MAX];
    let mut ucount = 1usize;
    let mut dcount = 1usize;
    let mut n = 1usize;

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

    upruns[ucount - 1] += 1;
    downruns[dcount - 1] += 1;
    Some((upruns, downruns, n))
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
    use super::{run_counts, runs_float, runs_float_both, REPEATS, SEQ_LEN};
    use crate::rng::{ConstantRng, Mt19937, Rng};

    /// 1 3 2 5 4 4 6: up-runs (1 3) (2 5) (4) (4 6) and down-runs (1) (3 2)
    /// (5 4 4) (6), the tie counting as a fall.  Both runs still open at the
    /// end, (4 6) and (6), are counted.
    #[test]
    fn both_final_runs_are_counted() {
        let (up, down, n) = run_counts([1, 3, 2, 5, 4, 4, 6]).unwrap();
        assert_eq!(n, 7);
        assert_eq!(up, [1, 3, 0, 0, 0, 0]);
        assert_eq!(down, [2, 1, 1, 0, 0, 0]);
        assert!(run_counts([]).is_none());
    }

    /// Without runs of 6 or more, every word lies in exactly one up-run and
    /// one down-run, so the run lengths in each direction sum to n.
    #[test]
    fn run_lengths_cover_the_sequence() {
        let mut rng = Mt19937::new(5489);
        for _ in 0..200 {
            let words: Vec<u32> = (0..40).map(|_| rng.next_u32()).collect();
            let (up, down, n) = run_counts(words).unwrap();
            for counts in [up, down] {
                if counts[5] == 0 {
                    let covered: usize = counts.iter().enumerate().map(|(i, c)| (i + 1) * c).sum();
                    assert_eq!(covered, n);
                }
            }
        }
    }

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
