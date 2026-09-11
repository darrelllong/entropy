//! Overlapping sums, as Marsaglia's `diehard.f` computes them.  Result name:
//! `diehard_historical::overlapping_sums_fortran`.
//!
//! # What it is
//!
//! Words are floated to uniforms U on [−√3, √3), with mean 0 and variance 1.
//! From 199 of them the 100 overlapping sums y(j) = U(j) + … + U(j + 99) are
//! nearly normal, with the Toeplitz covariance T(i, j) = 100 − |i − j|.  A
//! linear map x = M y with M T Mᵀ = I, the inverse of T's Cholesky factor,
//! whose rows have at most three nonzero entries, makes the 100 x's
//! uncorrelated with unit variance.  Each x goes through Φ and then through
//! Marsaglia's table f, and an Anderson–Darling statistic asks whether the
//! 100 results are uniform.  That is one inner test.  100 inner results get
//! an Anderson–Darling test of their own, ten of those a last one, and the
//! upper tail of that last is the p-value.  199 000 words in all.
//!
//! # Reference followed
//!
//! Marsaglia's `cdosum`, `fortran/diehard.f` lines 1386–1471
//! [pubs/diehard-fortran-1996.tar.gz]:
//!
//! - `uni()` is `jtbl()*.806549e-9` (line 1401): the word read as a signed
//!   integer, times 2√3/2³².
//! - 199 uniforms per inner test; y(1) = U(1) + … + U(100) and
//!   y(j) = y(j − 1) − U(j − 1) + U(j + 99) (lines 1415–1420).
//! - With m = 100: x(1) = y(1)/√m,
//!   x(2) = −(m − 1)·y(1)/√(m(2m − 1)) + √(m/(2m − 1))·y(2), and for i ≥ 3,
//!   with a = 2m + 2 − i and b = 4m + 2 − 2i,
//!   x(i) = y(1)/√(ab) − √((a − 1)/(b + 2))·y(i − 1) + √(a/b)·y(i)
//!   (lines 1429–1436).  The first term reads y(1), not y(i − 2).
//! - p = Φ(x(i)), h = 100p, j = ⌊h⌋, and the value passed on is
//!   f(j) + (h − j)(f(j + 1) − f(j)) (lines 1441–1444), with the 101-entry
//!   table f of lines 1390–1400.
//! - Three Anderson–Darling layers: over the 100 values (line 1446), over the
//!   100 inner results (line 1449), and over the ten outer results
//!   (line 1461).
//!
//! # Departures from `diehard.f`
//!
//! - Arithmetic is `f64`, and the scale is 2√3/2³² exactly; the Fortran is
//!   `REAL*4` throughout, with the scale truncated to 8.06549·10⁻¹⁰.
//! - The Anderson–Darling distribution function is
//!   [`crate::math::anderson_darling_cdf`] (Marsaglia and Marsaglia 2004) at
//!   every layer.  `KSTEST` computes the same statistic, floors each product
//!   at 10⁻²⁰ as this module does, and then maps it through Marsaglia's
//!   older asymptotic approximation, computing a small-sample correction that
//!   it does not return (`A=P+E` is set and `P` returned, line 1707).
//! - The p-value is 1 − CDF of the last layer, small for a bad fit.  DIEHARD
//!   prints the CDF value, which is near 1 for a bad fit; the note shows it.
//!   A² is unchanged by u → 1 − u, so the inner layers are the same either
//!   way.
//! - j is capped at 99.  At h = 100, which needs Φ(x) = 1 (x above about 8.3
//!   in `f64`), the Fortran reads f(101), past the end of its table; the cap
//!   gives the value 1.
//!
//! # Goldens
//!
//! On words 1 to 199 000 of the input the DIEHARD fidelity review gave its
//! gfortran build of `diehard.f` (an "aligned" build, whose `jkreset` also
//! resets `jtbl`'s buffer), these three layers with Marsaglia's `KSTEST`
//! formula in place of `anderson_darling_cdf` reproduce the ten outer values
//! and the last layer's value that build printed to within 6·10⁻⁵, the size of
//! its `REAL*4` rounding.  With `anderson_darling_cdf` the ten move by up to
//! 5.4·10⁻³: over 0.1 ≤ A² ≤ 10 the two distribution functions differ by up
//! to 4.3·10⁻³ at n = 10 and 4·10⁻⁴ at n = 100.
//!
//! # Calibration evidence
//!
//! - **The map.**  M T Mᵀ = I to 10⁻¹² (a test below).  With Dieharder's
//!   y(i − 2) in place of y(1) it is not: the largest off-diagonal entry is
//!   0.068 and the diagonal reaches 1.0094 (also a test).
//! - **The table.**  For i ≥ 3, x(i) is dominated by (U(i + 99) − U(i − 1))/√2,
//!   a near-triangular variable, so Φ(x) is far from uniform: the table f is
//!   the distribution of Φ(x) over the 100 x's pooled.  Simulating 10⁶ inner
//!   tests (10⁸ x's, NumPy) gave P(Φ(x) < 0.01) = 0.001738 against
//!   f(1) = 0.0017, and 92 of the 99 interior entries within their rounding
//!   plus three standard errors (largest gap 0.00028, at f(68)); columns 3 to
//!   100 alone match worse (81 of 99, P(Φ(x) < 0.01) = 0.001656).  A test
//!   repeats the f(1) check on 2·10⁷ x's.
//! - **Null simulation.**  100 000 streams of 199 000 words, each from a
//!   separately seeded PCG64 generator: p < 0.01 in 997 (0.997%; binomial
//!   standard deviation 0.031%) and p < 0.001 in 88 (0.088%; 0.010%), with
//!   p > 0.99 in 0.98%; a Kolmogorov–Smirnov test of the 100 000 p-values gives
//!   p = 0.65.  A test below runs a fixed 400-stream version under
//!   `cargo test --release`.
//!
//! # Dieharder's verdict
//!
//! Dieharder 3.31.1 lists `diehard_sums` as "Do Not Use" (`dieharder -l`,
//! test 14; `dieharder/list_tests.c` lines 31–36), and its source calls the
//! test "completely useless in every sense of the word.  It is broken, and it
//! is so broken that there is no point in trying to fix it", reporting that
//! the final KS test "CONVERGES to a non-zero pvalue of 0.09702690 for ALL rngs
//! tested" (`libdieharder/diehard_sums.c` lines 31–44).  That verdict is on
//! Dieharder's transcription, which is not the computation above:
//!
//! - it uses y\[t − 2\] where `diehard.f` uses y(1), leaving the y\[0\] line
//!   commented out (`diehard_sums.c` lines 249–250), so its x's are
//!   correlated;
//! - it drops the table f and treats Φ(x) as uniform (line 251);
//! - it takes a Kolmogorov–Smirnov test over the 100 values (line 269), and
//!   Dieharder's default of 100 p-samples, instead of Marsaglia's layers.
//!
//! This crate's module removed in commit 3b41af8 copied that transcription
//! (y\[t − 2\], no table, 10 repeats of 200 words and KS at both levels).
//! Simulated the same way over 200 000 streams, its final p-value fell below
//! 0.01 in 5.03% and below 0.001 in 0.95%: it was miscalibrated.
//!
//! # Why it is outside the default battery
//!
//! Commit 3b41af8 removed the transcription on the strength of Dieharder's
//! verdict, which does not carry over to the Fortran: as written, Marsaglia's
//! test is calibrated at the resolution DIEHARD reports (above).  It stays out
//! of the default battery because adding slots to that battery is a decision
//! of its own, and because the battery already tests sums of uniforms with
//! [`crate::dieharder::lagged_sums`], Brown's `rgb_lagged_sums`.
//!
//! # Author
//! George Marsaglia, DIEHARD (1995).

