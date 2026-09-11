//! Count-the-1s on specific bytes, over DIEHARD's 25 bit windows, each window
//! on its own words.  Result name:
//! `diehard_historical::count_ones_bytes_25_fresh`, 25 results, one per window.
//!
//! # What it is
//!
//! From each word one byte is taken, bits jk to jk + 7 counting from the
//! left, and turned into a letter by its number of 1s: 0, 1 or 2 is A, 3 is B,
//! 4 is C, 5 is D, and 6, 7 or 8 is E, with probabilities 37, 56, 70, 56 and 37
//! in 256.  Over 256 000 overlapping five-letter words the naive Pearson sums
//! Q5, over the 3 125 five-letter cells, and Q4, over the 625 cells of each
//! word's leading four letters, give Q5 − Q4, which Marsaglia takes to have
//! mean 2 500 and standard deviation √5000 under the null.  Its z-score gives
//! one p-value for each window, jk = 1 to 25.
//!
//! # Reference followed
//!
//! Marsaglia's `wknt1s`, `fortran/diehard.f` lines 837–919
//! [pubs/diehard-fortran-1996.tar.gz]:
//!
//! - one letter per word, `m8()=k(and(rshift(jtbl(),25-jk),255))`
//!   (line 854), that is the byte `w >> (25 − jk)`; its table `k`
//!   (lines 843–851) gives every byte the letter
//!   [`crate::diehard::count_ones`] gives it;
//! - windows `do 888 jk=1,25` (line 865), each counting 25 600 × n
//!   overlapping words with n = 10 (lines 855 and 873–874), and the
//!   four-letter word left when each word's first letter is erased (line 876);
//! - Q4 and Q5 against products of the letter probabilities (lines
//!   884–900) and z = (Q5 − Q4 − 2500)/√5000 (line 902): 25
//!   results and no summary.
//!
//! The Q5 − Q4 arithmetic is the default battery's
//! `diehard::count_ones_stream`, which the fidelity review found equal to
//! `diehard.f`'s on aligned input.  Dieharder's `diehard_count_1s_byte` is a
//! different test, on non-overlapping words with the byte offset cycling from
//! sample to sample.
//!
//! # Departures from `diehard.f`
//!
//! - **Fresh words for each window.**  Before each window `wknt1s` calls
//!   `jkreset` (line 866), which resets `jtbl`'s record counter but not
//!   its place in the current 4 096-word record (lines 414–428).  Each window
//!   after the first therefore reads out the rest of that record, the 1 933 to
//!   4 086 words that follow the previous window, and then rereads the file
//!   from word 1: run alone, window 2 starts at word 256 006 and window 25 at
//!   word 254 073.  Its 25 windows share nearly all their words, so their
//!   results are not independent, although for this statistic the dependence
//!   is weak: with every window on the same 256 004 words, as the aligned
//!   build reads them, adjacent windows' Q5 − Q4 correlated at 0.024 over
//!   4 000 simulated streams (0.006 two windows apart, and within ±0.004
//!   further out).  Here window jk reads words
//!   (jk − 1)·256 004 + 1 to jk·256 004, 6 400 100 words in all, and under the
//!   null the 25 results are independent.
//! - **256 004 words per window, not 256 005.**  `wknt1s` builds a five-letter
//!   initial word (line 872) whose first letter is erased before anything
//!   is counted; this module starts from four letters.  Which of the five
//!   `m8()` calls in that expression is erased depends on the compiler, since
//!   Fortran does not fix their order.  The review's gfortran build evaluates
//!   them left to right, so dropping its first word reproduces its counts.
//! - **p-value.**  erfc(|z|/√2), two-sided, as `count_ones_stream` reports
//!   it, so a Q5 − Q4 that is too small fails as well as one too large.
//!   DIEHARD prints Φ(z) (line 911), near 1 for a large statistic; the
//!   note shows it.
//!
//! # Goldens
//!
//! On words 2 to 256 005 of the input the DIEHARD fidelity review gave its
//! gfortran build of `diehard.f` (an "aligned" build, in which every window
//! starts at word 1), each window's Q5 − Q4 equals an independent
//! double-precision NumPy replica of `wknt1s` to 10⁻⁶ and the 25 values the
//! build printed to within 0.008, their two-decimal rounding plus `REAL*4`
//! sums.
//!
//! # Calibration evidence
//!
//! 10 000 streams of 6 400 100 words, each from a separately seeded PCG64
//! generator, gave 250 000 window p-values: p < 0.01 in 2 519 (1.008%;
//! binomial standard deviation 0.020%) and p < 0.001 in 244 (0.098%; 0.006%),
//! with p > 0.99 in 1.00%; a Kolmogorov–Smirnov test of the 250 000 gives
//! p = 0.31.  Q5 − Q4 had mean 2 499.99 and variance 5 034, against
//! Marsaglia's 2 500 and 5 000.  22.5% of streams had at least one of their 25
//! windows below 0.01, as 1 − 0.99²⁵ = 22.2% predicts for independent
//! windows.  Rerun on the landing tree, whose `erfc` sums Marsaglia's cPhi
//! series, another 10 000 streams gave 1.042% (0.020%) and 0.100% (0.006%), a
//! KS p of 0.09, and 23.3% of streams (standard deviation 0.4%) with some
//! window below 0.01.  A test below runs a fixed eight-stream version under
//! `cargo test --release`.
//!
//! # Why it is outside the default battery
//!
//! Commit 3b41af8 removed this crate's byte variant,
//! `count_ones_specific_bytes`, saying Dieharder's author found the byte test
//! effectively obsolete.  His judgement has two parts
//! (`libdieharder/diehard_count_1s_byte.c` lines 60–71).  Unconditionally,
//! the byte test is "LESS stringent than the stream version overall" (line 60)
//! and "vastly less sensitive than rgb_bitdist" (lines 66–67), which supports
//! leaving it out of a battery on grounds of power.  Conditionally, it "might
//! reveal problems with specific offsets ignored by the stream test", and he
//! "could fix the stream test to cycle through the possible bitlevel offsets
//! and make this test completely obsolete" (lines 68–71).
//! `dieharder -l` rates `diehard_count_1s_byte` "Good" (test 9;
//! `dieharder/list_tests.c` lines 31–36).  The removed variant was not
//! DIEHARD's test either: it read one lane, `w & 0xFF`, which is DIEHARD's
//! bits 25 to 32 alone, and scored Q5 by itself as χ²(3 124), which
//! overlapping words do not support: over 20 000 simulated streams its
//! p-value fell below 0.01 in 2.95% and below 0.001 in 0.63%.  This module
//! is DIEHARD's test.
//! It stays out of the default battery because it adds 25 slots, which is a
//! decision of its own; that battery's `count_ones_stream` and
//! [`crate::dieharder::bit_distribution`] already count the 1s in bytes.
//!
//! # Author
//! George Marsaglia, DIEHARD (1995).

