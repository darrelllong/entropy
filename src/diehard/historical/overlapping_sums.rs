//! Overlapping sums of uniforms.  Result name:
//! `diehard_historical::overlapping_sums`.
//!
//! # The statistic
//!
//! Each word w becomes a uniform U on [−√3, √3), with mean 0 and variance 1,
//! by reading it as a signed integer and scaling by 2√3/2³².  From 199 such
//! uniforms the m = 100 overlapping sums y(j) = U(j) + … + U(j + 99),
//! j = 1 … 100, are nearly normal with covariance T(i, j) = m − |i − j|.
//! The linear map x = M y, with rows
//!
//! - x(1) = y(1)/√m,
//! - x(2) = −(m − 1)·y(1)/√(m(2m − 1)) + √(m/(2m − 1))·y(2),
//! - x(i) = y(1)/√(ab) − √((a − 1)/(b + 2))·y(i − 1) + √(a/b)·y(i) for
//!   i ≥ 3, with a = 2m + 2 − i and b = 4m + 2 − 2i,
//!
//! satisfies M T Mᵀ = I, so the 100 x's are uncorrelated with unit variance.
//!
//! The x's are not normal.  For i ≥ 3, x(i) is dominated by
//! (U(i + 99) − U(i − 1))/√2, so Φ(x) is not uniform.  Its distribution is
//! corrected by f, the distribution function of Φ(x) pooled over the 100
//! coordinates: f(j) = (1/m) Σᵢ P(Φ(x(i)) < j/100).  Each value passed on is
//! f interpolated linearly at 100·Φ(x), and so is close to uniform.
//!
//! One inner test is the Anderson–Darling A² of those 100 values against
//! U(0, 1), mapped through its distribution function.  100 inner results get
//! an Anderson–Darling test of their own, ten of those a last one, and the
//! p-value is the upper tail of that last.  The test reads 199 000 words:
//! 10 × 100 × 199.
//!
//! # Computing f
//!
//! Each x(i) is a linear combination Σₖ cᵢₖ Uₖ of independent uniforms, so
//! its characteristic function is φᵢ(t) = Πₖ sinc(√3 cᵢₖ t), a product of at
//! most five distinct factors raised to their multiplicities.  By the
//! Gil-Pelaez inversion formula, and because the pooled distribution is
//! symmetric,
//!
//! f(j) = ½ + (1/π) ∫₀^∞ sin(t z) φ̄(t) / t dt,  z = Φ⁻¹(j/100),
//!
//! with φ̄ the mean of the 100 φᵢ.  φ̄ decays fast enough that Simpson's rule
//! on [0, 100] with step 0.05 gives every entry to better than 10⁻¹⁰
//! (checked against step 0.01 on [0, 400]).  The table is computed once per
//! process.  The 32-bit discreteness of U is ignored.
//!
//! # Calibration
//!
//! 100 000 streams of 199 000 words, each from a separately seeded PCG64
//! generator, gave p < 0.01 in 0.97% (binomial standard deviation 0.03%) and
//! p < 0.001 in 0.11% (0.01%), with a Kolmogorov–Smirnov p of 0.08 over the
//! 100 000 p-values.  A test below runs a fixed 400-stream version under
//! `cargo test --release`.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).
//! J. Gil-Pelaez, "Note on the inversion theorem", *Biometrika* 38 (1951),
//! for the inversion formula.

use super::anderson_darling_statistic;
use crate::{
    math::{anderson_darling_cdf, normal_cdf},
    result::TestResult,
};
use std::sync::OnceLock;

/// Result name.
const NAME: &str = "diehard_historical::overlapping_sums";
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
/// Cells of the table f: f(0) … f(100).
const TABLE_CELLS: usize = 100;
/// Upper limit of the inversion integral.
const INVERSION_LIMIT: f64 = 100.0;
/// Simpson step of the inversion integral.
const INVERSION_STEP: f64 = 0.05;