use super::anderson_darling_statistic;
use crate::{
    math::{anderson_darling_cdf, normal_cdf},
    result::TestResult,
};

/// Result name.
const NAME: &str = "diehard_historical::overlapping_sums_fortran";
/// m: uniforms in each sum, and sums in each inner test.
const M: usize = 100;
/// Uniforms per inner test: m for the first sum and one more for each of the
/// other m − 1.
const UNIFORMS_PER_INNER: usize = 2 * M - 1;
/// Inner tests per outer result.
const INNER_PER_OUTER: usize = 100;
/// Outer results in the last layer.
const OUTER: usize = 10;
/// Words the test reads: 10 × 100 × 199.
pub const WORDS: usize = OUTER * INNER_PER_OUTER * UNIFORMS_PER_INNER;

/// Marsaglia's table f (`fortran/diehard.f` lines 1390–1400): f(j) is his
/// value of P(Φ(x) < j/100) for the x's of an inner test.  Its entry 0.3180,
/// f(31), is his datum, not an approximation to 1/π.
#[allow(clippy::approx_constant)]
const F_TABLE: [f64; 101] = [
    0.0, 0.0017, 0.0132, 0.0270, 0.0406, 0.0538, 0.0665, 0.0787, 0.0905, 0.1020, 0.1133, 0.1242,
    0.1349, 0.1454, 0.1557, 0.1659, 0.1760, 0.1859, 0.1957, 0.2054, 0.2150, 0.2246, 0.2341, 0.2436,
    0.2530, 0.2623, 0.2716, 0.2809, 0.2902, 0.2995, 0.3087, 0.3180, 0.3273, 0.3366, 0.3459, 0.3552,
    0.3645, 0.3739, 0.3833, 0.3928, 0.4023, 0.4118, 0.4213, 0.4309, 0.4406, 0.4504, 0.4602, 0.4701,
    0.4800, 0.4900, 0.5000, 0.5100, 0.5199, 0.5299, 0.5397, 0.5495, 0.5593, 0.5690, 0.5787, 0.5882,
    0.5978, 0.6073, 0.6167, 0.6260, 0.6354, 0.6447, 0.6540, 0.6632, 0.6724, 0.6817, 0.6910, 0.7003,
    0.7096, 0.7189, 0.7282, 0.7375, 0.7468, 0.7562, 0.7657, 0.7752, 0.7848, 0.7944, 0.8041, 0.8140,
    0.8239, 0.8340, 0.8442, 0.8545, 0.8650, 0.8757, 0.8867, 0.8980, 0.9095, 0.9214, 0.9337, 0.9464,
    0.9595, 0.9731, 0.9868, 0.9983, 1.0,
];

