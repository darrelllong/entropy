//! A ziggurat for the standard normal, with its table derived here.
//!
//! The method is G. Marsaglia and W. W. Tsang, "The Ziggurat Method for
//! Generating Random Variables," *Journal of Statistical Software* 5(8),
//! 2000 \[pubs/marsaglia-tsang-2000-ziggurat.pdf\].  The area under the
//! half-normal density f(x) = e^{−x²/2} is cut into [`LAYERS`] pieces of
//! equal area v: one base piece holding the rectangle [0, r] × [0, f(r)]
//! together with the tail beyond r, and above it rectangles [0, xᵢ] × [f(xᵢ),
//! f(xᵢ₊₁)].  A draw picks a piece uniformly, then a point in it, and accepts
//! at once whenever the point is under the curve by construction.
//!
//! # The table
//!
//! Nothing here is tabulated from a publication.  Equal areas mean
//!
//! v = r·f(r) + ∫ᵣ^∞ f,  xᵢ₊₁ = f⁻¹(f(xᵢ) + v/xᵢ),  x₁ = r,
//!
//! with the tail integral √(π/2)·erfc(r/√2) from [`crate::math::erfc`] and
//! f⁻¹(y) = √(−2 ln y).  The recurrence must end at x_LAYERS = 0, which fixes
//! r; `derive` bisects for it and the tests check the closure, the equality of
//! the areas and the resulting distribution.  The derivation runs once.
//!
//! # What it samples
//!
//! The rectangles carry a 53-bit uniform, so inside them the draw lies on a
//! grid of that resolution; the tail is drawn by the rejection of the same
//! paper, from dense uniforms, and reaches as far as −ln(2⁻¹⁰⁷⁴)/r.  This is
//! a different sequence from [`Sample::normal`](crate::rng::Sample::normal),
//! which inverts Φ and is value-stable; both sample the same law, and this one
//! is the faster of the two.

use crate::{math::erfc, rng::Rng};
use std::sync::OnceLock;

/// Pieces of equal area the density is cut into: 256, so that a draw can take
/// its index from one byte of a word.
const LAYERS: usize = 256;

/// Bits of the uniform a rectangle draw uses, the full significand.
const UNIFORM_BITS: u32 = f64::MANTISSA_DIGITS;

/// Where the layer index sits in the drawn word, above the uniform.
const LAYER_SHIFT: u32 = UNIFORM_BITS;

/// Where the sign bit sits, above the layer index.
const SIGN_SHIFT: u32 = LAYER_SHIFT + 8;

/// √(π/2), the integral of f over the whole half-line.
const HALF_NORMAL_AREA: f64 = 1.253_314_137_315_500_3;

/// The bisection for r stops when the bracket is this wide; the closure
/// residual is smooth in r, and the table's areas are then equal to within a
/// few times ε.
const R_TOLERANCE: f64 = 1e-15;

/// The boundaries x₁ … x_LAYERS and the density at each, plus the base
/// piece's r and area.
pub(crate) struct Ziggurat {
    /// `x[i]` is xᵢ₊₁: `x[0]` = r and `x[LAYERS - 1]` = 0.
    x: [f64; LAYERS],
    /// `f[i]` = e^{−x[i]²/2}.
    f: [f64; LAYERS],
    /// The area of every piece.
    area: f64,
}

/// f(x) = e^{−x²/2}, the half-normal density without its normalisation.
fn density(x: f64) -> f64 {
    (-0.5 * x * x).exp()
}

/// ∫ₓ^∞ e^{−t²/2} dt = √(π/2)·erfc(x/√2).
fn tail_area(x: f64) -> f64 {
    HALF_NORMAL_AREA * erfc(x / std::f64::consts::SQRT_2)
}

impl Ziggurat {
    /// Build the table for a given base boundary r, returning the boundaries
    /// and the residual f(x_LAYERS) − 1, which is zero for the right r.
    ///
    /// The recurrence can leave the density above 1, which is outside f's
    /// range; that r is too small, and the residual is reported as positive so
    /// the bisection moves away from it.
    fn build(r: f64) -> (Self, f64) {
        let area = r * density(r) + tail_area(r);
        let mut table = Self {
            x: [0.0; LAYERS],
            f: [0.0; LAYERS],
            area,
        };
        table.x[0] = r;
        table.f[0] = density(r);
        // x[LAYERS - 1] is 0, where the density is 1, so the recurrence runs
        // to x[LAYERS - 2] and its next step must land exactly there.
        for i in 1..LAYERS - 1 {
            let y = table.f[i - 1] + area / table.x[i - 1];
            if y >= 1.0 || y.is_nan() {
                // The pieces are too tall: r is below the solution.
                return (table, y - 1.0);
            }
            table.x[i] = (-2.0 * y.ln()).sqrt();
            table.f[i] = y;
        }
        table.x[LAYERS - 1] = 0.0;
        table.f[LAYERS - 1] = 1.0;
        let residual = table.f[LAYERS - 2] + area / table.x[LAYERS - 2] - 1.0;
        (table, residual)
    }