/// Overlapping sums on the first [`WORDS`] words.
///
/// Reports SKIP for fewer than [`WORDS`] words.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn overlapping_sums(words: &[u32]) -> TestResult {
    if words.len() < WORDS {
        return TestResult::insufficient(NAME, "need 199 000 words");
    }
    let mut outer = outer_results(&words[..WORDS]);
    let a2 = anderson_darling_statistic(&mut outer);
    TestResult::with_note(
        NAME,
        1.0 - anderson_darling_cdf(OUTER, a2),
        format!("{WORDS} words, A²={a2:.4} over {OUTER} outer results"),
    )
}

/// The ten outer results of the first two layers on exactly [`WORDS`] words.
fn outer_results(words: &[u32]) -> [f64; OUTER] {
    let map = Whitening::new();
    let table = pooled_table();
    let mut blocks = words.chunks_exact(UNIFORMS_PER_INNER);
    let mut outer = [0.0; OUTER];
    for result in outer.iter_mut() {
        let mut inner = [0.0; INNER_PER_OUTER];
        for u in inner.iter_mut() {
            let block = blocks.next().expect("exactly WORDS words");
            let uniforms: [f64; UNIFORMS_PER_INNER] = std::array::from_fn(|k| uniform(block[k]));
            let mut values = map.apply(&uniforms).map(|x| through_table(table, x));
            *u = anderson_darling_cdf(M, anderson_darling_statistic(&mut values));
        }
        *result = anderson_darling_cdf(INNER_PER_OUTER, anderson_darling_statistic(&mut inner));
    }
    outer
}

/// The word as a signed integer, scaled by 2√3/2³² to a uniform on
/// [−√3, √3) with variance 1.
fn uniform(word: u32) -> f64 {
    f64::from(word as i32) * (12f64.sqrt() / 4_294_967_296.0)
}

/// Φ(x) mapped through the table f by linear interpolation.
fn through_table(table: &[f64; TABLE_CELLS + 1], x: f64) -> f64 {
    let h = TABLE_CELLS as f64 * normal_cdf(x);
    let j = (h as usize).min(TABLE_CELLS - 1);
    table[j] + (h - j as f64) * (table[j + 1] - table[j])
}

/// The table f, computed once (see the module documentation).
fn pooled_table() -> &'static [f64; TABLE_CELLS + 1] {
    static TABLE: OnceLock<[f64; TABLE_CELLS + 1]> = OnceLock::new();
    TABLE.get_or_init(|| pooled_table_with(INVERSION_STEP, INVERSION_LIMIT))
}

/// The table f by Simpson's rule with step `step` on [0, `limit`].
fn pooled_table_with(step: f64, limit: f64) -> [f64; TABLE_CELLS + 1] {
    let intervals = 2 * ((limit / step) as usize).div_ceil(2);
    let map = Whitening::new();
    let factors: Vec<Vec<(f64, i32)>> = (0..M).map(|i| map.sinc_factors(i)).collect();
    // Simpson weight times φ̄(t)/t at each node; the node t = 0 is handled
    // through its limit, sin(tz)/t → z.
    let nodes: Vec<(f64, f64)> = (0..=intervals)
        .map(|k| {
            let t = k as f64 * step;
            let weight = match k {
                0 => 1.0,
                _ if k == intervals => 1.0,
                _ if k % 2 == 1 => 4.0,
                _ => 2.0,
            };
            let mean_cf = factors
                .iter()
                .map(|f| characteristic_function(f, t))
                .sum::<f64>()
                / M as f64;
            (t, weight * step / 3.0 * mean_cf)
        })
        .collect();
    std::array::from_fn(|j| match j {
        0 => 0.0,
        TABLE_CELLS => 1.0,
        _ => {
            let z = normal_quantile(j as f64 / TABLE_CELLS as f64);
            let integral: f64 = nodes
                .iter()
                .map(|&(t, w)| {
                    if t == 0.0 {
                        w * z
                    } else {
                        w * (t * z).sin() / t
                    }
                })
                .sum();
            0.5 + integral / std::f64::consts::PI
        }
    })
}

