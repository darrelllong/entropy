//! OPERM5, the overlapping 5-permutation test.  Result name:
//! `diehard_historical::operm5`.
//!
//! # The statistic
//!
//! Each of n = 1 000 000 overlapping windows of five successive 32-bit words
//! w₀…w₄, w₁…w₅, … falls into one of the 5! = 120 orderings of its values.
//! Let c be the vector of the 120 counts.  Under the null every ordering has
//! probability 1/120, and c is asymptotically normal with mean n/120 and
//! covariance n·C, where C is the per-window covariance of the 120 ordering
//! indicators summed over the windows that overlap it:
//!
//! C = diag(1/120) − 9/120² + Σ_{d=1..4} (J_d + J_dᵀ),
//!
//! with J_d\[a\]\[b\] the probability that a window is in ordering a and the
//! window d places later is in ordering b.  J_d depends only on the relative
//! order of 5 + d distinct values, so it is computed exactly by enumerating
//! all (5 + d)! orderings.  Every row of C sums to zero, and C has rank
//! 5! − 4! = 96.  With P the Moore–Penrose pseudoinverse of C, the quadratic
//! form
//!
//! χ² = (c − n/120)ᵀ P (c − n/120) / n
//!
//! is asymptotically χ²(96).  The p-value is the upper tail Q(48, χ²/2).
//!
//! # Computation
//!
//! `ordering_index` numbers the orderings 0..120 by their Lehmer code.  C is
//! built by enumeration once per process, and P is formed from the
//! eigendecomposition of C (cyclic Jacobi rotations), inverting the 96
//! eigenvalues above a relative threshold of 10⁻⁹ and discarding the 24 that
//! vanish.  The tests check C·P·C = C, P·C·P = P and trace(C·P) = 96.
//!
//! Words are compared as unsigned integers.  Equal words, which occur with
//! probability about 10⁻⁹ per window, are ordered by position.
//!
//! # Calibration
//!
//! 10 000 streams of 1 000 005 words, each from a separately seeded PCG64
//! generator, gave p < 0.01 in 0.95% (binomial standard deviation 0.10%) and
//! p < 0.001 in 0.10% (0.03%), with a Kolmogorov–Smirnov p of 0.10 over the
//! 10 000 p-values.  A test below runs a fixed 100-stream version under
//! `cargo test --release`.
//!
//! # Scope
//!
//! The input is 1 000 005 words, one pass: five for the first window and one
//! more for each window after it.  Shorter input reports SKIP.  The suite's
//! [`crate::dieharder::permutations`] tests 5-permutations on non-overlapping
//! windows, where no covariance correction is needed.
//!
//! # References
//!
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995),
//! which defines the test on overlapping windows.  Robert G. Brown,
//! *Dieharder: A Random Number Test Suite* (2004–2011), which identified the
//! rank of the covariance as 96.

use crate::{math::igamc, result::TestResult};
use std::sync::OnceLock;

/// Result name.
const NAME: &str = "diehard_historical::operm5";
/// Overlapping windows counted.
const N_WINDOWS: usize = 1_000_000;
/// Values in each window.
const WINDOW: usize = 5;
/// Words the test reads: five for the first window, then one per window.
pub const WORDS: usize = N_WINDOWS + WINDOW;
/// Orderings of five values.
const N_ORDERINGS: usize = 120;
/// Rank of the covariance matrix, 5! − 4!.
const DF: f64 = 96.0;
/// Eigenvalues of C below this fraction of the largest are treated as zero.
const RANK_TOLERANCE: f64 = 1e-9;

/// A dense 120 × 120 matrix, row by row.
type Matrix = Vec<[f64; N_ORDERINGS]>;

/// OPERM5 on the first [`WORDS`] words.
///
/// Reports SKIP for fewer than [`WORDS`] words.  See the module documentation
/// for the statistic.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn operm5(words: &[u32]) -> TestResult {
    if words.len() < WORDS {
        return TestResult::insufficient(NAME, "need 1 000 005 words");
    }
    let chi_square = statistic(&words[..WORDS]);
    let p_value = igamc(DF / 2.0, chi_square / 2.0);
    TestResult::with_note(
        NAME,
        p_value,
        format!("n={N_WINDOWS}, df={DF}, χ²={chi_square:.4}"),
    )
    .chi_square(chi_square, DF)
}