    /// The table, derived once: bisect r for the closure f(x_LAYERS) = 1.
    ///
    /// The residual falls with r, and r lies between 1 and 8: at r = 1 the
    /// pieces are far too wide and at r = 8 far too narrow.
    pub(crate) fn derived() -> &'static Self {
        static TABLE: OnceLock<Ziggurat> = OnceLock::new();
        TABLE.get_or_init(|| {
            let (mut lo, mut hi) = (1.0f64, 8.0f64);
            while hi - lo > R_TOLERANCE * hi {
                let middle = 0.5 * (lo + hi);
                if Self::build(middle).1 > 0.0 {
                    lo = middle;
                } else {
                    hi = middle;
                }
            }
            Self::build(0.5 * (lo + hi)).0
        })
    }

    /// One standard normal variate.
    ///
    /// A word supplies the significand of a uniform, a layer index and a
    /// sign.  A point inside the layer's inner rectangle is returned at once;
    /// otherwise the point is accepted against the density, and the base
    /// layer's overhang is the tail.
    pub(crate) fn sample(&self, rng: &mut (impl Rng + ?Sized)) -> f64 {
        loop {
            let word = rng.next_u64();
            let uniform = (word & ((1 << UNIFORM_BITS) - 1)) as f64 / (1u64 << UNIFORM_BITS) as f64;
            let layer = (word >> LAYER_SHIFT) as usize % LAYERS;
            let negative = (word >> SIGN_SHIFT) & 1 == 1;
            let sign = if negative { -1.0 } else { 1.0 };
            if layer == 0 {
                // The base piece: the rectangle [0, r] × [0, f(r)] and the
                // tail beyond r, which the uniform selects in proportion to
                // its share of the piece's area.
                let position = uniform * self.area / self.f[0];
                if position < self.x[0] {
                    return sign * position;
                }
                return sign * self.tail(rng);
            }
            // Piece `layer` is [0, x[layer − 1]] × [f[layer − 1], f[layer]].
            let x = uniform * self.x[layer - 1];
            if x < self.x[layer] {
                // Left of the curve's crossing: under it already.
                return sign * x;
            }
            let (lower, upper) = (self.f[layer - 1], self.f[layer]);
            let height = lower + unit(rng) * (upper - lower);
            if height < density(x) {
                return sign * x;
            }
        }
    }

    /// The tail beyond r, by the rejection of Marsaglia and Tsang: with
    /// e₁ and e₂ independent exponentials, r + e₁/r is accepted when
    /// 2e₂ > (e₁/r)², which is the tail's law exactly.
    fn tail(&self, rng: &mut (impl Rng + ?Sized)) -> f64 {
        let r = self.x[0];
        loop {
            let excess = exponential(rng) / r;
            let height = exponential(rng);
            if 2.0 * height > excess * excess {
                return r + excess;
            }
        }
    }
}

/// A uniform in [0, 1) on the 2⁵³ grid, from one word.
fn unit(rng: &mut (impl Rng + ?Sized)) -> f64 {
    (rng.next_u64() >> (u64::BITS - UNIFORM_BITS)) as f64 / (1u64 << UNIFORM_BITS) as f64
}