/// Overlapping sums as `diehard.f` computes them, on the first [`WORDS`]
/// words.
///
/// Reports SKIP for fewer than [`WORDS`] words.  See the module
/// documentation for the statistic and its departures from `diehard.f`.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn overlapping_sums_fortran(words: &[u32]) -> TestResult {
    if words.len() < WORDS {
        return TestResult::insufficient(NAME, "need 199 000 words");
    }
    let mut outer = outer_results(&words[..WORDS], anderson_darling_cdf);
    let a2 = anderson_darling_statistic(&mut outer);
    let cdf = anderson_darling_cdf(OUTER, a2);
    TestResult::with_note(
        NAME,
        1.0 - cdf,
        format!("{WORDS} words, A²={a2:.4} over {OUTER} outer results, CDF={cdf:.6}"),
    )
}

/// The ten outer results of the first two layers on exactly [`WORDS`] words,
/// each A² mapped through `ad_cdf(n, A²)`.
fn outer_results(words: &[u32], ad_cdf: impl Fn(usize, f64) -> f64) -> [f64; OUTER] {
    let map = Whitening::new();
    let mut blocks = words.chunks_exact(UNIFORMS_PER_INNER);
    let mut outer = [0.0; OUTER];
    for result in outer.iter_mut() {
        let mut inner = [0.0; INNER_PER_OUTER];
        for u in inner.iter_mut() {
            let block = blocks.next().expect("layers reads exactly WORDS words");
            let uniforms: [f64; UNIFORMS_PER_INNER] = std::array::from_fn(|k| uniform(block[k]));
            let mut values = map.apply(&uniforms).map(through_table);
            *u = ad_cdf(M, anderson_darling_statistic(&mut values));
        }
        *result = ad_cdf(INNER_PER_OUTER, anderson_darling_statistic(&mut inner));
    }
    outer
}

