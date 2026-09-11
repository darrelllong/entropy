//! OPERM5, the overlapping 5-permutation test, as corrected in Dieharder
//! 3.31.1.  Result name: `diehard_historical::operm5_dieharder`.
//!
//! # What it is
//!
//! Marsaglia's OPERM5 (`tests.txt`) reads 32-bit words five at a time,
//! overlapping: each of n = 1 000 000 windows w₀…w₄, w₁…w₅, … is in one of
//! the 5! = 120 orderings of five values.  The 120 counts c are
//! asymptotically normal with mean n/120 and covariance n·C, where C sums the
//! covariances of one window's pattern indicators with those of the windows
//! up to four places either side.  C has rank 96 = 5! − 4!
//! (`diehard_operm5.c` lines 27–28, and the enumeration test below), so with
//! P its Moore–Penrose pseudoinverse the quadratic form
//! χ² = (c − n/120)ᵀ P (c − n/120) / n is χ²(96) under the null.  The p-value
//! is the upper tail Q(48, χ²/2): a small p is a bad fit.
//!
//! # Reference followed
//!
//! Robert G. Brown's Dieharder 3.31.1, `libdieharder/diehard_operm5.c`, in
//! the overlapping mode it runs by default, with the pseudoinverse Stephen
//! Moenkehues computed for it (`include/dieharder/diehard_operm5.h`, kept as
//! data in `operm5_table.rs`).  [pubs/dieharder-3.31.1.tgz]  The code is this
//! crate's `src/diehard/operm5.rs` as commit 3b41af8 removed it, with its
//! computation unchanged:
//!
//! - The permutation index is Dieharder's `kperm` (lines 73–122), not
//!   Marsaglia's `kp`: for i = 4 down to 1 it finds the largest of the first
//!   i + 1 values, ties going to the later one, at position k, sets
//!   index = (i + 1)·index + k and swaps that value into place i.
//! - One pass: five words fill the first window, then each of the 1 000 000
//!   windows is counted and the next word replaces the oldest (lines 143–175),
//!   1 000 005 words in all.
//! - χ² = Σᵢ Σⱼ xᵢ Pᵢⱼ xⱼ / n with x = c − n/120, summed in the order
//!   `diehard_operm5.c` sums it (lines 182–228); df 96 (line 230) and the
//!   upper tail `gsl_sf_gamma_inc_Q` (line 239).
//!
//! Only the names, this documentation, `static` storage for the table and the
//! input gate changed.  The removed module scored any stream of at least
//! 10 005 words with df 96; this one requires the full 1 000 005, the only
//! length its calibration covers.
//!
//! # Departures from Dieharder
//!
//! - Words are compared as `u32`.  Dieharder copies them into `int w[5]`
//!   (line 77) and so orders them as signed integers, as `diehard.f` also
//!   does.  Either is a total order on words, so the 120 orderings are
//!   equally likely under either and χ² has the same null law; for a given
//!   stream the counts differ.  Flipping bit 31 of every word turns one order
//!   into the other, and the tests pin both against Dieharder's own C.
//! - Dieharder reports |χ²| and warns when the form is negative
//!   (lines 224–228).  P is positive semidefinite, so the form is not
//!   negative beyond rounding, and the sign is kept.
//! - Dieharder repeats the test for 100 p-samples by default and combines them
//!   with a Kolmogorov–Smirnov test; this is one sample.
//!
//! # Departures from `diehard.f`
//!
//! Marsaglia's `cdoperm5` (`fortran/diehard.f` lines 1143–1209) differs from
//! this test in four ways.
//!
//! - Index and matrix: `kp` (lines 1210–1234) ranks the five words by its own
//!   rule, compares them as signed integers and relabels indices below 60
//!   through a 60-entry `map`.  The weak inverse is read from `operm5d.ata` as
//!   two 60 × 60 upper triangles R and S (lines 1154–1157) and applied as
//!   Σᵢⱼ [xᵢ Rᵢⱼ xⱼ + yᵢ Sᵢⱼ yⱼ] / (2·10⁸ n) with xᵢ = tᵢ + tᵢ₊₆₀ − av and
//!   yᵢ = tᵢ − tᵢ₊₆₀ (lines 1187–1193).
//! - Degrees of freedom: 99, "the rank is 99=50+49" (lines 1153 and 1198).
//! - Passes: two, over the same 1 000 005 words (`DO 8888`, lines 1165–1202).
//! - p-value: `chisq(chsq,99)` (line 1198), a Wilson–Hilferty approximation
//!   to the χ²(99) CDF, so a p near 1 is the bad fit.
//!
//! Marsaglia's version is miscalibrated, and the fault is in its data, not
//! in a transcription.  Parsing `operm5d.ata` with Fortran record semantics
//! and enumerating C in `kp`+`map` order (a NumPy script from the fidelity
//! review, rerun for this module): R has full rank 60, not 50, and 9 negative
//! eigenvalues (the smallest −1.453·10⁸ against a largest of 2.135·10⁸); S is
//! positive definite, with 49 eigenvalues above 7; and the combined matrix W
//! is not a generalised inverse of C (‖CWC − C‖/‖C‖ = 1.52).  Under the null
//! Marsaglia's χ² is Σ λᵢ χ²(1) over the eigenvalues λ of C^½ W C^½, with
//! mean 97.50 and variance 448 where χ²(99) has 99 and 198.  In 400 000
//! draws of that sum, `chisq(χ², 99)` exceeded 0.99 in 4.89% and 0.999 in
//! 1.94% of draws, and fell below 0.01 in 6.83%.  Dieharder's rank, 96, is
//! right; Marsaglia's 99 is not.
//!
//! # Calibration evidence
//!
//! - **The matrix.**  A test below rebuilds C by enumerating all (5 + d)!
//!   orderings for d = 1 to 4 and checks that Dieharder's table P satisfies
//!   C·P·C = C (largest error under 10⁻¹²), P·C·P = P (relative error under
//!   10⁻⁹) and trace(C·P) = 96 (within 10⁻⁶): P is a generalised inverse of C
//!   and the form has 96 degrees of freedom.  In NumPy the table equals
//!   pinv(C) to 4.5·10⁻¹⁰ (relative 3.7·10⁻¹²), and C^½ P C^½ has exactly 96
//!   unit eigenvalues.
//! - **Goldens.**  On words 1 to 1 000 005 of the fidelity review's input,
//!   χ² agrees to 10⁻⁹ with a C harness compiled from Dieharder's own `kperm`
//!   and quadratic form: 97.432 823 106 7 with this module's unsigned order,
//!   and 80.350 208 578 1 with Dieharder's signed order after bit 31 of every
//!   word is flipped.
//! - **Null simulation.**  40 000 streams of 1 000 005 words, each from a
//!   separately seeded PCG64 generator: p < 0.01 in 396 (0.99%; binomial
//!   standard deviation 0.05%) and p < 0.001 in 37 (0.093%; 0.016%), with
//!   p > 0.99 in 1.02%; a Kolmogorov–Smirnov test of the 40 000 p-values gives
//!   p = 0.067.  χ² had mean 95.87 and variance 192.7, against 96 and 192.  A
//!   test below runs a fixed 100-stream version under `cargo test --release`.
//!
//! # Why it is outside the default battery
//!
//! Commit 3b41af8 removed it, citing Dieharder's description of the original
//! OPERM5 as broken.  The version it removed was Dieharder's corrected one,
//! which `dieharder -l` rates "Good" (`dieharder/list_tests.c` lines 26–37;
//! OPERM5 is test 1 in `include/dieharder/tests.h`), and the evaluation above
//! finds it calibrated.  It stays out of the default battery because adding
//! slots to that battery is a decision of its own, not a side effect of
//! restoring a source.  The default battery already scores 5-permutations
//! with [`crate::dieharder::permutations`], Brown's `rgb_permutations`, which
//! avoids the covariance by using non-overlapping windows.
//!
//! # Author
//! George Marsaglia, DIEHARD (1995), for the test; Robert G. Brown and
//! Stephen Moenkehues, Dieharder 3.31.1, for this form of it.

