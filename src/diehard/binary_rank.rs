//! DIEHARD Tests 3, 4, 5 — Binary Rank Tests (31×31, 32×32, 6×8).
//!
//! The NIST SP 800-22 binary rank test uses only 32×32 matrices with specific
//! theoretical probabilities.  DIEHARD additionally tests 31×31 and 6×8
//! matrices.  These are provided as separate tests here.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::igamc, result::TestResult};

// ── 32×32 ─────────────────────────────────────────────────────────────────────

// Theoretical probabilities: P(rank=32)≈0.2888, P(rank=31)≈0.5776, P(≤30)≈0.1336
// (same as NIST §2.5 but DIEHARD uses 40 000 matrices).

/// 32×32 binary matrix rank test (DIEHARD variant; 40 000 matrices).
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn binary_rank_32x32(words: &[u32]) -> TestResult {
    let bits_per = 32 * 32;
    let n_matrices = 40_000;
    if words.len() * 32 < bits_per * n_matrices {
        return TestResult::insufficient("diehard::binary_rank_32x32", "not enough words");
    }
    rank_test(words, 32, 32, n_matrices, "diehard::binary_rank_32x32")
}

// ── 31×31 ─────────────────────────────────────────────────────────────────────

// P(rank=31)≈0.2888, P(rank=30)≈0.5776, P(rank=29)≈0.1284, P(rank≤28)≈0.0053.
// Same probabilities as 32×32: difference is 2^{-32} ≈ 2.3×10^{-10}, negligible.

/// 31×31 binary matrix rank test (DIEHARD variant; 40 000 matrices).
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn binary_rank_31x31(words: &[u32]) -> TestResult {
    let n_matrices = 40_000;
    rank_test(words, 31, 31, n_matrices, "diehard::binary_rank_31x31")
}

// ── 6×8 ───────────────────────────────────────────────────────────────────────

/// 6×8 binary matrix rank test (one byte per row, 6 rows; 100 000 matrices).
///
/// Each row is one byte (8 bits) drawn from a specified byte position within
/// successive 32-bit words.  This tests byte-level linear dependence.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn binary_rank_6x8(words: &[u32]) -> TestResult {
    let rows = 6usize;
    let cols = 8usize;
    let n_matrices = 100_000;

    if words.len() < rows * n_matrices {
        return TestResult::insufficient("diehard::binary_rank_6x8", "not enough words");
    }

    // Theoretical probabilities for a 6×8 binary matrix over GF(2).
    // P(rank=6) = (255×254×252×248×240×224) / 2^48 = 217613271859200 / 281474976710656 ≈ 0.7731
    // P(rank=5) = 63 × (255×254×252×248×240) / 2^48 = 61203732710400 / 281474976710656 ≈ 0.2174
    // P(rank≤4) = 1 − P(rank=6) − P(rank=5) ≈ 0.0094
    let p_full: f64  = 217_613_271_859_200.0 / 281_474_976_710_656.0;  // rank = 6
    let p_five: f64  = 61_203_732_710_400.0  / 281_474_976_710_656.0;  // rank = 5
    let p_less: f64  = 1.0 - p_full - p_five;                           // rank ≤ 4

    let mut f = [0usize; 3]; // f[0]=rank≤4, f[1]=rank=5, f[2]=rank=6
    let mut word_iter = words.iter().copied();

    for _ in 0..n_matrices {
        // Build 6-row matrix: byte 0 of each of 6 consecutive words.
        let mut matrix = [0u8; 6];
        for r in 0..rows {
            matrix[r] = (word_iter.next().unwrap_or(0) & 0xFF) as u8;
        }
        let rank = gf2_rank_6x8(&matrix, rows, cols);
        match rank {
            6 => f[2] += 1,
            5 => f[1] += 1,
            _ => f[0] += 1,
        }
    }

    let m = n_matrices as f64;
    let chi_sq =
        (f[0] as f64 - m * p_less).powi(2) / (m * p_less)
        + (f[1] as f64 - m * p_five).powi(2) / (m * p_five)
        + (f[2] as f64 - m * p_full).powi(2) / (m * p_full);

    let p_value = igamc(1.0, chi_sq / 2.0); // df = 2

    TestResult::with_note(
        "diehard::binary_rank_6x8",
        p_value,
        format!("N={n_matrices}, χ²={chi_sq:.4}"),
    )
}

// ── shared helpers ─────────────────────────────────────────────────────────────