use crate::{
    diehard::count_ones::{hamming_letter, q5_q4, q_difference_z, LETTERS_PER_TEST},
    math::{erfc, normal_cdf},
    result::TestResult,
};
use std::f64::consts::SQRT_2;

/// Result name, shared by the 25 windows.
const NAME: &str = "diehard_historical::count_ones_bytes_25_fresh";
/// Bit windows: bits jk to jk + 7 from the left, jk = 1 to 25.
pub const WINDOWS: usize = 25;
/// Words each window reads, one letter per word.
pub const WORDS_PER_WINDOW: usize = LETTERS_PER_TEST;
/// Words the test reads.
pub const WORDS: usize = WINDOWS * WORDS_PER_WINDOW;

/// Count-the-1s on the byte of each of DIEHARD's 25 bit windows, window jk
/// reading the jk-th block of [`WORDS_PER_WINDOW`] words: 25 results, in
/// window order.
///
/// Reports 25 SKIPs for fewer than [`WORDS`] words.  See the module
/// documentation for the statistic and its departures from `diehard.f`.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn count_ones_bytes_25_fresh(words: &[u32]) -> Vec<TestResult> {
    if words.len() < WORDS {
        return (0..WINDOWS)
            .map(|_| TestResult::insufficient(NAME, "need 6 400 100 words"))
            .collect();
    }
    words
        .chunks_exact(WORDS_PER_WINDOW)
        .take(WINDOWS)
        .enumerate()
        .map(|(i, block)| {
            let jk = i + 1;
            let (q5, q4) = window_q5_q4(block, WINDOWS - jk);
            let z = q_difference_z(q5, q4);
            TestResult::with_note(
                NAME,
                erfc(z.abs() / SQRT_2),
                format!(
                    "bits {jk} to {}, Q5-Q4={:.2}, z={z:.3}, Φ(z)={:.6}",
                    jk + 7,
                    q5 - q4,
                    normal_cdf(z)
                ),
            )
        })
        .collect()
}

