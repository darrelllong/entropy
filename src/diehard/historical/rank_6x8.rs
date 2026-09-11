//! The 6×8 binary rank test over DIEHARD's 25 bit windows, each window on its
//! own words, with one Anderson–Darling summary.  Result name:
//! `diehard_historical::rank_6x8_25_fresh`.
//!
//! # What it is
//!
//! Six bytes from six successive words, bits b to b + 7 counting from the
//! left, form a 6 × 8 matrix over GF(2), whose rank is 6 with probability
//! 0.7731, 5 with probability 0.2174 and at most 4 with probability 0.0094.
//! For each of 25 windows, b = 1 to 25, 100 000 matrices give a χ² on the
//! counts of those three cells (df 2) and the p-value exp(−χ²/2).  An
//! Anderson–Darling statistic of the 25 p-values against U(0, 1) gives the
//! single result, p = 1 − CDF.
//!
//! # Reference followed
//!
//! Marsaglia's `cdbinrnk`, `fortran/diehard.f` lines 920–1001
//! [pubs/diehard-fortran-1996.tar.gz]:
//!
//! - rows `i8bit()=and(rshift(jtbl(),kr),255)` (line 932) for
//!   `do 55 ij=25,1,-1`, kr = ij − 1 (lines 946–950), printed as
//!   bits 25 − kr to 32 − kr;
//! - 100 000 matrices per window (line 959), ranks up to 4 pooled by
//!   `mr=max(4,rankb(r,6,8))` (line 962), and a three-cell χ²;
//! - `pp(kr)=1.-exp(-s/2)` (line 976) and `KSTEST` over the 25
//!   (line 993), an Anderson–Darling test that `tests.txt` calls a
//!   Kolmogorov–Smirnov test.
//!
//! The rank counts and χ² are those of the default battery's
//! [`crate::diehard::binary_rank::binary_rank_6x8`], whose low byte is window
//! 25 and whose χ² on that window equals `diehard.f`'s on the same words.
//!
//! # Departures from `diehard.f`
//!
//! - **Fresh words for each window.**  Before each window `cdbinrnk` calls
//!   `jkreset` (line 947), which resets `jtbl`'s record counter but not
//!   its place in the current 4 096-word record (lines 414–428).  Each window
//!   after the first therefore reads out the rest of that record, 128 to
//!   3 520 words, and then rereads the file from word 1: run alone, window 2
//!   starts at word 600 001 and window 25 at word 596 481.  Its 25 windows
//!   share nearly all their words, with matrix boundaries shifted by 0, 2 or 4
//!   words, so their p-values are dependent, which its summary does not allow
//!   for.  Simulated with all 25 windows on the same 600 000 words, as the
//!   aligned build reads them, the summary fell below 0.01 in 2.55% and
//!   below 0.001 in 0.45% of 10 000 streams.  Here window b reads words
//!   (b − 1)·600 000 + 1 to b·600 000, 15 000 000 words in all, and under the
//!   null the 25 p-values are independent.
//! - **Cell probabilities** are exact, where line 929 has six digits.
//! - **p-values.**  Each window's is the upper tail exp(−χ²/2), where DIEHARD
//!   takes 1 − exp(−χ²/2); A² is the same for either.  The summary's
//!   distribution function is [`crate::math::anderson_darling_cdf`], and the
//!   result is 1 − CDF.  `KSTEST` uses Marsaglia's older approximation, and
//!   DIEHARD prints the CDF, which the note shows.
//!
//! # Goldens
//!
//! On words 1 to 600 000 of the input the DIEHARD fidelity review gave its
//! gfortran build of `diehard.f` (an "aligned" build, in which every window
//! starts at word 1), each window's three rank counts equal the build's, all
//! 75 of them.  With DIEHARD's six-digit probabilities the 25 χ² values match
//! its printed sums to within 4.8·10⁻⁴ and its `KSTEST` summary (0.450587) to
//! within 6.6·10⁻⁶; with the exact probabilities used here χ² moves by up to
//! 0.0015.
//!
//! # A limitation of the summary
//!
//! The Anderson–Darling statistic weighs the smallest p-value by 1/25, and
//! `KSTEST` floors each product at 10⁻²⁰, so one window with p = 0 raises A²
//! by at most ln(10²⁰)/25 ≈ 1.8: a generator with a single broken window can
//! pass the summary, as a test below shows.  The note gives the smallest
//! window p-value and its bits.
//!
//! # Calibration evidence
//!
//! 10 000 streams of 15 000 000 words, each from a separately seeded PCG64
//! generator: the summary's p-value fell below 0.01 in 94 (0.94%; binomial
//! standard deviation 0.10%) and below 0.001 in 15 (0.15%; 0.03%), with
//! p > 0.99 in 0.83%; a Kolmogorov–Smirnov test of the 10 000 p-values gives
//! p = 0.61.  A test below runs a fixed 20-stream version under
//! `cargo test --release`.
//!
//! # Why it is outside the default battery
//!
//! The default battery's `binary_rank_6x8` reads one window, the low byte, on
//! 600 000 words (AUDIT.md item 10).  The DIEHARD fidelity review recommended
//! replacing it with this sweep; doing so would change that battery's output,
//! which this suite must not do, and the decision is a separate one.  Until it
//! is taken the sweep runs here.
//!
//! # Author
//! George Marsaglia, DIEHARD (1995).