/// General binary rank test for R×C matrices (C ≤ 32).
///
/// Uses 4 bins matching `diehard_rank_32x32.c`: rank=full, full-1, full-2, ≤full-3.
/// Bins with expected count < 5.0 are excluded from the chi-square (Vtest cutoff).
fn rank_test(words: &[u32], rows: usize, cols: usize, n_matrices: usize, name: &'static str) -> TestResult {
    if words.len() < rows * n_matrices {
        return TestResult::insufficient(name, "not enough words");
    }

    let (p3, p2, p1, p0) = theoretical_probs(rows, cols);
    // p3 = P(rank=full), p2 = P(rank=full-1), p1 = P(rank=full-2), p0 = P(rank≤full-3)

    let mut f = [0usize; 4]; // f[3]=rank=full, f[2]=full-1, f[1]=full-2, f[0]=≤full-3

    // Mask is the same for every matrix; pre-allocate the row buffer once.
    let mask = if cols < 32 { (1u32 << cols) - 1 } else { u32::MAX };
    let mut matrix = Vec::with_capacity(rows);

    for m_idx in 0..n_matrices {
        let slice = &words[m_idx * rows..(m_idx + 1) * rows];
        matrix.clear();
        matrix.extend(slice.iter().map(|&w| w & mask));
        let rank = gf2_rank_generic(&matrix, rows, cols);
        let full = rows.min(cols);
        if rank == full         { f[3] += 1; }
        else if rank == full-1  { f[2] += 1; }
        else if rank == full-2  { f[1] += 1; }
        else                    { f[0] += 1; }
    }

    let m = n_matrices as f64;
    let probs = [p0, p1, p2, p3];
    let chi_sq: f64 = f.iter().zip(probs.iter())
        .filter(|(_, &p)| p * m >= 5.0)
        .map(|(&cnt, &p)| (cnt as f64 - m * p).powi(2) / (m * p))
        .sum();
    let df = f.iter().zip(probs.iter())
        .filter(|(_, &p)| p * m >= 5.0)
        .count()
        .saturating_sub(1);

    let p_value = igamc(df as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(name, p_value, format!("{rows}×{cols}, N={n_matrices}, χ²={chi_sq:.4}"))
}

/// Theoretical rank-distribution probabilities for an R×C matrix over GF(2).
///
/// Returns (P(rank=full), P(rank=full-1), P(rank=full-2), P(rank≤full-3)).
///
/// For 32×32: values from `diehard_rank_32x32.c` (David Bauer, "On the Rank
/// of Random Matrices"), pooling ranks ≤ 29 into the tail bin.
/// Source: `dieharder-3.31.1/libdieharder/diehard_rank_32x32.c`.
///
/// For other sizes we compute the exact GF(2) rank probabilities directly from
/// the matrix-count formula instead of reusing the 32×32 constants.
fn theoretical_probs(rows: usize, cols: usize) -> (f64, f64, f64, f64) {
    match (rows, cols) {
        // Probabilities from diehard_rank_32x32.c, bins [rank≤29, 30, 31, 32].
        (32, 32) => (0.2887880952, 0.5775761902, 0.1283502644, 0.0052854502),
        (31, 31) => {
            let full = gf2_rank_probability(31, 31, 31);
            let full_minus_1 = gf2_rank_probability(31, 31, 30);
            let full_minus_2 = gf2_rank_probability(31, 31, 29);
            let tail = (1.0 - full - full_minus_1 - full_minus_2).max(0.0);
            (full, full_minus_1, full_minus_2, tail)
        }
        _ => {
            let full = rows.min(cols);
            let p_full = gf2_rank_probability(rows, cols, full);
            let p_full_minus_1 = if full >= 1 {
                gf2_rank_probability(rows, cols, full - 1)
            } else {
                0.0
            };
            let p_full_minus_2 = if full >= 2 {
                gf2_rank_probability(rows, cols, full - 2)
            } else {
                0.0
            };
            let p_tail = (1.0 - p_full - p_full_minus_1 - p_full_minus_2).max(0.0);
            (p_full, p_full_minus_1, p_full_minus_2, p_tail)
        }
    }
}

fn gf2_rank_probability(rows: usize, cols: usize, rank: usize) -> f64 {
    if rank > rows.min(cols) {
        return 0.0;
    }
    if rank == 0 {
        return 2f64.powi(-((rows * cols) as i32));
    }
    let mut log_prob = -((rows * cols) as f64) * std::f64::consts::LN_2;
    for i in 0..rank {
        let ip = i as i32;
        let rows_term = 2f64.powi(rows as i32) - 2f64.powi(ip);
        let cols_term = 2f64.powi(cols as i32) - 2f64.powi(ip);
        let rank_term = 2f64.powi(rank as i32) - 2f64.powi(ip);
        log_prob += rows_term.ln() + cols_term.ln() - rank_term.ln();
    }
    log_prob.exp()
}

/// GF(2) rank of a matrix stored as rows of u32 (up to 32 columns).
fn gf2_rank_generic(matrix: &[u32], rows: usize, cols: usize) -> usize {
    let mut m = matrix.to_vec();
    let mut rank = 0usize;
    let mut pivot_row = 0usize;

    for col in 0..cols {
        let found = (pivot_row..rows).find(|&r| (m[r] >> col) & 1 == 1);
        if let Some(r) = found {
            m.swap(pivot_row, r);
            rank += 1;
            let pivot = m[pivot_row];
            for r in 0..rows {
                if r != pivot_row && (m[r] >> col) & 1 == 1 {
                    m[r] ^= pivot;
                }
            }
            pivot_row += 1;
        }
    }
    rank
}

/// GF(2) rank of a 6×8 matrix stored as 6 bytes.
fn gf2_rank_6x8(matrix: &[u8; 6], rows: usize, cols: usize) -> usize {
    let as_u32: Vec<u32> = matrix.iter().map(|&b| b as u32).collect();
    gf2_rank_generic(&as_u32, rows, cols)
}

#[cfg(test)]
mod tests {
    use super::{gf2_rank_probability, theoretical_probs};

    #[test]
    fn exact_31x31_probabilities_sum_to_one() {
        let (p_full, p_m1, p_m2, p_tail) = theoretical_probs(31, 31);
        assert!(((p_full + p_m1 + p_m2 + p_tail) - 1.0).abs() < 1e-12);
        assert!(p_full > 0.28 && p_full < 0.29);
        assert!(p_tail > 0.0);
    }

    #[test]
    fn generic_rank_probability_matches_32x32_reference_close() {
        let p = gf2_rank_probability(32, 32, 32);
        assert!((p - 0.2887880952).abs() < 1e-9);
    }
}
