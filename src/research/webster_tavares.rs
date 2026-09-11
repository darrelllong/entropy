//! Webster–Tavares avalanche analysis.
//!
//! This module implements the dependence-matrix and avalanche-variable
//! correlation machinery described in:
//!
//! A. F. Webster and S. E. Tavares, "On the Design of S-Boxes,"
//! CRYPTO 1985 / LNCS 218, pp. 523-534.
//!
//! The paper defines:
//! - the strict avalanche criterion (SAC): each output bit should flip with
//!   probability one half when a single input bit is complemented
//! - pairwise independence of avalanche variables, measured by correlation
//!   coefficients between output-bit flip indicators
//!
//! For large input spaces the authors recommend random sampling rather than
//! exhaustive enumeration. This implementation follows that rule.
//!
//! # Degenerate pairs
//! The paper measures the dependence of two avalanche variables A and B by
//! their correlation coefficient ρ(A, B) = cov(A, B) / (σ(A) σ(B)) (pp. 526–527).
//! When either variable never or always flips over the sample its variance
//! is 0 and ρ is 0/0.  For a GF(2)-linear map such as Xorshift every pair is
//! degenerate, because the avalanche vector f(x) ⊕ f(x ⊕ eⱼ) = f(eⱼ) does
//! not depend on x.  Scoring those pairs as ρ = 0, the ideal value, would
//! report perfect independence for exactly the most structured maps, so they
//! are counted in [`AvalancheReport::bic_degenerate_pairs`] and left out of
//! the BIC maximum and mean, which are NaN when no pair has a defined
//! correlation.

use crate::seed::splitmix64;
use core::fmt;

/// Joint flip counts for one pair of avalanche variables.
#[derive(Debug, Clone, Copy, Default)]
struct CorrAccum {
    n: usize,
    ones_x: usize,
    ones_y: usize,
    ones_xy: usize,
}

impl CorrAccum {
    fn observe(&mut self, x: bool, y: bool) {
        self.n += 1;
        self.ones_x += usize::from(x);
        self.ones_y += usize::from(y);
        self.ones_xy += usize::from(x && y);
    }

    /// Pearson correlation of the two indicators, or `None` when either one
    /// is constant over the sample (zero variance makes ρ = 0/0).
    fn correlation(self) -> Option<f64> {
        let constant = |ones: usize| ones == 0 || ones == self.n;
        if constant(self.ones_x) || constant(self.ones_y) {
            return None;
        }
        let n = self.n as f64;
        let ex = self.ones_x as f64 / n;
        let ey = self.ones_y as f64 / n;
        let cov = (self.ones_xy as f64 / n) - ex * ey;
        let var_x = ex * (1.0 - ex);
        let var_y = ey * (1.0 - ey);
        Some(cov / (var_x * var_y).sqrt())
    }
}

/// Summary of a Webster–Tavares sampled avalanche analysis.
#[derive(Debug, Clone)]
pub struct AvalancheReport {
    /// Number of input bits whose single-bit complements were tested.
    pub input_bits: usize,
    /// Number of output bits observed from the transformation.
    pub output_bits: usize,
    /// Number of base inputs sampled.
    pub samples: usize,
    /// `true` if the full input space was enumerated exactly.
    pub exact: bool,
    /// Dependence matrix `A[i][j]`, output bit `i` vs input bit `j`.
    pub dependence: Vec<Vec<f64>>,
    /// Maximum absolute deviation of any dependence entry from 0.5.
    pub max_sac_bias: f64,
    /// Mean absolute deviation of dependence entries from 0.5.
    pub mean_sac_bias: f64,
    /// Root-mean-square SAC bias over the dependence matrix.
    pub rms_sac_bias: f64,
    /// Maximum absolute avalanche-variable correlation over the input-bit and
    /// output-bit-pair combinations whose correlation is defined; NaN when
    /// none is (see [`bic_degenerate_pairs`](Self::bic_degenerate_pairs)).
    pub max_bic_abs_corr: f64,
    /// Mean absolute avalanche-variable correlation over the same defined
    /// combinations; NaN when none is.
    pub mean_bic_abs_corr: f64,
    /// Number of input-bit and output-bit-pair combinations examined,
    /// `input_bits · output_bits · (output_bits − 1) / 2`.
    pub bic_pairs: usize,
    /// Combinations whose correlation is undefined because at least one of
    /// the two avalanche variables never or always flipped over the sample.
    /// They are excluded from the BIC maximum and mean (see the module docs).
    pub bic_degenerate_pairs: usize,
}