use super::anderson_darling_statistic;
use crate::{
    diehard::binary_rank::{
        rank_6x8_chi_square, rank_6x8_counts, RANK_6X8_MATRICES, RANK_6X8_ROWS,
    },
    math::{anderson_darling_cdf, igamc},
    result::TestResult,
};

/// Result name.
const NAME: &str = "diehard_historical::rank_6x8_25_fresh";
/// Bit windows: bits b to b + 7 from the left, b = 1 to 25.
pub const WINDOWS: usize = 25;
/// Words each window reads: six rows for each of 100 000 matrices.
pub const WORDS_PER_WINDOW: usize = RANK_6X8_ROWS * RANK_6X8_MATRICES;
/// Words the test reads.
pub const WORDS: usize = WINDOWS * WORDS_PER_WINDOW;

/// The 6×8 rank test on each of DIEHARD's 25 bit windows, window b reading
/// the b-th block of [`WORDS_PER_WINDOW`] words, summarised by one
/// Anderson–Darling test.
///
/// Reports SKIP for fewer than [`WORDS`] words.  See the module documentation
/// for the statistic and its departures from `diehard.f`.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn rank_6x8_25_fresh(words: &[u32]) -> TestResult {
    if words.len() < WORDS {
        return TestResult::insufficient(NAME, "need 15 000 000 words");
    }
    let windows = windows(&words[..WORDS]);
    let mut p: Vec<f64> = windows.iter().map(|w| w.p_value).collect();
    let a2 = anderson_darling_statistic(&mut p);
    let cdf = anderson_darling_cdf(WINDOWS, a2);
    let worst = windows
        .iter()
        .min_by(|a, b| a.p_value.total_cmp(&b.p_value))
        .expect("25 windows");
    TestResult::with_note(
        NAME,
        1.0 - cdf,
        format!(
            "25 windows of {RANK_6X8_MATRICES} matrices, A²={a2:.4}, CDF={cdf:.6}; \
             smallest window p={:.6} (bits {} to {}, χ²={:.4})",
            worst.p_value,
            25 - worst.shift,
            32 - worst.shift,
            worst.chi_square
        ),
    )
}

/// One bit window's χ² and p-value; `shift` is DIEHARD's kr.
struct Window {
    shift: u32,
    chi_square: f64,
    p_value: f64,
}

/// The 25 windows of exactly [`WORDS`] words, bits 1 to 8 first.
fn windows(words: &[u32]) -> Vec<Window> {
    words
        .chunks_exact(WORDS_PER_WINDOW)
        .zip((0..WINDOWS as u32).rev())
        .map(|(block, shift)| {
            let chi_square = rank_6x8_chi_square(&window_counts(block, shift));
            Window {
                shift,
                chi_square,
                p_value: igamc(1.0, chi_square / 2.0),
            }
        })
        .collect()
}