/// `uni()` of `diehard.f` line 1401: the word as a signed integer, scaled
/// by 2√3/2³² to a uniform on [−√3, √3) with variance 1.
fn uniform(word: u32) -> f64 {
    f64::from(word as i32) * (12f64.sqrt() / 4_294_967_296.0)
}

/// Φ(x) mapped through Marsaglia's table f by linear interpolation.
fn through_table(x: f64) -> f64 {
    let h = 100.0 * normal_cdf(x);
    let j = (h as usize).min(M - 1);
    F_TABLE[j] + (h - j as f64) * (F_TABLE[j + 1] - F_TABLE[j])
}

/// The rows of M: x(i) = first[i]·y(1) + prev[i]·y(i − 1) + own[i]·y(i),
/// 0-based, with prev[0] = prev[1] = own[0] = 0.
struct Whitening {
    first: [f64; M],
    prev: [f64; M],
    own: [f64; M],
}

impl Whitening {
    /// The coefficients of `fortran/diehard.f` lines 1429–1436.
    fn new() -> Self {
        let m = M as f64;
        let mut w = Self {
            first: [0.0; M],
            prev: [0.0; M],
            own: [0.0; M],
        };
        w.first[0] = 1.0 / m.sqrt();
        w.first[1] = -(m - 1.0) / (m * (m + m - 1.0)).sqrt();
        w.own[1] = (m / (m + m - 1.0)).sqrt();
        for idx in 2..M {
            let i = (idx + 1) as f64; // diehard.f's 1-based i
            let a = m + m + 2.0 - i;
            let b = 4.0 * m + 2.0 - i - i;
            w.first[idx] = 1.0 / (a * b).sqrt();
            w.prev[idx] = -((a - 1.0) / (b + 2.0)).sqrt();
            w.own[idx] = (a / b).sqrt();
        }
        w
    }

