//! NIST SP 800-22 §2.5 — Binary Matrix Rank Test.
//!
//! Constructs 32×32 binary matrices from the bit sequence and tests whether
//! the distribution of GF(2) ranks matches the theoretical distribution.
//!
//! Minimum recommended sequence length: n ≥ 38 912 (for at least 38 matrices).

use crate::{math::{gf2_rank, igamc}, result::TestResult};

const ROWS: usize = 32;
const COLS: usize = 32;

// Theoretical probabilities for rank R of a 32×32 binary matrix (SP 800-22 §2.5.4):
//   P(rank = 32) ≈ 0.2888
//   P(rank = 31) ≈ 0.5776
//   P(rank ≤ 30) ≈ 0.1336
const P32: f64 = 0.2888;
const P31: f64 = 0.5776;
const P_LESS: f64 = 0.1336;

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

    let m = num_matrices as f64;
    let chi_sq = (f_32 as f64 - m * P32).powi(2) / (m * P32)
        + (f_31 as f64 - m * P31).powi(2) / (m * P31)
        + (f_less as f64 - m * P_LESS).powi(2) / (m * P_LESS);

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
