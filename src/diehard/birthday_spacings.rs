//! DIEHARD Test 1 — Birthday Spacings Test.
//!
//! Chooses m = 512 "birthdays" from a year of n = 2²⁴ days.  The number j of
//! repeated spacings in each trial is asymptotically Poisson(λ = m³/(4n) = 2).
//! After 500 trials per bit offset, a chi-square test of the j histogram against
//! the Poisson(2) distribution gives one p-value.  Nine p-values (bit offsets
//! 0..=8) are combined with a final KS test.
//!
//! Each trial follows Marsaglia's `cdbday` (`fortran/diehard.f` lines
//! 1235–1307).  The birthdays are sorted, the spacings C(1) = B(1),
//! C(i) = B(i) − B(i−1) are sorted, and j counts the i with C(i) = C(i−1)
//! (lines 1277–1283), so a spacing value that occurs three times adds 2.
//! `tests.txt` words j as the number of values that occur more than once,
//! which would add 1.  Dieharder's `diehard_birthdays.c` builds the same
//! spacings, but its counting loop (lines 187–203) sets `m = mnext` and then
//! increments `m` again, skipping the spacing after each run of equal values:
//! on sorted spacings [5, 5, 7, 7] it counts 1 where DIEHARD counts 2.
//!
//! The histogram is scored in the cells Marsaglia's `CHSQTS` builds (lines
//! 1312–1375): at 500 trials, j = 0 to 5 each alone and j ≥ 6 pooled,
//! expecting 67.668, 135.335, 135.335, 90.224, 45.112, 18.045 and 8.282
//! trials, df = 6.  Dieharder keeps `kmax` = 8 cells, j = 0 to 7, discards
//! trials with larger j, and its `chisq_poisson` (`chisq.c`) sums all eight
//! with df = 7, or six cells and df = 5 at its default of 100 trials.  On
//! 12 000 null calls with MT19937 the reported p-value fell below 0.01 in
//! 0.96% of calls and below 0.001 in 0.06%.
//!
//! Each of the nine bit windows reads its own 256 000 words, 2 304 000 in
//! all.  `cdbday` instead rewinds the file for every window (`jkreset`, line
//! 1267), so its nine p-values come largely from the same words and are
//! dependent, which a summary over them does not allow for.  Offset o reads
//! bits o to o + 23 of each word, DIEHARD's window `kr` = o (line 1242), which
//! it prints as bits 9 − o to 32 − o counting from the left.  DIEHARD reports
//! each window's chi-square as its CDF (`chisq(s,j)`, line 1375) and
//! summarizes the nine with Marsaglia's Anderson–Darling statistic, which
//! `tests.txt` calls a KS test (`KSTEST`, lines 1668–1709); this module
//! reports upper-tail p-values and a Kolmogorov–Smirnov summary.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).
//! Source: Marsaglia's `fortran/diehard.f`, subroutines `cdbday` and `CHSQTS`
//! [pubs/diehard-fortran-1996.tar.gz]; for comparison,
//! `dieharder-3.31.1/libdieharder/diehard_birthdays.c`.

use crate::{
    math::{igamc, ks_test},
    result::TestResult,
};

const M: usize = 512; // birthdays per trial
const YEAR: u32 = 1 << 24; // year size = 2^24 (nbits = 24)
const SAMPLES: usize = 500; // trials per bit window
const LAMBDA: f64 = 2.0; // m³/(4n) = 512³/(4×2^24) = 2
const WINDOWS: usize = 9; // bit offsets 0..=8

/// Run the birthday spacings test.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn birthday_spacings(words: &[u32]) -> TestResult {
    if words.len() < WINDOWS * SAMPLES * M {
        return TestResult::insufficient("diehard::birthday_spacings", "not enough words");
    }

    let (cell_of, expected) = chsqts_cells(LAMBDA, SAMPLES);
    let df = expected.len() - 1;
    let mut p_values: Vec<f64> = Vec::with_capacity(WINDOWS);
    let mut word_iter = words.iter().copied();
    let mut birthdays = vec![0u32; M];
    let mut spacings = vec![0u32; M];

    for offset in 0..WINDOWS {
        let mut observed = vec![0u32; expected.len()];

        for _ in 0..SAMPLES {
            for birthday in birthdays.iter_mut() {
                // The length gate guarantees the iterator has enough words.
                let word = word_iter
                    .next()
                    .expect("birthday_spacings: word iterator exhausted (precondition failed)");
                *birthday = (word >> offset) & (YEAR - 1);
            }
            birthdays.sort_unstable();

            // C(1) = B(1), C(i) = B(i) − B(i−1), as in cdbday and
            // diehard_birthdays.c.
            spacings[0] = birthdays[0];
            for (spacing, pair) in spacings[1..].iter_mut().zip(birthdays.windows(2)) {
                *spacing = pair[1] - pair[0];
            }
            spacings.sort_unstable();

            let j = repeated_spacings(&spacings);
            observed[cell_of[j.min(cell_of.len() - 1)]] += 1;
        }

        let chi_sq: f64 = observed
            .iter()
            .zip(&expected)
            .map(|(&obs, &exp)| (f64::from(obs) - exp).powi(2) / exp)
            .sum();
        p_values.push(igamc(df as f64 / 2.0, chi_sq / 2.0));
    }

    // Final KS test on the 9 chi-square p-values.
    let p_value = ks_test(&mut p_values);

    TestResult::with_note(
        "diehard::birthday_spacings",
        p_value,
        format!("m={M}, year=2^24, samples={SAMPLES}"),
    )
}