/// Q5 and Q4 on the letters of the bytes `w >> shift` of `block`.
fn window_q5_q4(block: &[u32], shift: usize) -> (f64, f64) {
    q5_q4(block.iter().map(|&w| hamming_letter((w >> shift) as u8)))
}

#[cfg(test)]
mod tests {
    use super::super::oracle;
    use super::{count_ones_bytes_25_fresh, window_q5_q4, WINDOWS, WORDS, WORDS_PER_WINDOW};
    use crate::{
        math::ks_test,
        rng::{Mt19937, Pcg64, Rng},
    };

    /// Q5 − Q4 for bits 1 to 8 through 25 to 32 as the review's aligned
    /// gfortran build printed them for `in.bin` words 1 to 256 005 (two
    /// decimals, from `REAL*4` sums).
    const FORTRAN_Q5_MINUS_Q4: [f64; WINDOWS] = [
        2342.52, 2486.87, 2556.04, 2567.95, 2445.79, 2484.32, 2545.11, 2415.55, 2433.14, 2384.57,
        2461.95, 2515.16, 2460.59, 2429.04, 2580.86, 2407.12, 2513.11, 2464.67, 2498.27, 2437.60,
        2582.98, 2578.42, 2368.89, 2524.30, 2485.84,
    ];

    /// The same statistics from an independent NumPy replica of `wknt1s` in
    /// double precision, dropping word 1 as the left-to-right initial word
    /// does (ten decimals).
    const NUMPY_Q5_MINUS_Q4: [f64; WINDOWS] = [
        2_342.517_401_503_0,
        2_486.873_700_274_4,
        2_556.036_879_635_4,
        2_567.953_689_900_8,
        2_445.796_655_136_4,
        2_484.325_315_792_5,
        2_545.110_170_124_6,
        2_415.556_458_817_6,
        2_433.138_829_032_1,
        2_384.569_777_053_6,
        2_461.957_420_747_9,
        2_515.156_244_496_1,
        2_460.588_546_897_0,
        2_429.040_506_611_4,
        2_580.865_645_586_9,
        2_407.123_098_387_2,
        2_513.113_337_877_9,
        2_464.674_336_823_3,
        2_498.269_152_713_4,
        2_437.604_547_766_3,
        2_582.984_372_752_6,
        2_578.420_122_666_6,
        2_368.897_898_639_5,
        2_524.307_280_706_0,
        2_485.843_775_383_7,
    ];

    /// Fidelity: every window's Q5 − Q4 against an independent NumPy replica
    /// of `wknt1s` in double precision, to 10⁻⁶, and against the gfortran
    /// build's print.  The second tolerance, 0.01, follows that printout, not
    /// this code: two decimals from `REAL*4` sums (largest gap 0.0079).
    #[test]
    fn windows_match_the_gfortran_build_on_the_review_input() {
        let words = oracle::words(WORDS_PER_WINDOW + 1);
        let block = &words[1..];
        for jk in 1..=WINDOWS {
            let (q5, q4) = window_q5_q4(block, WINDOWS - jk);
            let d = q5 - q4;
            assert!(
                (d - NUMPY_Q5_MINUS_Q4[jk - 1]).abs() < 1e-6,
                "bits {jk}: {d}"
            );
            assert!(
                (d - FORTRAN_Q5_MINUS_Q4[jk - 1]).abs() < 0.01,
                "bits {jk}: {d}"
            );
        }
    }