    /// x = M y for the overlapping sums y of `u`.
    fn apply(&self, u: &[f64; UNIFORMS_PER_INNER]) -> [f64; M] {
        let mut y = [0.0f64; M];
        let mut sum: f64 = u[..M].iter().sum();
        y[0] = sum;
        for (yj, (&old, &new)) in y[1..].iter_mut().zip(u.iter().zip(&u[M..])) {
            sum = sum - old + new;
            *yj = sum;
        }
        std::array::from_fn(|i| {
            let prev = if i > 0 { self.prev[i] * y[i - 1] } else { 0.0 };
            self.first[i] * y[0] + prev + self.own[i] * y[i]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::{anderson_darling_statistic, oracle};
    use super::{
        outer_results, overlapping_sums_fortran, through_table, uniform, Whitening, F_TABLE, M,
        OUTER, UNIFORMS_PER_INNER, WORDS,
    };
    use crate::{
        math::{ks_test, normal_cdf},
        rng::{Pcg64, Rng},
    };

    /// Row i of M as a dense vector; `dieharder` moves the y(1) coefficient
    /// to y(i − 2) for i ≥ 3, as `diehard_sums.c` line 250 does.
    fn row(w: &Whitening, i: usize, dieharder: bool) -> [f64; M] {
        let mut r = [0.0; M];
        let first_col = if dieharder && i >= 2 { i - 2 } else { 0 };
        r[first_col] += w.first[i];
        if i > 0 {
            r[i - 1] += w.prev[i];
        }
        r[i] += w.own[i];
        r
    }

    /// M T Mᵀ with T(i, j) = m − |i − j|.
    fn whitened_covariance(dieharder: bool) -> Vec<[f64; M]> {
        let w = Whitening::new();
        let rows: Vec<[f64; M]> = (0..M).map(|i| row(&w, i, dieharder)).collect();
        let t = |j: usize, l: usize| (M - j.abs_diff(l)) as f64;
        rows.iter()
            .map(|ri| {
                std::array::from_fn(|k| {
                    let mut s = 0.0;
                    for (j, &rij) in ri.iter().enumerate() {
                        for (l, &rkl) in rows[k].iter().enumerate() {
                            s += rij * t(j, l) * rkl;
                        }
                    }
                    s
                })
            })
            .collect()
    }

    #[test]
    fn whitening_gives_the_identity_covariance() {
        for (i, r) in whitened_covariance(false).iter().enumerate() {
            for (k, &v) in r.iter().enumerate() {
                let want = if i == k { 1.0 } else { 0.0 };
                assert!((v - want).abs() < 1e-12, "(M T Mᵀ)[{i}][{k}] = {v}");
            }
        }
    }

    /// Dieharder's y\[t − 2\] leaves the x's correlated.  NumPy on the same
    /// matrices: largest off-diagonal 0.067577, largest diagonal 1.009416.
    #[test]
    fn dieharders_transcription_does_not_whiten() {
        let k = whitened_covariance(true);
        let (mut off, mut diag) = (0.0f64, 0.0f64);
        for (i, r) in k.iter().enumerate() {
            for (j, &v) in r.iter().enumerate() {
                if i == j {
                    diag = diag.max(v);
                } else {
                    off = off.max(v.abs());
                }
            }
        }
        assert!((off - 0.067577).abs() < 1e-6, "off-diagonal {off}");
        assert!((diag - 1.009416).abs() < 1e-6, "diagonal {diag}");
    }

    /// The table runs from 0 to 1, rises, and is symmetric to within its
    /// rounding: f(j) + f(100 − j) is within 3·10⁻⁴ of 1 (f(30) + f(70) =
    /// 0.9997).
    #[test]
    fn table_is_a_symmetric_distribution_function() {
        assert_eq!((F_TABLE[0], F_TABLE[100]), (0.0, 1.0));
        assert!(F_TABLE.windows(2).all(|p| p[0] < p[1]));
        for j in 0..=100 {
            assert!(
                (F_TABLE[j] + F_TABLE[100 - j] - 1.0).abs() <= 3.0001e-4,
                "f({j})"
            );
        }
        assert_eq!(through_table(f64::INFINITY), 1.0);
        assert_eq!(through_table(f64::NEG_INFINITY), 0.0);
    }

    /// The ten outer results the review's aligned gfortran build printed for
    /// `in.bin` words 1 to 199 000 ("Test no. 1" to "Test no. 10", from
    /// `fortran/diehard.f` line 1452), and its `KSTEST` over them (line 1461).
    const FORTRAN_OUTER: [f64; OUTER] = [
        0.620414, 0.072143, 0.324695, 0.401280, 0.008539, 0.250802, 0.710268, 0.998711, 0.467575,
        0.126249,
    ];
    const FORTRAN_FINAL: f64 = 0.712980;

    /// This module's p-value on the same words, pinned.  It passes through
    /// `normal_cdf` 10⁵ times, so the tolerance leaves room for a more
    /// accurate `erfc`.
    const GOLDEN_P: f64 = 0.291_314_442_603_088_9;

    /// With Marsaglia's `KSTEST` formula in place of `anderson_darling_cdf`,
    /// the three layers reproduce every value the gfortran build printed to
    /// 10⁻⁴; the largest gaps, 5.9·10⁻⁵ (test 4) and 5.5·10⁻⁵ (the last
    /// layer), come from the Fortran's `REAL*4` arithmetic.
    #[test]
    fn layers_match_the_gfortran_build_on_the_review_input() {
        let words = oracle::words(WORDS);
        let mut fortran = outer_results(&words, oracle::diehard_kstest_cdf);
        for (k, (&got, &want)) in fortran.iter().zip(&FORTRAN_OUTER).enumerate() {
            assert!((got - want).abs() < 1e-4, "test {}: {got} vs {want}", k + 1);
        }
        let last = oracle::diehard_kstest_cdf(OUTER, anderson_darling_statistic(&mut fortran));
        assert!((last - FORTRAN_FINAL).abs() < 1e-4, "last layer {last}");

        let result = overlapping_sums_fortran(&words);
        assert!(
            (result.p_value - GOLDEN_P).abs() < 1e-4,
            "{:.17} {result}",
            result.p_value
        );
    }

    /// Inner tests simulated for the f(1) check: 2·10⁷ x's.
    const F1_INNER_TESTS: u64 = 200_000;

    /// Marsaglia's f(1) = 0.0017 is P(Φ(x) < 0.01) over the 100 x's of an
    /// inner test: the simulated rate is within the table's rounding (5·10⁻⁵)
    /// plus four standard errors (9.2·10⁻⁶ each).  NumPy over 10⁸ x's gave
    /// 0.001738.
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "2·10⁷ transformed sums; runs under cargo test --release"
    )]
    fn first_table_entry_matches_the_simulated_tail() {
        let map = Whitening::new();
        let mut rng = Pcg64::new(u128::from(F1_INNER_TESTS), 1);
        let mut below = 0u64;
        for _ in 0..F1_INNER_TESTS {
            let u: [f64; UNIFORMS_PER_INNER] = std::array::from_fn(|_| uniform(rng.next_u32()));
            below += map
                .apply(&u)
                .iter()
                .filter(|&&x| normal_cdf(x) < 0.01)
                .count() as u64;
        }
        let n = (F1_INNER_TESTS * M as u64) as f64;
        let rate = below as f64 / n;
        let se = (F_TABLE[1] * (1.0 - F_TABLE[1]) / n).sqrt();
        assert!(
            (rate - F_TABLE[1]).abs() < 5e-5 + 4.0 * se,
            "P(Φ(x) < 0.01) = {rate}"
        );
    }

    #[test]
    fn short_inputs_skip_and_constant_input_fails() {
        assert!(overlapping_sums_fortran(&[]).skipped());
        assert!(overlapping_sums_fortran(&vec![0; WORDS - 1]).skipped());
        let r = overlapping_sums_fortran(&vec![0; WORDS]);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }

    /// Streams from separately seeded PCG64 generators.
    const SMOKE_STREAMS: u64 = 400;

    /// A small, fixed version of the null calibration in the module
    /// documentation: over 400 PCG64 streams the p-values pass a KS test and
    /// few fall below 0.01 (Binomial(400, 0.01) exceeds 10 with probability
    /// 0.003).
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "400 streams of 199 000 words; runs under cargo test --release"
    )]
    fn null_streams_give_uniform_p_values() {
        let mut p: Vec<f64> = (0..SMOKE_STREAMS)
            .map(|i| {
                let mut rng = Pcg64::new(u128::from(i), u128::from(SMOKE_STREAMS));
                overlapping_sums_fortran(&rng.collect_u32s(WORDS)).p_value
            })
            .collect();
        let below = p.iter().filter(|&&x| x < 0.01).count();
        assert!(below <= 10, "{below} of {SMOKE_STREAMS} below 0.01");
        let ks = ks_test(&mut p);
        assert!(ks > 1e-3, "KS p = {ks}");
    }
}