impl fmt::Display for AvalancheReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "samples={}{} input_bits={} output_bits={} sac[max={:.4}, mean={:.4}, rms={:.4}] bic[max|rho|={:.4}, mean|rho|={:.4}, degenerate={}/{}]",
            self.samples,
            if self.exact { " exact" } else { "" },
            self.input_bits,
            self.output_bits,
            self.max_sac_bias,
            self.mean_sac_bias,
            self.rms_sac_bias,
            self.max_bic_abs_corr,
            self.mean_bic_abs_corr,
            self.bic_degenerate_pairs,
            self.bic_pairs,
        )
    }
}

fn pair_count(n: usize) -> usize {
    n.saturating_mul(n.saturating_sub(1)) / 2
}

fn pair_index(output_bits: usize, a: usize, b: usize) -> usize {
    debug_assert!(a < b && b < output_bits);
    a * (2 * output_bits - a - 1) / 2 + (b - a - 1)
}

/// Evaluate SAC/BIC style avalanche behavior for a Boolean transformation
/// induced by `f`.
///
/// `f` maps an `input_bits`-bit value to an `output_bits`-bit value. If the
/// full domain is small enough and `samples` covers it, the routine enumerates
/// every input exactly; otherwise it uses `samples` SplitMix64-scrambled inputs.
pub fn evaluate_u64<F>(
    input_bits: usize,
    output_bits: usize,
    samples: usize,
    mut f: F,
) -> AvalancheReport
where
    F: FnMut(u64) -> u64,
{
    assert!(
        (1..=64).contains(&input_bits),
        "input_bits must be in 1..=64"
    );
    assert!(
        (1..=64).contains(&output_bits),
        "output_bits must be in 1..=64"
    );
    assert!(samples > 0, "samples must be positive");

    let input_mask = if input_bits == 64 {
        u64::MAX
    } else {
        (1u64 << input_bits) - 1
    };
    let output_mask = if output_bits == 64 {
        u64::MAX
    } else {
        (1u64 << output_bits) - 1
    };

    let full_domain = if input_bits == 64 {
        None
    } else {
        Some(1usize << input_bits)
    };
    let exact = full_domain.is_some_and(|domain| samples >= domain);
    let base_inputs: Vec<u64> = if exact {
        (0..full_domain.unwrap()).map(|x| x as u64).collect()
    } else {
        let mut state = 0x243f_6a88_85a3_08d3u64;
        (0..samples)
            .map(|_| splitmix64(&mut state) & input_mask)
            .collect()
    };

    let used_samples = base_inputs.len();
    let mut ones = vec![vec![0usize; input_bits]; output_bits];
    let mut bic = vec![vec![CorrAccum::default(); pair_count(output_bits)]; input_bits];

    for &x in &base_inputs {
        let y = f(x) & output_mask;
        for j in 0..input_bits {
            let xj = x ^ (1u64 << j);
            let avalanche = (y ^ (f(xj & input_mask) & output_mask)) & output_mask;
            for (i, col) in ones.iter_mut().enumerate().take(output_bits) {
                if ((avalanche >> i) & 1) != 0 {
                    col[j] += 1;
                }
            }
            for a in 0..output_bits {
                let bit_a = ((avalanche >> a) & 1) != 0;
                for b in (a + 1)..output_bits {
                    let bit_b = ((avalanche >> b) & 1) != 0;
                    let idx = pair_index(output_bits, a, b);
                    bic[j][idx].observe(bit_a, bit_b);
                }
            }
        }
    }

    let total_cells = (input_bits * output_bits) as f64;
    let mut dependence = vec![vec![0.0; input_bits]; output_bits];
    let mut max_sac_bias = 0.0f64;
    let mut sum_abs_bias = 0.0;
    let mut sum_sq_bias = 0.0;
    for i in 0..output_bits {
        for j in 0..input_bits {
            let p = ones[i][j] as f64 / used_samples as f64;
            let bias = (p - 0.5).abs();
            dependence[i][j] = p;
            max_sac_bias = max_sac_bias.max(bias);
            sum_abs_bias += bias;
            sum_sq_bias += bias * bias;
        }
    }

    let mut max_bic_abs_corr = 0.0f64;
    let mut sum_abs_corr = 0.0;
    let mut corr_terms = 0usize;
    let mut bic_degenerate_pairs = 0usize;
    for accs in &bic {
        for &acc in accs {
            match acc.correlation() {
                Some(rho) => {
                    let rho = rho.abs();
                    max_bic_abs_corr = max_bic_abs_corr.max(rho);
                    sum_abs_corr += rho;
                    corr_terms += 1;
                }
                None => bic_degenerate_pairs += 1,
            }
        }
    }
    let defined = corr_terms > 0;

    AvalancheReport {
        input_bits,
        output_bits,
        samples: used_samples,
        exact,
        dependence,
        max_sac_bias,
        mean_sac_bias: sum_abs_bias / total_cells,
        rms_sac_bias: (sum_sq_bias / total_cells).sqrt(),
        max_bic_abs_corr: if defined { max_bic_abs_corr } else { f64::NAN },
        mean_bic_abs_corr: if defined {
            sum_abs_corr / corr_terms as f64
        } else {
            f64::NAN
        },
        bic_pairs: corr_terms + bic_degenerate_pairs,
        bic_degenerate_pairs,
    }
}

