//! DIEHARD Tests 3, 4, 5 — Binary Rank Tests (31×31, 32×32, 6×8).
//!
//! The NIST SP 800-22 binary rank test uses only 32×32 matrices with specific
//! theoretical probabilities.  DIEHARD additionally tests 31×31 and 6×8
//! matrices.  These are provided as separate tests here.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{
    math::{gf2_rank, igamc},
    result::TestResult,
};

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
/// Each row is the leftmost 31 bits of one 32-bit word (`w >> 1`), as
/// Marsaglia specifies: "The leftmost 31 bits of 31 random integers from the
/// test sequence are used to form a 31x31 binary matrix" (`tests.txt`).
/// Ranks ≤ 28 are pooled, giving the four cells 31, 30, 29 and ≤ 28.
///
/// Dieharder 3.31.1 has no 31×31 test to compare against:
/// `diehard_rank_32x32.c` quotes this description in its header but builds
/// only 32×32 matrices from whole words.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn binary_rank_31x31(words: &[u32]) -> TestResult {
    let n_matrices = 40_000;
    rank_test(words, 31, 31, n_matrices, "diehard::binary_rank_31x31")
}

// ── 6×8 ───────────────────────────────────────────────────────────────────────

// Theoretical probabilities for a 6×8 binary matrix over GF(2).
// P(rank=6) = (255×254×252×248×240×224) / 2^48 = 217613271859200 / 281474976710656 ≈ 0.7731
// P(rank=5) = 63 × (255×254×252×248×240) / 2^48 = 61203732710400 / 281474976710656 ≈ 0.2174
// P(rank≤4) = 1 − P(rank=6) − P(rank=5) ≈ 0.0094

/// P(rank = 6) for a random 6×8 GF(2) matrix (a dyadic fraction, exact in f64).
const P6X8_FULL: f64 = 217_613_271_859_200.0 / 281_474_976_710_656.0;
/// P(rank = 5) for a random 6×8 GF(2) matrix (a dyadic fraction, exact in f64).
const P6X8_FIVE: f64 = 61_203_732_710_400.0 / 281_474_976_710_656.0;