use super::operm5_table::PSEUDO_INVERSE;
use crate::{math::igamc, result::TestResult};

/// Result name.
const NAME: &str = "diehard_historical::operm5_dieharder";
/// Overlapping windows counted: Dieharder's default `tsamples`.
const N_WINDOWS: usize = 1_000_000;
/// Words the test reads: five for the first window, then one per window.
pub const WORDS: usize = N_WINDOWS + 5;
/// Orderings of five values.
const N_PERMS: usize = 120;
/// Rank of the covariance matrix, 5! − 4!.
const DF: f64 = 96.0;

/// OPERM5 as Dieharder 3.31.1 computes it, on the first [`WORDS`] words.
///
/// Reports SKIP for fewer than [`WORDS`] words.  See the module
/// documentation for the statistic and its departures from `diehard.f`.
///
/// # Author
/// George Marsaglia, DIEHARD (1995); Robert G. Brown and Stephen Moenkehues,
/// Dieharder 3.31.1.
pub fn operm5_dieharder(words: &[u32]) -> TestResult {
    if words.len() < WORDS {
        return TestResult::insufficient(NAME, "need 1 000 005 words");
    }
    let chisq = operm5_statistic(&words[..WORDS]);
    let p_value = igamc(DF / 2.0, chisq / 2.0);
    TestResult::with_note(
        NAME,
        p_value,
        format!("n={N_WINDOWS}, df={DF}, χ²={chisq:.4}"),
    )
}