/// The number of i with C(i) = C(i−1) in sorted spacings, `L` in `cdbday`
/// (`fortran/diehard.f` lines 1277–1283): a value that occurs r times adds
/// r − 1.
fn repeated_spacings(sorted: &[u32]) -> usize {
    sorted.windows(2).filter(|pair| pair[0] == pair[1]).count()
}

/// The chi-square cells of Marsaglia's `CHSQTS` (`fortran/diehard.f` lines
/// 1312–1344) for a Poisson(`lambda`) count over `trials` trials.
///
/// Returns `cell_of`, the cell of each count up to the first pooled one
/// (larger counts share the last cell), and each cell's expected number of
/// trials.  Counts join the open cell from 0 upward, and the cell closes once
/// it expects at least 5 trials (cell 0 needs more than 5).  As soon as fewer
/// than 5 trials are expected above count i, count i and every larger count
/// join the open cell, which takes the whole remaining expectation.
fn chsqts_cells(lambda: f64, trials: usize) -> (Vec<usize>, Vec<f64>) {
    let n = trials as f64;
    let mut p = (-lambda).exp();
    let mut cumulative = p * n;
    let mut open = p * n;
    let mut cell_of = vec![0];
    let mut expected = Vec::new();
    if open > 5.0 {
        expected.push(open);
        open = 0.0;
    }
    let last = (lambda + 4.0 * lambda.sqrt()) as usize;
    for i in 1..=last {
        p = lambda * p / i as f64;
        cumulative += p * n;
        open += p * n;
        cell_of.push(expected.len());
        if cumulative > n - 5.0 {
            expected.push(open + n - cumulative);
            return (cell_of, expected);
        }
        if open >= 5.0 {
            expected.push(open);
            open = 0.0;
        }
    }
    // CHSQTS leaves this case undefined; it does not arise at λ = 2 and 500
    // trials.  Close the open cell with the remaining expectation.
    expected.push(open + n - cumulative);
    (cell_of, expected)
}

#[cfg(test)]
mod tests {
    use super::{birthday_spacings, chsqts_cells, repeated_spacings, LAMBDA, M, SAMPLES, WINDOWS};

    /// A value seen r times adds r − 1, as `cdbday` counts it.
    #[test]
    fn repeats_count_adjacent_equal_spacings() {
        assert_eq!(repeated_spacings(&[1, 2, 3]), 0);
        assert_eq!(repeated_spacings(&[5, 5, 7, 7]), 2);
        assert_eq!(repeated_spacings(&[5, 5, 5]), 2);
        assert_eq!(repeated_spacings(&[4, 4, 4, 4, 9]), 3);
    }

    /// At λ = 2 and 500 trials CHSQTS scores counts 0 to 5 alone and pools
    /// 6 and up: expectations 500·P(j) and 500·P(j ≥ 6), which the gfortran
    /// build of diehard.f prints to three decimals.
    #[test]
    fn cells_match_chsqts() {
        let (cell_of, expected) = chsqts_cells(LAMBDA, SAMPLES);
        assert_eq!(cell_of, [0, 1, 2, 3, 4, 5, 6]);
        let printed = [67.668, 135.335, 135.335, 90.224, 45.112, 18.045, 8.282];
        for (e, want) in expected.iter().zip(printed) {
            assert!((e - want).abs() < 5e-4, "{e} vs {want}");
        }
        assert!((expected.iter().sum::<f64>() - 500.0).abs() < 1e-9);
    }

    /// Words needed: 500 trials of 512 birthdays at each of 9 bit offsets.
    const NEEDED: usize = WINDOWS * SAMPLES * M;

    #[test]
    fn short_inputs_skip() {
        assert!(birthday_spacings(&[]).skipped());
        assert!(birthday_spacings(&vec![0; NEEDED - 1]).skipped());
    }

    /// Every birthday is day 0, so every spacing is 0 and every trial lands in
    /// the pooled cell.
    #[test]
    fn constant_input_fails() {
        let r = birthday_spacings(&vec![0; NEEDED]);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }
}