/// −ln U for a dense uniform U: an exponential variate whose tail reaches
/// about 744, so the normal's tail is limited by the rejection, not by this.
fn exponential(rng: &mut (impl Rng + ?Sized)) -> f64 {
    loop {
        let u = crate::rng::Sample::unit_f64_dense(rng);
        if u > 0.0 {
            return -u.ln();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        math::{ks_test, normal_cdf},
        rng::{Pcg64, Sample, Xoshiro256},
    };

    /// Relative tolerance on the derived table's equal areas: the bisection
    /// leaves r accurate to about ε, and the last pieces, whose widths are
    /// smallest, carry that error amplified by the recurrence.
    const AREA: f64 = 1e-10;

    /// Standard errors allowed on a sample statistic.
    const SIGMAS: f64 = 5.0;

    /// The derivation closes: the boundaries descend to zero, every piece has
    /// the same area, and the base piece's rectangle plus tail is that area.
    #[test]
    fn the_derived_table_has_equal_areas() {
        let z = Ziggurat::derived();
        assert!(z.x[0] > 3.0 && z.x[0] < 4.0, "r = {}", z.x[0]);
        assert_eq!(z.x[LAYERS - 1], 0.0);
        for i in 1..LAYERS {
            assert!(z.x[i] < z.x[i - 1], "x[{i}] = {} not below", z.x[i]);
        }
        let base = z.x[0] * z.f[0] + tail_area(z.x[0]);
        assert!((base - z.area).abs() <= AREA * z.area, "base {base}");
        for i in 1..LAYERS {
            let piece = z.x[i - 1] * (z.f[i] - z.f[i - 1]);
            assert!(
                (piece - z.area).abs() <= AREA * z.area,
                "piece {i} = {piece} against {}",
                z.area
            );
        }
    }

    /// The draws are standard normal: a Kolmogorov–Smirnov test of Φ(z) over
    /// 200 000 draws, the mean and variance, and the tail frequencies.
    #[test]
    fn draws_follow_the_standard_normal() {
        const DRAWS: usize = 200_000;
        let z = Ziggurat::derived();
        let mut rng = Pcg64::new(7, 11);
        let values: Vec<f64> = (0..DRAWS).map(|_| z.sample(&mut rng)).collect();
        let mut uniforms: Vec<f64> = values.iter().map(|&v| normal_cdf(v)).collect();
        let p = ks_test(&mut uniforms);
        assert!(p > 0.001, "Kolmogorov–Smirnov p = {p}");

        let n = DRAWS as f64;
        let mean = values.iter().sum::<f64>() / n;
        assert!(mean.abs() < SIGMAS / n.sqrt(), "mean {mean}");
        let variance = values.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / n;
        assert!(
            (variance - 1.0).abs() < SIGMAS * (2.0 / n).sqrt(),
            "variance {variance}"
        );
        for threshold in [1.0, 2.0, 3.0, 4.0] {
            let seen = values.iter().filter(|v| v.abs() > threshold).count() as f64 / n;
            let want = 2.0 * normal_cdf(-threshold);
            let se = (want * (1.0 - want) / n).sqrt();
            assert!(
                (seen - want).abs() < SIGMAS * se,
                "|z| > {threshold}: {seen} against {want}"
            );
        }
    }

    /// The tail branch reaches past the table's r and is itself normal:
    /// conditioned on |z| > r, the excess should follow the same law.
    #[test]
    fn the_tail_branch_is_the_tail_of_the_law() {
        const DRAWS: usize = 400_000;
        let z = Ziggurat::derived();
        let mut rng = Xoshiro256::new(1, 2, 3, 4);
        let r = z.x[0];
        let tails: Vec<f64> = (0..DRAWS)
            .map(|_| z.sample(&mut rng))
            .filter(|v| v.abs() > r)
            .collect();
        assert!(!tails.is_empty(), "no draw reached the tail");
        // P(|Z| > x | |Z| > r) = erfc(x/√2)/erfc(r/√2).
        let mut uniforms: Vec<f64> = tails
            .iter()
            .map(|v| {
                1.0 - erfc(v.abs() / std::f64::consts::SQRT_2) / erfc(r / std::f64::consts::SQRT_2)
            })
            .collect();
        let p = ks_test(&mut uniforms);
        assert!(
            p > 0.001,
            "tail Kolmogorov–Smirnov p = {p} on {} draws",
            uniforms.len()
        );
    }

    /// The ziggurat and the inverse agree in distribution, on the same
    /// generator's words: a two-sample comparison through the exact CDF.
    #[test]
    fn it_agrees_with_the_inverse_in_distribution() {
        const DRAWS: usize = 100_000;
        let z = Ziggurat::derived();
        let mut a = Pcg64::new(21, 3);
        let mut b = Pcg64::new(22, 5);
        let mut zig: Vec<f64> = (0..DRAWS).map(|_| normal_cdf(z.sample(&mut a))).collect();
        let mut inverse: Vec<f64> = (0..DRAWS).map(|_| normal_cdf(b.normal())).collect();
        assert!(ks_test(&mut zig) > 0.001);
        assert!(ks_test(&mut inverse) > 0.001);
    }
}