/// Rank counts (≤ 4, 5, 6) of the matrices whose rows are the bytes
/// `w >> shift` of `block`.
fn window_counts(block: &[u32], shift: u32) -> [usize; 3] {
    rank_6x8_counts(block.iter().map(|&w| u32::from((w >> shift) as u8)))
}

#[cfg(test)]
mod tests {
    use super::super::{anderson_darling_statistic, oracle};
    use super::{rank_6x8_25_fresh, window_counts, windows, WINDOWS, WORDS, WORDS_PER_WINDOW};
    use crate::{
        diehard::binary_rank::{rank_6x8_chi_square, RANK_6X8_MATRICES},
        math::ks_test,
        rng::{Mt19937, Pcg64, Rng},
    };

    /// Rank counts (≤ 4, 5, 6) and χ² ("SUM", three decimals) for bits 1 to 8
    /// through 25 to 32 as the review's aligned gfortran build printed them
    /// for `in.bin` words 1 to 600 000.
    const FORTRAN_WINDOWS: [([usize; 3], f64); WINDOWS] = [
        ([976, 21615, 77409], 1.950),
        ([949, 21813, 77238], 0.313),
        ([922, 21803, 77275], 0.705),
        ([923, 21545, 77532], 2.927),
        ([969, 21488, 77543], 4.349),
        ([904, 21765, 77331], 1.745),
        ([971, 21812, 77217], 1.084),
        ([971, 21723, 77306], 0.775),
        ([939, 21917, 77144], 1.772),
        ([955, 21776, 77269], 0.192),
        ([991, 21550, 77459], 4.319),
        ([973, 21605, 77422], 1.917),
        ([925, 21623, 77452], 1.321),
        ([931, 21485, 77584], 4.228),
        ([904, 21621, 77475], 2.759),
        ([906, 21665, 77429], 2.017),
        ([968, 21571, 77461], 2.258),
        ([968, 21593, 77439], 1.851),
        ([967, 21734, 77299], 0.552),
        ([934, 21805, 77261], 0.317),
        ([952, 21705, 77343], 0.145),
        ([972, 21576, 77452], 2.363),
        ([983, 21585, 77432], 2.934),
        ([962, 21733, 77305], 0.338),
        ([906, 21839, 77255], 2.011),
    ];
    /// The build's `KSTEST` over its 25 p-values ("KS p-value").
    const FORTRAN_SUMMARY: f64 = 0.450587;

    /// DIEHARD's six-digit cell probabilities (`fortran/diehard.f` line 929):
    /// ranks 2 to 4 pooled, rank 5 and rank 6.
    const DIEHARD_PROBABILITIES: [f64; 3] =
        [0.149858e-6 + 0.808926e-4 + 0.936197e-2, 0.217439, 0.773118];

    /// χ² as `cdbinrnk` computes it, with its probabilities.
    fn diehard_chi_square(counts: &[usize; 3]) -> f64 {
        DIEHARD_PROBABILITIES
            .iter()
            .zip(counts)
            .map(|(&p, &c)| {
                let e = RANK_6X8_MATRICES as f64 * p;
                (c as f64 - e).powi(2) / e
            })
            .sum()
    }

    /// This module's p-value when every window reads those words, pinned.
    const GOLDEN_P: f64 = 0.548_258_484_323_871_1;