#[cfg(test)]
mod tests {
    use super::evaluate_u64;
    use crate::rng::{Rng, Xorshift32};

    #[test]
    fn identity_mapping_is_perfectly_non_avalanche() {
        let report = evaluate_u64(4, 4, 16, |x| x);
        assert!(report.exact);
        assert_eq!(16, report.samples);
        for i in 0..4 {
            for j in 0..4 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((report.dependence[i][j] - expected).abs() < 1e-12);
            }
        }
        assert!((report.max_sac_bias - 0.5).abs() < 1e-12);
        // Each avalanche vector is the constant e_j: all 4 · C(4,2) pairs are 0/0.
        assert_eq!(24, report.bic_pairs);
        assert_eq!(24, report.bic_degenerate_pairs);
        assert!(report.max_bic_abs_corr.is_nan());
        assert!(report.mean_bic_abs_corr.is_nan());
    }

    /// Regression: a zero-variance avalanche variable used to score ρ = 0,
    /// so GF(2)-linear Xorshift32 printed BICmax = 0.0000, the ideal value.
    #[test]
    fn linear_map_reports_degenerate_pairs_not_ideal_bic() {
        // f(x) ⊕ f(x ⊕ e_j) = f(e_j) for every sampled x: every pair is 0/0.
        let report = evaluate_u64(32, 32, 256, |seed| {
            Xorshift32::new((seed as u32).max(1)).next_u64()
        });
        assert_eq!(32 * (32 * 31 / 2), report.bic_pairs);
        assert_eq!(report.bic_pairs, report.bic_degenerate_pairs);
        assert!(
            report.max_bic_abs_corr.is_nan(),
            "BIC must not read as ideal"
        );
        assert!(report.mean_bic_abs_corr.is_nan());
    }

    #[test]
    fn parity_broadcast_is_maximally_correlated() {
        let report = evaluate_u64(4, 4, 16, |x| {
            let g = (x & 1) & ((x >> 1) & 1);
            g | (g << 1)
        });
        for i in 0..2 {
            for j in 0..4 {
                let expected = if j <= 1 { 0.5 } else { 0.0 };
                assert!((report.dependence[i][j] - expected).abs() < 1e-12);
            }
        }
        assert!((report.max_bic_abs_corr - 1.0).abs() < 1e-12);
        // Flipping x0 (or x1) flips output bits 0 and 1 together whenever the
        // other input is 1, so ρ(0,1) = 1; the other 5 pairs for those inputs
        // involve a never-flipping bit, and inputs 2 and 3 flip nothing.
        assert_eq!(24, report.bic_pairs);
        assert_eq!(5 + 5 + 6 + 6, report.bic_degenerate_pairs);
        assert!((report.mean_bic_abs_corr - 1.0).abs() < 1e-12);
    }
}