/// χ² = xᵀ P x / n over the overlapping windows of `words`, which must hold
/// at least [`WORDS`] words.
fn operm5_statistic(words: &[u32]) -> f64 {
    let mut count = [0u32; N_PERMS];

    // Circular buffer of 5 words plus a rolling offset, mirroring the C code.
    let mut v = [0u32; 5];
    v.copy_from_slice(&words[..5]);
    let mut vind = 0usize;

    for t in 0..N_WINDOWS {
        count[kperm(&v, vind)] += 1;
        // Each counted window consumes one fresh replacement word.
        v[vind] = words[t + 5];
        vind = (vind + 1) % 5;
    }

    let nf = N_WINDOWS as f64;
    let expected = nf / N_PERMS as f64; // n / 120
    let x: Vec<f64> = count.iter().map(|&c| c as f64 - expected).collect();

    // One running sum over i, then j, as diehard_operm5.c accumulates it.
    let mut chisq = 0.0f64;
    for (&xi, row) in x.iter().zip(PSEUDO_INVERSE.iter()) {
        for (&pij, &xj) in row.iter().zip(&x) {
            chisq += xi * pij * xj;
        }
    }
    chisq / nf
}

/// Dieharder's `kperm`: the permutation index, 0..120, of the five words
/// `v[voffset]`, `v[voffset + 1]`, … read circularly, compared as `u32`.
///
/// Translated from Dieharder 3.31.1 `diehard_operm5.c` lines 71–110.
fn kperm(v: &[u32; 5], voffset: usize) -> usize {
    let mut w: [u32; 5] = std::array::from_fn(|i| v[(i + voffset) % 5]);
    let mut pindex = 0usize;
    for i in (1..=4).rev() {
        let mut max = w[0];
        let mut k = 0usize;
        for (j, &wj) in w.iter().enumerate().take(i + 1).skip(1) {
            if max <= wj {
                max = wj;
                k = j;
            }
        }
        pindex = (i + 1) * pindex + k;
        w.swap(i, k);
    }
    pindex
}

#[cfg(test)]
mod tests {
    use super::super::{operm5_table::PSEUDO_INVERSE, oracle};
    use super::{kperm, operm5_dieharder, operm5_statistic, N_PERMS, WORDS};
    use crate::{
        math::ks_test,
        rng::{Pcg64, Rng},
    };

    /// Bit 31, whose flip turns unsigned order into signed order.
    const SIGN_BIT: u32 = 1 << 31;