/// χ² = xᵀ P x / n over the overlapping windows of exactly [`WORDS`] words,
/// with x the ordering counts minus n/120.
fn statistic(words: &[u32]) -> f64 {
    let mut counts = [0u64; N_ORDERINGS];
    for window in words.windows(WINDOW).take(N_WINDOWS) {
        let window: &[u32; WINDOW] = window.try_into().expect("windows of five");
        counts[ordering_index(window)] += 1;
    }
    let n = N_WINDOWS as f64;
    let expected = n / N_ORDERINGS as f64;
    let x: Vec<f64> = counts.iter().map(|&c| c as f64 - expected).collect();
    let quadratic_form: f64 = pseudo_inverse()
        .iter()
        .zip(&x)
        .map(|(row, &xi)| xi * row.iter().zip(&x).map(|(&p, &xj)| p * xj).sum::<f64>())
        .sum();
    quadratic_form / n
}

/// The Lehmer-code index, 0..120, of the ordering of `w`: position i
/// contributes the number of later values smaller than `w[i]`, weighted by
/// (4 − i)!.
fn ordering_index<T: Ord>(w: &[T; WINDOW]) -> usize {
    let mut index = 0;
    for i in 0..WINDOW {
        let smaller_later = w[i + 1..].iter().filter(|v| **v < w[i]).count();
        index = index * (WINDOW - i) + smaller_later;
    }
    index
}

/// The pseudoinverse P of the per-window covariance C, computed once.
fn pseudo_inverse() -> &'static Matrix {
    static P: OnceLock<Matrix> = OnceLock::new();
    P.get_or_init(|| pseudo_inverse_of(&covariance()))
}

/// The per-window covariance C of the 120 ordering indicators, by
/// enumeration (see the module documentation).
fn covariance() -> Matrix {
    let p = 1.0 / N_ORDERINGS as f64;
    let overlaps = 2 * (WINDOW - 1) + 1;
    let mut c: Matrix = vec![[-(overlaps as f64) * p * p; N_ORDERINGS]; N_ORDERINGS];
    for (a, row) in c.iter_mut().enumerate() {
        row[a] += p;
    }
    for d in 1..WINDOW {
        let (joint, total) = joint_counts(d);
        for (a, row) in c.iter_mut().enumerate() {
            for (b, entry) in row.iter_mut().enumerate() {
                *entry += (joint[a][b] + joint[b][a]) as f64 / total as f64;
            }
        }
    }
    c
}

/// Over all orderings of 5 + `d` distinct values, how often the first five
/// are in ordering a and the last five in ordering b; and the number of
/// orderings, (5 + d)!.
fn joint_counts(d: usize) -> (Vec<[u64; N_ORDERINGS]>, u64) {
    let mut joint = vec![[0u64; N_ORDERINGS]; N_ORDERINGS];
    let mut total = 0;
    let mut values: Vec<usize> = (0..WINDOW + d).collect();
    for_each_ordering(&mut values, |v| {
        let first: &[usize; WINDOW] = v[..WINDOW].try_into().expect("five values");
        let last: &[usize; WINDOW] = v[d..].try_into().expect("five values");
        joint[ordering_index(first)][ordering_index(last)] += 1;
        total += 1;
    });
    (joint, total)
}

/// Calls `visit` once on each ordering of `values`, rearranging them in
/// place by single swaps (Heap's method).
fn for_each_ordering(values: &mut [usize], mut visit: impl FnMut(&[usize])) {
    let n = values.len();
    let mut counter = vec![0; n];
    visit(values);
    let mut i = 0;
    while i < n {
        if counter[i] < i {
            let j = if i % 2 == 0 { 0 } else { counter[i] };
            values.swap(j, i);
            visit(values);
            counter[i] += 1;
            i = 0;
        } else {
            counter[i] = 0;
            i += 1;
        }
    }
}

/// The Moore–Penrose pseudoinverse of the symmetric matrix `c`: Σ vvᵀ/λ over
/// the eigenpairs whose eigenvalue exceeds [`RANK_TOLERANCE`] times the
/// largest.
fn pseudo_inverse_of(c: &Matrix) -> Matrix {
    let (values, vectors) = symmetric_eigen(c);
    let largest = values.iter().fold(0.0f64, |m, v| m.max(v.abs()));
    let mut p: Matrix = vec![[0.0; N_ORDERINGS]; N_ORDERINGS];
    for (k, &lambda) in values.iter().enumerate() {
        if lambda <= RANK_TOLERANCE * largest {
            continue;
        }
        for (i, row) in p.iter_mut().enumerate() {
            let vik = vectors[i][k] / lambda;
            for (j, entry) in row.iter_mut().enumerate() {
                *entry += vik * vectors[j][k];
            }
        }
    }
    p
}