/// Π sinc(√3·c·t)^multiplicity over the distinct coefficients c of one x: the
/// characteristic function of Σ c·U with U uniform on [−√3, √3).
fn characteristic_function(factors: &[(f64, i32)], t: f64) -> f64 {
    factors
        .iter()
        .map(|&(c, multiplicity)| {
            let a = 3f64.sqrt() * c * t;
            if a == 0.0 {
                1.0
            } else {
                (a.sin() / a).powi(multiplicity)
            }
        })
        .product()
}

/// Φ⁻¹(p) for 0 < p < 1, by bisection on [`normal_cdf`] to full precision.
fn normal_quantile(p: f64) -> f64 {
    let (mut lo, mut hi) = (-40.0f64, 40.0f64);
    while hi - lo > 1e-15 * hi.abs().max(1.0) {
        let mid = 0.5 * (lo + hi);
        if normal_cdf(mid) < p {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// The rows of M: x(i) = first[i]·y(1) + prev[i]·y(i − 1) + own[i]·y(i),
/// 0-based, with prev[0] = prev[1] = own[0] = 0.
struct Whitening {
    first: [f64; M],
    prev: [f64; M],
    own: [f64; M],
}

impl Whitening {
    /// The coefficients given in the module documentation.
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
            let i = (idx + 1) as f64; // 1-based, as in the formula
            let a = m + m + 2.0 - i;
            let b = 4.0 * m + 2.0 - i - i;
            w.first[idx] = 1.0 / (a * b).sqrt();
            w.prev[idx] = -((a - 1.0) / (b + 2.0)).sqrt();
            w.own[idx] = (a / b).sqrt();
        }
        w
    }

    /// The distinct coefficients of x(i) (0-based) on the uniforms
    /// U(0) … U(198), each with its multiplicity.
    fn sinc_factors(&self, i: usize) -> Vec<(f64, i32)> {
        let mut c = [0.0f64; UNIFORMS_PER_INNER];
        for (k, ck) in c.iter_mut().enumerate() {
            if k < M {
                *ck += self.first[i];
            }
            if i > 0 && (i - 1..i - 1 + M).contains(&k) {
                *ck += self.prev[i];
            }
            if (i..i + M).contains(&k) {
                *ck += self.own[i];
            }
        }
        let mut factors: Vec<(f64, i32)> = Vec::new();
        for ck in c.into_iter().filter(|&ck| ck != 0.0) {
            match factors
                .iter_mut()
                .find(|(v, _)| v.to_bits() == ck.to_bits())
            {
                Some((_, count)) => *count += 1,
                None => factors.push((ck, 1)),
            }
        }
        factors
    }

    /// Row i of M as a dense vector over y(0) … y(m − 1).
    #[cfg(test)]
    fn row(&self, i: usize) -> [f64; M] {
        let mut r = [0.0; M];
        r[0] += self.first[i];
        if i > 0 {
            r[i - 1] += self.prev[i];
        }
        r[i] += self.own[i];
        r
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
    use super::{
        overlapping_sums, pooled_table, pooled_table_with, through_table, uniform, Whitening, M,
        TABLE_CELLS, UNIFORMS_PER_INNER, WORDS,
    };
    use crate::{
        math::{ks_test, normal_cdf},
        rng::{Pcg64, Rng},
    };

    /// Every x(i) has at most five distinct coefficients, whose squares,
    /// weighted by multiplicity, sum to Var x(i) = 1.
    #[test]
    fn each_coordinate_has_few_distinct_unit_variance_coefficients() {
        let map = Whitening::new();
        for i in 0..M {
            let factors = map.sinc_factors(i);
            assert!(factors.len() <= 5, "x({i}): {} factors", factors.len());
            let variance: f64 = factors.iter().map(|&(c, m)| f64::from(m) * c * c).sum();
            assert!((variance - 1.0).abs() < 1e-12, "Var x({i}) = {variance}");
        }
    }

    /// The table is a distribution function on 0 … 100, symmetric because
    /// every x is, and agrees to 10⁻¹⁰ with a finer and longer integration.
    #[test]
    fn table_is_a_symmetric_distribution_function() {
        let table = pooled_table();
        assert_eq!((table[0], table[TABLE_CELLS]), (0.0, 1.0));
        assert!(table.windows(2).all(|p| p[0] < p[1]));
        for j in 0..=TABLE_CELLS {
            assert!(
                (table[j] + table[TABLE_CELLS - j] - 1.0).abs() < 1e-12,
                "f({j})"
            );
        }
        let fine = pooled_table_with(0.01, 400.0);
        for (j, (a, b)) in table.iter().zip(&fine).enumerate() {
            assert!((a - b).abs() < 1e-10, "f({j}): {a} vs {b}");
        }
        assert_eq!(through_table(table, f64::INFINITY), 1.0);
        assert_eq!(through_table(table, f64::NEG_INFINITY), 0.0);
    }

    /// Inner tests simulated for the check of f(1): 2·10⁷ x's.
    const F1_INNER_TESTS: u64 = 200_000;

    /// The rate at which Φ(x) < 0.01 over simulated inner tests agrees with
    /// f(1) to within four standard errors.
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "2·10⁷ transformed sums; runs under cargo test --release"
    )]
    fn first_table_entry_matches_the_simulated_tail() {
        let map = Whitening::new();
        let f1 = pooled_table()[1];
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
        let se = (f1 * (1.0 - f1) / n).sqrt();
        assert!(
            (rate - f1).abs() < 4.0 * se,
            "P(Φ(x) < 0.01) = {rate}, f(1) = {f1}"
        );
    }

    /// p-value on a fixed PCG64 stream, pinned.
    const GOLDEN_P: f64 = 0.120_754_929_669_773_3;

    #[test]
    fn p_value_on_a_fixed_stream_is_pinned() {
        let words = Pcg64::new(20_260_916, 199).collect_u32s(WORDS);
        let p = overlapping_sums(&words).p_value;
        assert!((p - GOLDEN_P).abs() < 1e-9, "{p:?}");
    }

    /// M T Mᵀ with T(i, j) = m − |i − j|, the covariance of the sums.
    #[test]
    fn whitening_gives_the_identity_covariance() {
        let w = Whitening::new();
        let rows: Vec<[f64; M]> = (0..M).map(|i| w.row(i)).collect();
        let t = |j: usize, l: usize| (M - j.abs_diff(l)) as f64;
        for (i, ri) in rows.iter().enumerate() {
            for (k, rk) in rows.iter().enumerate() {
                let mut v = 0.0;
                for (j, &rij) in ri.iter().enumerate() {
                    for (l, &rkl) in rk.iter().enumerate() {
                        v += rij * t(j, l) * rkl;
                    }
                }
                let want = if i == k { 1.0 } else { 0.0 };
                assert!((v - want).abs() < 1e-12, "(M T Mᵀ)[{i}][{k}] = {v}");
            }
        }
    }

    #[test]
    fn short_inputs_skip_and_constant_input_fails() {
        assert!(overlapping_sums(&[]).skipped());
        assert!(overlapping_sums(&vec![0; WORDS - 1]).skipped());
        let r = overlapping_sums(&vec![0; WORDS]);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }

    /// Streams from separately seeded PCG64 generators.
    const SMOKE_STREAMS: u64 = 400;

    /// Over 400 PCG64 streams the p-values pass a KS test and few fall below
    /// 0.01 (Binomial(400, 0.01) exceeds 10 with probability 0.003).
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "400 streams of 199 000 words; runs under cargo test --release"
    )]
    fn null_streams_give_uniform_p_values() {
        let mut p: Vec<f64> = (0..SMOKE_STREAMS)
            .map(|i| {
                let mut rng = Pcg64::new(u128::from(i), u128::from(SMOKE_STREAMS));
                overlapping_sums(&rng.collect_u32s(WORDS)).p_value
            })
            .collect();
        let below = p.iter().filter(|&&x| x < 0.01).count();
        assert!(below <= 10, "{below} of {SMOKE_STREAMS} below 0.01");
        let ks = ks_test(&mut p);
        assert!(ks > 1e-3, "KS p = {ks}");
    }
}