    /// χ² on `in.bin` words 1 to 1 000 005 from a C harness compiled from
    /// Dieharder 3.31.1's own code: `kperm` (`diehard_operm5.c` lines 73–122)
    /// and the overlapping count and quadratic form (lines 143–228), with
    /// `pseudoInv` included from the header.  With `int w[5]`, as Dieharder
    /// declares it, the harness printed this value.
    const DIEHARDER_C_SIGNED: f64 = 80.350_208_578_1;
    /// The same harness with `w[5]` declared `unsigned int`, the order this
    /// module uses.  The fidelity review reported 97.4328 for the removed
    /// module on the same words.
    const DIEHARDER_C_UNSIGNED: f64 = 97.432_823_106_7;
    /// The harness printed ten decimals.
    const PRINTED_TOL: f64 = 1e-9;

    /// Fidelity: χ² matches Dieharder's own C on the review input in both
    /// orders, to the ten decimals the harness printed.
    #[test]
    fn statistic_matches_dieharder_c_on_the_review_input() {
        let words = oracle::words(WORDS);
        let unsigned = operm5_statistic(&words);
        assert!(
            (unsigned - DIEHARDER_C_UNSIGNED).abs() < PRINTED_TOL,
            "{unsigned}"
        );
        let flipped: Vec<u32> = words.iter().map(|&w| w ^ SIGN_BIT).collect();
        let signed = operm5_statistic(&flipped);
        assert!(
            (signed - DIEHARDER_C_SIGNED).abs() < PRINTED_TOL,
            "{signed}"
        );
    }

    /// Q(48, χ²/2) on the review input from this crate's `igamc`, pinned on
    /// the landing tree.
    const GOLDEN_P: f64 = 0.439_998_873_488_646_64;

    /// Regression: the result on the review input, note and p-value.
    #[test]
    fn result_on_the_review_input_is_pinned() {
        let result = operm5_dieharder(&oracle::words(WORDS));
        assert_eq!(result.note.as_deref(), Some("n=1000000, df=96, χ²=97.4328"));
        assert!((result.p_value - GOLDEN_P).abs() < 1e-12, "{result}");
    }

    #[test]
    fn kperm_numbers_the_120_orderings_once_each() {
        let mut seen = [false; N_PERMS];
        for_each_permutation(&mut [0, 1, 2, 3, 4], |p| {
            let v: [u32; 5] = p.try_into().expect("five values");
            let k = kperm(&v, 0);
            assert!(!seen[k], "index {k} repeated");
            seen[k] = true;
        });
        assert!(seen.iter().all(|&s| s));
    }

    /// Calls `visit` on each of the n! orderings of `items` (Heap's
    /// algorithm).
    fn for_each_permutation(items: &mut [u32], mut visit: impl FnMut(&[u32])) {
        let n = items.len();
        let mut c = vec![0usize; n];
        visit(items);
        let mut i = 1;
        while i < n {
            if c[i] < i {
                if i % 2 == 0 {
                    items.swap(0, i);
                } else {
                    items.swap(c[i], i);
                }
                visit(items);
                c[i] += 1;
                i = 1;
            } else {
                c[i] = 0;
                i += 1;
            }
        }
    }

    type Matrix = Vec<[f64; N_PERMS]>;

    /// The per-window covariance C of the 120 pattern indicators, in `kperm`
    /// order, by enumeration:
    /// C = diag(1/120) − 9/120² + Σ_{d=1..4} (J_d + J_dᵀ), where J_d[a][b] is
    /// the fraction of the (5 + d)! orderings of 5 + d distinct values whose
    /// first five are in pattern a and whose last five are in pattern b.
    fn covariance_by_enumeration() -> Matrix {
        let p = 1.0 / N_PERMS as f64;
        let mut c: Matrix = vec![[-9.0 * p * p; N_PERMS]; N_PERMS];
        for (a, row) in c.iter_mut().enumerate() {
            row[a] += p;
        }
        for d in 1..=4usize {
            let n = 5 + d;
            let mut joint = vec![[0u64; N_PERMS]; N_PERMS];
            let mut total = 0u64;
            let mut items: Vec<u32> = (0..n as u32).collect();
            for_each_permutation(&mut items, |perm| {
                let first: [u32; 5] = perm[..5].try_into().expect("five values");
                let last: [u32; 5] = perm[d..].try_into().expect("five values");
                joint[kperm(&first, 0)][kperm(&last, 0)] += 1;
                total += 1;
            });
            assert_eq!(total, (1..=n as u64).product::<u64>());
            for a in 0..N_PERMS {
                for b in 0..N_PERMS {
                    c[a][b] += (joint[a][b] + joint[b][a]) as f64 / total as f64;
                }
            }
        }
        c
    }