/// Sweeps of Jacobi rotations before giving up on convergence.
const MAX_SWEEPS: usize = 64;

/// Eigenvalues and eigenvectors (as the columns of the second matrix) of the
/// symmetric matrix `a`, by cyclic Jacobi rotations.
#[allow(clippy::needless_range_loop)]
fn symmetric_eigen(a: &Matrix) -> ([f64; N_ORDERINGS], Matrix) {
    let n = N_ORDERINGS;
    let mut a = a.clone();
    let mut v: Matrix = vec![[0.0; N_ORDERINGS]; N_ORDERINGS];
    for (i, row) in v.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    for _ in 0..MAX_SWEEPS {
        let off: f64 = (0..n)
            .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
            .map(|(i, j)| a[i][j] * a[i][j])
            .sum();
        let diagonal: f64 = (0..n).map(|i| a[i][i] * a[i][i]).sum();
        if off <= f64::EPSILON * f64::EPSILON * diagonal {
            break;
        }
        for p in 0..n {
            for q in p + 1..n {
                if a[p][q] == 0.0 {
                    continue;
                }
                // The rotation that zeroes a[p][q]: t = tan θ, the smaller
                // root of t² + 2τt − 1 = 0.
                let tau = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
                let t = tau.signum() / (tau.abs() + (1.0 + tau * tau).sqrt());
                let t = if tau == 0.0 { 1.0 } else { t };
                let cos = 1.0 / (1.0 + t * t).sqrt();
                let sin = t * cos;
                for k in 0..n {
                    let (akp, akq) = (a[k][p], a[k][q]);
                    a[k][p] = cos * akp - sin * akq;
                    a[k][q] = sin * akp + cos * akq;
                }
                for k in 0..n {
                    let (apk, aqk) = (a[p][k], a[q][k]);
                    a[p][k] = cos * apk - sin * aqk;
                    a[q][k] = sin * apk + cos * aqk;
                }
                for row in v.iter_mut() {
                    let (vkp, vkq) = (row[p], row[q]);
                    row[p] = cos * vkp - sin * vkq;
                    row[q] = sin * vkp + cos * vkq;
                }
            }
        }
    }
    (std::array::from_fn(|i| a[i][i]), v)
}

#[cfg(test)]
mod tests {
    /// A row sum or entry of the exact covariance matrix.
    const EXACT_COVARIANCE: f64 = 1e-15;

    /// The trace of the covariance times its pseudoinverse, over 120 rows.
    const MATRIX_PRODUCT: f64 = 1e-8;

    /// Tolerance on the pinned χ² of a fixed stream.
    const GOLDEN_CHI_TOLERANCE: f64 = 1e-9;

    /// Tolerance on the p-value of that χ².
    const GOLDEN_P_TOLERANCE: f64 = 1e-12;

    use super::{
        covariance, for_each_ordering, operm5, ordering_index, pseudo_inverse, statistic, Matrix,
        N_ORDERINGS, WINDOW, WORDS,
    };
    use crate::{
        math::ks_test,
        rng::{Pcg64, Rng},
    };

    fn mat_mul(a: &Matrix, b: &Matrix) -> Matrix {
        let mut out: Matrix = vec![[0.0; N_ORDERINGS]; N_ORDERINGS];
        for (row_out, row_a) in out.iter_mut().zip(a) {
            for (&aik, row_b) in row_a.iter().zip(b) {
                for (o, &bkj) in row_out.iter_mut().zip(row_b) {
                    *o += aik * bkj;
                }
            }
        }
        out
    }

    fn max_abs(a: &Matrix) -> f64 {
        a.iter().flatten().fold(0.0, |m, x| m.max(x.abs()))
    }

    fn max_abs_diff(a: &Matrix, b: &Matrix) -> f64 {
        a.iter()
            .flatten()
            .zip(b.iter().flatten())
            .fold(0.0, |m, (x, y)| m.max((x - y).abs()))
    }

    #[test]
    fn ordering_index_numbers_the_120_orderings_once_each() {
        let mut seen = [false; N_ORDERINGS];
        let mut values: Vec<usize> = (0..WINDOW).collect();
        for_each_ordering(&mut values, |v| {
            let k = ordering_index::<usize>(v.try_into().expect("five values"));
            assert!(!seen[k], "index {k} repeated");
            seen[k] = true;
        });
        assert!(seen.iter().all(|&s| s));
        assert_eq!(ordering_index(&[1, 2, 3, 4, 5]), 0);
        assert_eq!(ordering_index(&[5, 4, 3, 2, 1]), N_ORDERINGS - 1);
    }

