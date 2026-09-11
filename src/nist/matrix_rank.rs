//! NIST SP 800-22 §2.5 — Binary Matrix Rank Test.
//!
//! Constructs 32×32 binary matrices from the bit sequence and tests whether
//! the distribution of GF(2) ranks matches the theoretical distribution.
//!
//! Minimum recommended sequence length: n ≥ 38 912 (for at least 38 matrices).
//!
//! # References
//! * A. Rukhin et al., *NIST SP 800-22 Rev. 1a*, 2010, §2.5 and §3.5.
//!   [pubs/NIST-SP-800-22r1a.pdf]
//! * I. N. Kovalenko, "Distribution of the linear rank of a random matrix,"
//!   *Theory of Probability and its Applications* 17, pp. 342–346, 1972.
//!   [Rank distribution of a random binary matrix, as cited in §3.5]
//! * G. Marsaglia and L. H. Tsay, "Matrices and the structure of random
//!   number sequences," *Linear Algebra and its Applications* 67,
//!   pp. 147–156, 1985.  [Same result, as cited in §3.5]

use crate::{
    math::{gf2_rank, igamc},
    result::TestResult,
};

const ROWS: usize = 32;
const COLS: usize = 32;

/// Probability that a `rows`×`cols` matrix of independent uniform bits has
/// GF(2) rank `r` (SP 800-22 §3.5):
///
/// p_r = 2^{r(Q+M−r)−MQ} ∏_{i=0}^{r−1} (1 − 2^{i−Q})(1 − 2^{i−M}) / (1 − 2^{i−r})
///
/// with M = `rows` and Q = `cols`.  §2.5.4 prints the 32×32 values rounded to
/// 0.2888, 0.5776 and 0.1336; the §2.5.8 example uses the unrounded ones.
fn rank_probability(r: usize, rows: usize, cols: usize) -> f64 {
    let (r, m, q) = (r as i32, rows as i32, cols as i32);
    let product: f64 = (0..r)
        .map(|i| (1.0 - 2f64.powi(i - q)) * (1.0 - 2f64.powi(i - m)) / (1.0 - 2f64.powi(i - r)))
        .product();
    2f64.powi(r * (q + m - r) - m * q) * product
}

/// χ² of §2.5.4 step (4) for the counts of full-rank, rank-31 and lower-rank
/// matrices, against p₃₂, p₃₁ and 1 − p₃₂ − p₃₁.
fn rank_chi_square(f_32: usize, f_31: usize, f_less: usize) -> f64 {
    let n = (f_32 + f_31 + f_less) as f64;
    let p_32 = rank_probability(ROWS, ROWS, COLS);
    let p_31 = rank_probability(ROWS - 1, ROWS, COLS);
    let p_less = 1.0 - p_32 - p_31;
    [(f_32, p_32), (f_31, p_31), (f_less, p_less)]
        .into_iter()
        .map(|(count, p)| (count as f64 - n * p).powi(2) / (n * p))
        .sum()
}

/// Run the binary matrix rank test.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.5.
pub fn matrix_rank(bits: &[u8]) -> TestResult {
    let bits_per_matrix = ROWS * COLS;
    let n = bits.len();
    let num_matrices = n / bits_per_matrix;

    if num_matrices < 38 {
        return TestResult::insufficient("nist::matrix_rank", "n too small (need ≥ 38 matrices)");
    }

    let (mut f_32, mut f_31, mut f_less) = (0usize, 0usize, 0usize);

    for chunk in bits.chunks_exact(bits_per_matrix).take(num_matrices) {
        let rank = gf2_rank_32x32(chunk);
        match rank {
            32 => f_32 += 1,
            31 => f_31 += 1,
            _ => f_less += 1,
        }
    }

    let chi_sq = rank_chi_square(f_32, f_31, f_less);

    let p_value = igamc(1.0, chi_sq / 2.0); // df = 2, so igamc(1, χ²/2)

    TestResult::with_note(
        "nist::matrix_rank",
        p_value,
        format!("N={num_matrices}, F32={f_32}, F31={f_31}, F≤30={f_less}, χ²={chi_sq:.4}"),
    )
}

/// Compute the GF(2) rank of a 32×32 binary matrix.
///
/// `bits` must have exactly 1024 elements (0 or 1), row-major.
/// Packs each row into a u32 word, then delegates to `math::gf2_rank`.
fn gf2_rank_32x32(bits: &[u8]) -> usize {
    let mut matrix = [0u32; ROWS];
    for (r, row) in bits.chunks_exact(COLS).enumerate() {
        let mut word = 0u32;
        for (c, &b) in row.iter().enumerate() {
            word |= (b as u32) << c;
        }
        matrix[r] = word;
    }
    gf2_rank(&matrix, ROWS, COLS)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// p₃₂ and p₃₁ from an independent exact rational evaluation of the §3.5
    /// formula (Python `fractions`), in shortest round-trip form.
    const EXACT_P32: f64 = 0.28878809515384113;
    const EXACT_P31: f64 = 0.5775761901732048;

    #[test]
    fn rank_probabilities_match_exact_evaluation() {
        let p_32 = rank_probability(32, ROWS, COLS);
        let p_31 = rank_probability(31, ROWS, COLS);
        assert!((p_32 - EXACT_P32).abs() < 1e-15, "p₃₂ = {p_32}");
        assert!((p_31 - EXACT_P31).abs() < 1e-15, "p₃₁ = {p_31}");
    }

    /// The rank distribution over r = 0..=32 sums to 1, so the lower-rank
    /// cell 1 − p₃₂ − p₃₁ is the mass of ranks 0..=30.
    #[test]
    fn rank_distribution_sums_to_one() {
        let p = |r| rank_probability(r, ROWS, COLS);
        let total: f64 = (0..=32).map(p).sum();
        assert!((total - 1.0).abs() < 1e-14, "Σ p_r = {total}");
        let lower: f64 = (0..=30).map(p).sum();
        assert!((lower - (1.0 - p(32) - p(31))).abs() < 1e-14);
    }

    /// SP 800-22 §2.5.8: the first 100 000 bits of e give N = 97 matrices,
    /// F_M = 23, F_{M−1} = 60 and 14 of lower rank, with χ² = 1.2619656 and
    /// P-value = 0.532069.  The rounded §2.5.4 probabilities give
    /// χ² = 1.262580 and P-value = 0.531905.
    #[test]
    fn chi_square_reproduces_section_2_5_8_example() {
        let chi_sq = rank_chi_square(23, 60, 14);
        assert!((chi_sq - 1.2619656).abs() < 5e-8, "χ² = {chi_sq}");
        let p = igamc(1.0, chi_sq / 2.0);
        assert!((p - 0.532069).abs() < 1e-6, "p = {p}");
    }
}