    fn mat_mul(a: &[[f64; N_PERMS]], b: &[[f64; N_PERMS]]) -> Matrix {
        let mut out: Matrix = vec![[0.0; N_PERMS]; N_PERMS];
        for (row_out, row_a) in out.iter_mut().zip(a) {
            for (&aik, row_b) in row_a.iter().zip(b) {
                for (o, &bkj) in row_out.iter_mut().zip(row_b) {
                    *o += aik * bkj;
                }
            }
        }
        out
    }

    fn max_abs_diff(a: &[[f64; N_PERMS]], b: &[[f64; N_PERMS]]) -> f64 {
        a.iter()
            .flatten()
            .zip(b.iter().flatten())
            .map(|(x, y)| (x - y).abs())
            .fold(0.0, f64::max)
    }

    /// Dieharder's table is a generalised inverse of the covariance rebuilt
    /// from scratch: C·P·C = C, P·C·P = P, and trace(C·P) = rank(C) = 96,
    /// the degrees of freedom.  NumPy on the same C found max |C·P·C − C| of
    /// 3.0·10⁻¹⁴ (largest |C| entry 0.0109) and a relative P·C·P error of
    /// 2.5·10⁻¹².
    #[test]
    fn pseudo_inverse_is_a_generalised_inverse_of_the_enumerated_covariance() {
        let c = covariance_by_enumeration();
        for row in &c {
            assert!(
                row.iter().sum::<f64>().abs() < 1e-15,
                "a row of C sums to 0"
            );
        }
        let cp = mat_mul(&c, &PSEUDO_INVERSE);
        let cpc = mat_mul(&cp, &c);
        let err = max_abs_diff(&cpc, &c);
        assert!(err < 1e-12, "max |CPC − C| = {err:e}");

        let pcp = mat_mul(&mat_mul(&PSEUDO_INVERSE, &c), &PSEUDO_INVERSE);
        let scale = PSEUDO_INVERSE
            .iter()
            .flatten()
            .fold(0.0f64, |m, x| m.max(x.abs()));
        let rel = max_abs_diff(&pcp, &PSEUDO_INVERSE) / scale;
        assert!(rel < 1e-9, "max |PCP − P| / max |P| = {rel:e}");

        let trace: f64 = cp.iter().enumerate().map(|(i, row)| row[i]).sum();
        assert!((trace - 96.0).abs() < 1e-6, "trace(CP) = {trace}");
    }

    #[test]
    fn short_inputs_skip_and_constant_input_fails() {
        assert!(operm5_dieharder(&[]).skipped());
        assert!(operm5_dieharder(&vec![7; WORDS - 1]).skipped());
        let r = operm5_dieharder(&vec![7; WORDS]);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }

    /// Streams from separately seeded PCG64 generators.
    const SMOKE_STREAMS: u64 = 100;

    /// A small, fixed version of the null calibration in the module
    /// documentation: over 100 PCG64 streams the p-values pass a KS test and
    /// few fall below 0.01 (Binomial(100, 0.01) exceeds 5 with probability
    /// 5·10⁻⁴).
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "100 million-word streams; runs under cargo test --release"
    )]
    fn null_streams_give_uniform_p_values() {
        let mut p: Vec<f64> = (0..SMOKE_STREAMS)
            .map(|i| {
                let mut rng = Pcg64::new(u128::from(i), u128::from(SMOKE_STREAMS));
                operm5_dieharder(&rng.collect_u32s(WORDS)).p_value
            })
            .collect();
        let below = p.iter().filter(|&&x| x < 0.01).count();
        assert!(below <= 5, "{below} of {SMOKE_STREAMS} below 0.01");
        let ks = ks_test(&mut p);
        assert!(ks > 1e-3, "KS p = {ks}");
    }
}