    /// Every row of C sums to zero, since the 120 indicators of one window sum
    /// to one; C is symmetric; and its diagonal entry for the increasing
    /// ordering has the value computed by hand from its overlaps.
    #[test]
    fn covariance_rows_sum_to_zero() {
        let c = covariance();
        for (a, row) in c.iter().enumerate() {
            assert!(row.iter().sum::<f64>().abs() < EXACT_COVARIANCE, "row {a}");
            for (b, &v) in row.iter().enumerate() {
                assert_eq!(v.to_bits(), c[b][a].to_bits(), "C[{a}][{b}]");
            }
        }
        // The increasing ordering overlaps itself d places on with
        // probability 1/(5 + d)!, so its variance is
        // 1/120 − 9/120² + 2·(1/720 + 1/5040 + 1/40320 + 1/362880).
        let p = 1.0 / 120.0;
        let want =
            p - 9.0 * p * p + 2.0 * (1.0 / 720.0 + 1.0 / 5040.0 + 1.0 / 40320.0 + 1.0 / 362880.0);
        assert!((c[0][0] - want).abs() < EXACT_COVARIANCE, "{}", c[0][0]);
    }

    /// P is the pseudoinverse of C: C·P·C = C, P·C·P = P, and
    /// trace(C·P) = rank(C) = 96, the degrees of freedom.
    #[test]
    fn pseudo_inverse_satisfies_the_penrose_conditions() {
        let c = covariance();
        let p = pseudo_inverse();
        let cp = mat_mul(&c, p);
        let cpc = mat_mul(&cp, &c);
        let err = max_abs_diff(&cpc, &c) / max_abs(&c);
        assert!(err < 1e-10, "max |CPC − C| / max |C| = {err:e}");
        let pcp = mat_mul(&mat_mul(p, &c), p);
        let err = max_abs_diff(&pcp, p) / max_abs(p);
        assert!(err < 1e-10, "max |PCP − P| / max |P| = {err:e}");
        let cp_transpose: Matrix = (0..N_ORDERINGS)
            .map(|i| std::array::from_fn(|j| cp[j][i]))
            .collect();
        let err = max_abs_diff(&cp, &cp_transpose);
        assert!(err < 1e-10, "CP is not symmetric: {err:e}");
        let trace: f64 = cp.iter().enumerate().map(|(i, row)| row[i]).sum();
        assert!((trace - 96.0).abs() < MATRIX_PRODUCT, "trace(CP) = {trace}");
    }

    /// χ² on a fixed PCG64 stream, pinned.
    const GOLDEN_CHI_SQUARE: f64 = 100.490_414_107_250_66;
    /// Its p-value, pinned.
    const GOLDEN_P: f64 = 0.356_772_043_317_471_6;

    #[test]
    fn result_on_a_fixed_stream_is_pinned() {
        let words = Pcg64::new(20_260_916, 5).collect_u32s(WORDS);
        let chi_square = statistic(&words);
        assert!(
            (chi_square - GOLDEN_CHI_SQUARE).abs() < GOLDEN_CHI_TOLERANCE,
            "{chi_square:?}"
        );
        let result = operm5(&words);
        assert!(
            (result.p_value - GOLDEN_P).abs() < GOLDEN_P_TOLERANCE,
            "{:?}",
            result.p_value
        );
    }

    #[test]
    fn short_inputs_skip_and_constant_input_fails() {
        assert!(operm5(&[]).skipped());
        assert!(operm5(&vec![7; WORDS - 1]).skipped());
        let r = operm5(&vec![7; WORDS]);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }

    /// Streams from separately seeded PCG64 generators.
    const SMOKE_STREAMS: u64 = 100;

    /// Over 100 PCG64 streams the p-values pass a KS test and few fall below
    /// 0.01 (Binomial(100, 0.01) exceeds 5 with probability 5·10⁻⁴).
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "100 million-word streams; runs under cargo test --release"
    )]
    fn null_streams_give_uniform_p_values() {
        let mut p: Vec<f64> = (0..SMOKE_STREAMS)
            .map(|i| {
                let mut rng = Pcg64::new(u128::from(i), u128::from(SMOKE_STREAMS));
                operm5(&rng.collect_u32s(WORDS)).p_value
            })
            .collect();
        let below = p.iter().filter(|&&x| x < 0.01).count();
        assert!(below <= 5, "{below} of {SMOKE_STREAMS} below 0.01");
        let ks = ks_test(&mut p);
        assert!(ks > 1e-3, "KS p = {ks}");
    }
}