    /// Sum of the 25 p-values when each window reads `in.bin` words 2 to
    /// 256 005, pinned on the landing tree, whose `erfc` sums Marsaglia's cPhi
    /// series.
    const GOLDEN_P_SUM: f64 = 11.841_968_816_007_894;

    /// Regression: the 25 results on the review input, in window order, each
    /// note heading with its window and Q5 − Q4, and the p-values pinned to
    /// 10⁻¹² through their sum.
    #[test]
    fn results_on_the_review_input_are_pinned() {
        let words = oracle::words(WORDS_PER_WINDOW + 1);
        let results = count_ones_bytes_25_fresh(&words[1..].repeat(WINDOWS));
        assert_eq!(results.len(), WINDOWS);
        for (jk, r) in (1..=WINDOWS).zip(&results) {
            let head = format!(
                "bits {jk} to {}, Q5-Q4={:.2},",
                jk + 7,
                NUMPY_Q5_MINUS_Q4[jk - 1]
            );
            assert!(r.note.as_deref().unwrap_or("").starts_with(&head), "{r}");
        }
        let sum: f64 = results.iter().map(|r| r.p_value).sum();
        assert!((sum - GOLDEN_P_SUM).abs() < 1e-12, "{sum:?}");
    }

    /// Clearing every word's top byte breaks the windows that read any of
    /// bits 1 to 8 (jk ≤ 8) and no other; zeroing window 20's block breaks
    /// window 20 alone.
    #[test]
    fn each_window_reads_its_own_bits_and_block() {
        let mut words: Vec<u32> = Mt19937::new(5489)
            .collect_u32s(WORDS)
            .into_iter()
            .map(|w| w & (u32::MAX >> 8))
            .collect();
        words[19 * WORDS_PER_WINDOW..20 * WORDS_PER_WINDOW].fill(0);
        for (jk, r) in (1..=WINDOWS).zip(count_ones_bytes_25_fresh(&words)) {
            if jk <= 8 || jk == 20 {
                assert!(r.p_value < 1e-10, "{r}");
            } else {
                assert!(r.p_value > 1e-6, "{r}");
            }
        }
    }

    #[test]
    fn short_inputs_skip_and_constant_input_fails() {
        let short = count_ones_bytes_25_fresh(&vec![0; WORDS - 1]);
        assert_eq!(short.len(), WINDOWS);
        assert!(short.iter().all(|r| r.skipped()));
        let constant = count_ones_bytes_25_fresh(&vec![0; WORDS]);
        assert_eq!(constant.len(), WINDOWS);
        assert!(constant.iter().all(|r| !r.skipped() && !r.passed()));
    }

    /// Streams from separately seeded PCG64 generators.
    const SMOKE_STREAMS: u64 = 8;

    /// A small, fixed version of the null calibration in the module
    /// documentation: the 200 window p-values of eight PCG64 streams pass a KS
    /// test and few fall below 0.01 (Binomial(200, 0.01) exceeds 7 with
    /// probability 0.001).
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "eight 6.4 million-word streams; runs under cargo test --release"
    )]
    fn null_streams_give_uniform_p_values() {
        let mut p: Vec<f64> = (0..SMOKE_STREAMS)
            .flat_map(|i| {
                let mut rng = Pcg64::new(u128::from(i), u128::from(SMOKE_STREAMS));
                count_ones_bytes_25_fresh(&rng.collect_u32s(WORDS))
            })
            .map(|r| r.p_value)
            .collect();
        let below = p.iter().filter(|&&x| x < 0.01).count();
        assert!(below <= 7, "{below} of {} below 0.01", p.len());
        let ks = ks_test(&mut p);
        assert!(ks > 1e-3, "KS p = {ks}");
    }
}