    /// All 75 rank counts equal the build's.  With its six-digit probabilities
    /// the χ² values match its three-decimal sums to 6·10⁻⁴ (largest gap
    /// 4.8·10⁻⁴) and its `KSTEST` over the 25 p-values matches its summary to
    /// 2·10⁻⁵ (gap 6.6·10⁻⁶); the exact probabilities move χ² by up to 0.0015.
    #[test]
    fn windows_match_the_gfortran_build_on_the_review_input() {
        let words = oracle::words(WORDS_PER_WINDOW);
        let mut fortran_p = Vec::with_capacity(WINDOWS);
        for (b, (counts, sum)) in (1..).zip(&FORTRAN_WINDOWS) {
            let got = window_counts(&words, (WINDOWS - b) as u32);
            assert_eq!(&got, counts, "bits {b} to {}", b + 7);
            let exact = rank_6x8_chi_square(&got);
            assert!((exact - sum).abs() < 0.002, "bits {b}: χ² {exact} vs {sum}");
            let diehard = diehard_chi_square(&got);
            assert!(
                (diehard - sum).abs() < 6e-4,
                "bits {b}: χ² {diehard} vs {sum}"
            );
            fortran_p.push(1.0 - (-diehard / 2.0).exp());
        }
        let a2 = anderson_darling_statistic(&mut fortran_p);
        let summary = oracle::diehard_kstest_cdf(WINDOWS, a2);
        assert!(
            (summary - FORTRAN_SUMMARY).abs() < 2e-5,
            "summary {summary}"
        );

        let result = rank_6x8_25_fresh(&words.repeat(WINDOWS));
        assert!((result.p_value - GOLDEN_P).abs() < 1e-12, "{result}");
    }

    /// Zeroing window 8's byte (bits 8 to 15) in window 8's block gives every
    /// matrix there rank 0: that window's p-value is 0 and no other window
    /// moves.  The summary does not fail on that one window, because its
    /// floored product adds at most ln(10²⁰)/25 ≈ 1.8 to A²; the note names it.
    #[test]
    fn each_window_reads_its_own_bits_and_block() {
        let mut words = Mt19937::new(5489).collect_u32s(WORDS);
        let b = 8;
        let byte = u32::from(u8::MAX) << (WINDOWS - b);
        for w in &mut words[(b - 1) * WORDS_PER_WINDOW..b * WORDS_PER_WINDOW] {
            *w &= !byte;
        }
        for (i, w) in (1..).zip(windows(&words)) {
            if i == b {
                assert!(w.p_value < 1e-10, "bits {i}: p = {}", w.p_value);
            } else {
                assert!(w.p_value > 1e-6, "bits {i}: p = {}", w.p_value);
            }
        }
        let result = rank_6x8_25_fresh(&words);
        let note = result.note.as_deref().unwrap_or_default();
        assert!(
            note.contains("smallest window p=0.000000 (bits 8 to 15"),
            "{result}"
        );
    }

    #[test]
    fn short_inputs_skip_and_constant_input_fails() {
        assert!(rank_6x8_25_fresh(&[]).skipped());
        assert!(rank_6x8_25_fresh(&vec![0; WORDS - 1]).skipped());
        let r = rank_6x8_25_fresh(&vec![0; WORDS]);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }

    /// Streams from separately seeded PCG64 generators.
    const SMOKE_STREAMS: u64 = 20;

    /// A small, fixed version of the null calibration in the module
    /// documentation: over 20 PCG64 streams the 500 window p-values pass a KS
    /// test, and at most two summaries fall below 0.01 (Binomial(20, 0.01)
    /// exceeds 2 with probability 0.001).
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "twenty 15 million-word streams; runs under cargo test --release"
    )]
    fn null_streams_give_uniform_p_values() {
        let mut window_p = Vec::new();
        let mut below = 0;
        for i in 0..SMOKE_STREAMS {
            let words = Pcg64::new(u128::from(i), u128::from(SMOKE_STREAMS)).collect_u32s(WORDS);
            window_p.extend(windows(&words).iter().map(|w| w.p_value));
            below += usize::from(rank_6x8_25_fresh(&words).p_value < 0.01);
        }
        assert!(
            below <= 2,
            "{below} of {SMOKE_STREAMS} summaries below 0.01"
        );
        let ks = ks_test(&mut window_p);
        assert!(ks > 1e-3, "KS p = {ks}");
    }
}