/// 6×8 binary matrix rank test (one byte per row, 6 rows; 100 000 matrices).
///
/// Each row is the low byte (bits 0–7, `w & 0xFF`) of one of six successive
/// 32-bit words; bytes 1–3 never enter a matrix.  This matches Dieharder's
/// `diehard_rank_6x8.c`, whose `binary_rank(mtx, 6, 8)` (`rank.c`) reads the
/// eight columns from bit 0 upward, even though its comment speaks of the
/// leftmost byte.  It does not match DIEHARD: Marsaglia forms rows from "a
/// specified byte" (`tests.txt`) and repeats the test for 25 overlapping
/// 8-bit windows, bits 1–8 through 25–32, combining the 25 p-values with a
/// KS test (`diehard.exe` in `pubs/Diehard.zip`: "TEST SUMMARY, 25 tests on
/// 100,000 random 6x8 matrices").
///
/// The chi-square uses three cells, rank ≤ 4, 5 and 6 (df 2).  Dieharder's
/// `diehard_rank_6x8.c` also scores rank 3 on its own (4 cells, df 3).
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

    let p_full = P6X8_FULL; // rank = 6
    let p_five = P6X8_FIVE; // rank = 5
    let p_less: f64 = 1.0 - p_full - p_five; // rank ≤ 4

    let mut f = [0usize; 3]; // f[0]=rank≤4, f[1]=rank=5, f[2]=rank=6
    let mut word_iter = words.iter().copied();

    for _ in 0..n_matrices {
        // Build 6-row matrix: byte 0 of each of 6 consecutive words.
        let mut matrix = [0u32; 6];
        for slot in matrix.iter_mut().take(rows) {
            *slot = word_iter.next().unwrap_or(0) & 0xFF;
        }
        let rank = gf2_rank(&matrix, rows, cols);
        match rank {
            6 => f[2] += 1,
            5 => f[1] += 1,
            _ => f[0] += 1,
        }
    }

    let m = n_matrices as f64;
    let chi_sq = (f[0] as f64 - m * p_less).powi(2) / (m * p_less)
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
/// Each row is the leftmost `cols` bits of one word (see [`leftmost_bits`]).
/// Uses 4 bins matching `diehard_rank_32x32.c`: rank=full, full-1, full-2, ≤full-3.
/// Bins with expected count < 5.0 are excluded from the chi-square (Vtest cutoff).
fn rank_test(
    words: &[u32],
    rows: usize,
    cols: usize,
    n_matrices: usize,
    name: &'static str,
) -> TestResult {
    if words.len() < rows * n_matrices {
        return TestResult::insufficient(name, "not enough words");
    }

    let (p3, p2, p1, p0) = theoretical_probs(rows, cols);
    // p3 = P(rank=full), p2 = P(rank=full-1), p1 = P(rank=full-2), p0 = P(rank≤full-3)

    let mut f = [0usize; 4]; // f[3]=rank=full, f[2]=full-1, f[1]=full-2, f[0]=≤full-3

    // Pre-allocate the row buffer once.
    let mut matrix = Vec::with_capacity(rows);

    for m_idx in 0..n_matrices {
        let slice = &words[m_idx * rows..(m_idx + 1) * rows];
        matrix.clear();
        matrix.extend(slice.iter().map(|&w| leftmost_bits(w, cols)));
        let rank = gf2_rank(&matrix, rows, cols);
        let full = rows.min(cols);
        if rank == full {
            f[3] += 1;
        } else if rank == full - 1 {
            f[2] += 1;
        } else if rank == full - 2 {
            f[1] += 1;
        } else {
            f[0] += 1;
        }
    }

    let m = n_matrices as f64;
    let probs = [p0, p1, p2, p3];
    let chi_sq: f64 = f
        .iter()
        .zip(probs.iter())
        .filter(|(_, &p)| p * m >= 5.0)
        .map(|(&cnt, &p)| (cnt as f64 - m * p).powi(2) / (m * p))
        .sum();
    let df = f
        .iter()
        .zip(probs.iter())
        .filter(|(_, &p)| p * m >= 5.0)
        .count()
        .saturating_sub(1);

    let p_value = igamc(df as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(
        name,
        p_value,
        format!("{rows}×{cols}, N={n_matrices}, χ²={chi_sq:.4}"),
    )
}

/// The leftmost `cols` bits of `w` (1 ≤ `cols` ≤ 32), right-aligned so that
/// [`gf2_rank`] sees them as its low `cols` columns.  DIEHARD forms rank-test
/// rows "from leftmost" bits "of each 32-bit integer" (`diehard.exe`); for
/// `cols` = 32 this is the whole word.
fn leftmost_bits(w: u32, cols: usize) -> u32 {
    debug_assert!((1..=32).contains(&cols), "cols = {cols} must be 1..=32");
    w >> (32 - cols)
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
        // Probabilities from diehard_rank_32x32.c, in tuple order
        // (rank=32, 31, 30, ≤29) — matching the docstring above.
        (32, 32) => (0.2887880952, 0.5775761902, 0.1283502644, 0.0052854502),
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

#[cfg(test)]
mod tests {
    use super::{
        binary_rank_31x31, binary_rank_32x32, binary_rank_6x8, gf2_rank_probability, leftmost_bits,
        theoretical_probs, P6X8_FIVE, P6X8_FULL,
    };
    use crate::{
        math::gf2_rank,
        result::TestResult,
        rng::{Mt19937, Rng},
    };

    /// Exact P(rank = 31), P(30), P(29) and P(≤ 28) for a random 31×31 GF(2)
    /// matrix, from the product formula in exact rational arithmetic (Python
    /// `fractions`), rounded once to f64.
    #[test]
    fn theoretical_31x31_matches_exact_rank_probabilities() {
        let got = theoretical_probs(31, 31);
        let want = (
            0.28878809522107984,
            0.5775761901732048,
            0.1283502643633989,
            0.00528545024231639,
        );
        for (g, w) in [
            (got.0, want.0),
            (got.1, want.1),
            (got.2, want.2),
            (got.3, want.3),
        ] {
            assert!((g - w).abs() < 1e-12, "{g} vs {w}");
        }
    }

    /// High bytes set on every word; only the low byte may enter a row.
    const HIGH_NOISE: u32 = 0xDEAD_BE00;

    /// A stream of 100 000 6×8 matrices with known ranks: 944 of rank 4,
    /// 21 744 of rank 5 and 77 312 of rank 6 (ranks checked by an independent
    /// Python elimination).  χ² and p come from a Python replica of the
    /// three-cell statistic with the exact GF(2) probabilities.
    #[test]
    fn rank_6x8_statistic_matches_replica_on_constructed_ranks() {
        const RANK6: [u32; 6] = [1, 2, 4, 8, 16, 32];
        const RANK5: [u32; 6] = [1, 2, 4, 8, 16, 3];
        const RANK4: [u32; 6] = [3, 5, 6, 24, 40, 48];
        let words: Vec<u32> = [(RANK4, 944), (RANK5, 21_744), (RANK6, 77_312)]
            .into_iter()
            .flat_map(|(rows, n)| std::iter::repeat_n(rows, n))
            .flatten()
            .map(|b| b | HIGH_NOISE)
            .collect();
        let result = binary_rank_6x8(&words);
        let note = result.note.as_deref().unwrap_or_default();
        assert!(note.contains("χ²=0.0001"), "{note}");
        assert!(
            (result.p_value - 0.9999514430863818).abs() < 1e-9,
            "{result}"
        );
    }

    /// Words needed by the 31×31 test: 40 000 matrices of 31 rows.
    const WORDS_31X31: usize = 31 * 40_000;

    /// Row i = 2^(i+1).  Its leftmost 31 bits are 2^i, the identity (rank 31);
    /// its low 31 bits lose row 30 entirely (rank 30).  Ranks checked with an
    /// independent Python GF(2) elimination.
    #[test]
    fn rank_31x31_rows_are_the_leftmost_31_bits() {
        let words: Vec<u32> = (0..31).map(|i| 1u32 << (i + 1)).collect();
        let high: Vec<u32> = words.iter().map(|&w| leftmost_bits(w, 31)).collect();
        assert_eq!(high, (0..31).map(|i| 1u32 << i).collect::<Vec<_>>());
        assert_eq!(gf2_rank(&high, 31, 31), 31);
        let low: Vec<u32> = words.iter().map(|&w| w & (u32::MAX >> 1)).collect();
        assert_eq!(gf2_rank(&low, 31, 31), 30);
        assert_eq!(leftmost_bits(u32::MAX, 32), u32::MAX);
    }

    /// The statistic must not see bit 0.  With the low-31-bit mask, clearing
    /// bit 0 zeroed a whole column (every rank ≤ 30, p ≈ 0) while setting it
    /// did not, so the two p-values differed.
    #[test]
    fn rank_31x31_ignores_the_lowest_bit() {
        let words = Mt19937::new(5489).collect_u32s(WORDS_31X31);
        let cleared: Vec<u32> = words.iter().map(|&w| w & !1).collect();
        let set: Vec<u32> = words.iter().map(|&w| w | 1).collect();
        let (p_cleared, p_set) = (
            binary_rank_31x31(&cleared).p_value,
            binary_rank_31x31(&set).p_value,
        );
        assert_eq!(
            p_cleared.to_bits(),
            p_set.to_bits(),
            "{p_cleared} vs {p_set}"
        );
    }

    /// A stuck top bit zeroes a column of every matrix, so no matrix reaches
    /// rank 31 and the test must fail.
    #[test]
    fn rank_31x31_fails_a_stuck_top_bit() {
        let words: Vec<u32> = Mt19937::new(5489)
            .collect_u32s(WORDS_31X31)
            .into_iter()
            .map(|w| w & (u32::MAX >> 1))
            .collect();
        let result = binary_rank_31x31(&words);
        assert!(!result.skipped() && result.p_value < 1e-10, "{result}");
    }

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

    /// The 32×32 constants carry ten decimals, so their four entries sum to 1
    /// only within 4 × 5 × 10⁻¹¹.  The 6×8 masses P(6) and P(5) are exact and,
    /// with the exact P(rank ≤ 4) = 0.009443013983400306 (Python rationals),
    /// sum to 1; the generic formula reproduces both and sums to 1 over ranks
    /// 0..=6.
    #[test]
    fn rank_probability_tables_sum_to_one() {
        let (a, b, c, d) = theoretical_probs(32, 32);
        let sum = a + b + c + d;
        assert!((sum - 1.0).abs() <= 4.0 * 5e-11, "32×32 sum = {sum}");

        assert!((P6X8_FULL + P6X8_FIVE + 0.009443013983400306 - 1.0).abs() < 1e-15);
        assert!((gf2_rank_probability(6, 8, 6) - P6X8_FULL).abs() < 1e-13);
        assert!((gf2_rank_probability(6, 8, 5) - P6X8_FIVE).abs() < 1e-13);
        let total: f64 = (0..=6).map(|r| gf2_rank_probability(6, 8, r)).sum();
        assert!((total - 1.0).abs() < 1e-13, "6×8 total = {total}");
    }

    const RANK_TESTS: [fn(&[u32]) -> TestResult; 3] =
        [binary_rank_32x32, binary_rank_31x31, binary_rank_6x8];

    #[test]
    fn short_inputs_skip() {
        for test in RANK_TESTS {
            assert!(test(&[]).skipped());
            assert!(test(&[0; 100]).skipped());
        }
    }

    /// One word short of a full set of matrices must skip: 32x32 and 31x31
    /// need 40 000 matrices, 6x8 needs 100 000.
    #[test]
    fn one_word_short_of_the_matrices_skips() {
        for (test, needed) in RANK_TESTS.iter().zip([1_280_000usize, 1_240_000, 600_000]) {
            assert!(test(&vec![0; needed - 1]).skipped(), "needed {needed}");
        }
    }

    /// An all-zero matrix has rank 0, so every matrix lands in the lowest cell.
    #[test]
    fn constant_input_fails() {
        let words = vec![0u32; 32 * 40_000];
        for test in RANK_TESTS {
            let r = test(&words);
            assert!(!r.skipped() && !r.passed(), "{r}");
        }
    }
}
