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

// P(rank=31)≈0.5765, P(rank=30)≈0.3461, P(≤29)≈0.0774 (SP 800-22 appendix, Marsaglia).

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
fn rank_test(words: &[u32], rows: usize, cols: usize, n_matrices: usize, name: &'static str) -> TestResult {
    if words.len() < rows * n_matrices {
        return TestResult::insufficient(name, "not enough words");
    }

    let (p_full, p_one_less, p_rest) = theoretical_probs(rows, cols);

    let mut f_full = 0usize;
    let mut f_one  = 0usize;
    let mut f_rest = 0usize;

    for m_idx in 0..n_matrices {
        let slice = &words[m_idx * rows..(m_idx + 1) * rows];
        // Truncate each word to `cols` bits.
        let mask = if cols < 32 { (1u32 << cols) - 1 } else { u32::MAX };
        let matrix: Vec<u32> = slice.iter().map(|&w| w & mask).collect();
        let rank = gf2_rank_generic(&matrix, rows, cols);
        let full = rows.min(cols);
        if rank == full { f_full += 1; }
        else if rank == full - 1 { f_one += 1; }
        else { f_rest += 1; }
    }

    let m = n_matrices as f64;
    let chi_sq =
        (f_full as f64 - m * p_full    ).powi(2) / (m * p_full    )
        + (f_one  as f64 - m * p_one_less).powi(2) / (m * p_one_less)
        + (f_rest as f64 - m * p_rest  ).powi(2) / (m * p_rest  );

    let p_value = igamc(1.0, chi_sq / 2.0);

    TestResult::with_note(name, p_value, format!("{rows}×{cols}, N={n_matrices}, χ²={chi_sq:.4}"))
}

/// Theoretical rank-distribution probabilities for an R×C matrix over GF(2).
/// Returns (P(rank=min(R,C)), P(rank=min(R,C)−1), P(rest)).
fn theoretical_probs(rows: usize, cols: usize) -> (f64, f64, f64) {
    // Product formula for the probability that a random binary m×n matrix
    // has rank exactly k.  We use the closed-form for full rank and rank−1.
    //
    // P(rank = m) = Π_{i=0}^{m-1} (1 − 2^(i-n)) · Π_{i=0}^{m-1} (1 − 2^(i-m))
    // This is complex to compute for arbitrary m,n; use precomputed values
    // for the specific cases DIEHARD cares about.
    match (rows, cols) {
        (32, 32) => (0.2888, 0.5776, 0.1336),
        // P(rank=31) = ∏_{j=1}^{31}(1-2^{-j}) ≈ 0.2888 (same limit as 32×32
        // because 2^{-32} is negligible); higher ranks follow the same pattern.
        (31, 31) => (0.2888, 0.5776, 0.1336),
        _ => {
            // Generic approximation: most mass near full rank.
            (0.5, 0.3, 0.2)
        }
    }
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
