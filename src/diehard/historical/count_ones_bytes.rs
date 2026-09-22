//! Count-the-1s on specific bytes, at each of the 25 byte offsets of a word.
//! Result name: `diehard_historical::count_ones_bytes`, 25 results, one per
//! offset.
//!
//! # The statistic
//!
//! Window b, for b = 1 to 25, takes from each word the byte at bits b to
//! b + 7 counting from the most significant bit, that is `(w >> (25 − b)) &
//! 255`.  Each byte becomes a letter by its Hamming weight, and the Q5 − Q4
//! statistic of [`crate::diehard::count_ones`] is computed over 256 000
//! overlapping five-letter words, with its two-sided p-value in χ²(2 500).
//!
//! Window b reads its own block of 256 004 words, the b-th in the input
//! (6 400 100 words in all), so under the null the 25 results are
//! independent.  No summary over the windows is reported; a fault confined to
//! some bit positions shows in the windows that read them.
//!
//! # Calibration
//!
//! Q5 − Q4 is only approximately χ²(2 500): over 5 000 000 null windows its
//! mean is 2 500.00 and its standard deviation 71.07 against √5000 = 70.71,
//! the same in every window, and scored against χ²(2 500) it rejected 1.04%
//! of them at the 1% level.  [`crate::diehard::count_ones`] scores it against
//! the gamma law with that measured variance instead.  A test below runs a
//! fixed eight-stream version under `cargo test --release`.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{
    diehard::count_ones::{
        hamming_letter, q5_q4, q_difference_p_value, LETTERS_PER_TEST, QDIFF_NULL,
    },
    result::TestResult,
};

/// Result name, shared by the 25 windows.
const NAME: &str = "diehard_historical::count_ones_bytes";
/// Byte windows: bits b to b + 7 from the most significant, b = 1 to 25.
pub const WINDOWS: usize = 25;
/// Words each window reads, one letter per word.
pub const WORDS_PER_WINDOW: usize = LETTERS_PER_TEST;
/// Words the test reads.
pub const WORDS: usize = WINDOWS * WORDS_PER_WINDOW;

/// Count-the-1s on the byte of each of the 25 windows, window b reading the
/// b-th block of [`WORDS_PER_WINDOW`] words: 25 results, in window order.
///
/// Reports 25 SKIPs for fewer than [`WORDS`] words.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn count_ones_bytes(words: &[u32]) -> Vec<TestResult> {
    if words.len() < WORDS {
        return (0..WINDOWS)
            .map(|_| TestResult::insufficient(NAME, "need 6 400 100 words"))
            .collect();
    }
    words
        .chunks_exact(WORDS_PER_WINDOW)
        .take(WINDOWS)
        .zip(1..=WINDOWS)
        .map(|(block, b)| {
            let (q5, q4) = window_q5_q4(block, WINDOWS - b);
            TestResult::with_note(
                NAME,
                q_difference_p_value(q5, q4),
                format!("bits {b} to {}, Q5-Q4={:.2}", b + 7, q5 - q4),
            )
            .with_statistic("Q5 - Q4", q5 - q4, Some(2_500.0), QDIFF_NULL)
        })
        .collect()
}

/// Q5 and Q4 on the letters of the bytes `w >> shift` of `block`.
fn window_q5_q4(block: &[u32], shift: usize) -> (f64, f64) {
    q5_q4(block.iter().map(|&w| hamming_letter((w >> shift) as u8)))
}

#[cfg(test)]
mod tests {
    use super::{count_ones_bytes, WINDOWS, WORDS, WORDS_PER_WINDOW};
    use crate::{
        math::ks_test,
        rng::{Mt19937, Pcg64, Rng},
    };

    /// Sum of the 25 p-values on a fixed PCG64 stream, pinned.
    const GOLDEN_P_SUM: f64 = 12.581_780_947_500_436;

    /// The 25 results on a fixed stream, in window order, pinned
    /// through their sum.
    #[test]
    fn results_on_a_fixed_stream_are_pinned() {
        let words = Pcg64::new(20_260_916, 25).collect_u32s(WORDS);
        let results = count_ones_bytes(&words);
        assert_eq!(results.len(), WINDOWS);
        for (b, r) in (1..=WINDOWS).zip(&results) {
            let head = format!("bits {b} to {},", b + 7);
            assert!(r.note.as_deref().unwrap_or("").starts_with(&head), "{r}");
        }
        let sum: f64 = results.iter().map(|r| r.p_value).sum();
        assert!((sum - GOLDEN_P_SUM).abs() < 1e-12, "{sum:?}");
    }

    /// Clearing every word's top byte breaks the windows that read any of
    /// bits 1 to 8 (b ≤ 8) and no other; zeroing window 20's block breaks
    /// window 20 alone.
    #[test]
    fn each_window_reads_its_own_bits_and_block() {
        let mut words: Vec<u32> = Mt19937::new(5489)
            .collect_u32s(WORDS)
            .into_iter()
            .map(|w| w & (u32::MAX >> 8))
            .collect();
        words[19 * WORDS_PER_WINDOW..20 * WORDS_PER_WINDOW].fill(0);
        for (b, r) in (1..=WINDOWS).zip(count_ones_bytes(&words)) {
            if b <= 8 || b == 20 {
                assert!(r.p_value < 1e-10, "{r}");
            } else {
                assert!(r.p_value > 1e-6, "{r}");
            }
        }
    }

    #[test]
    fn short_inputs_skip_and_constant_input_fails() {
        let short = count_ones_bytes(&vec![0; WORDS - 1]);
        assert_eq!(short.len(), WINDOWS);
        assert!(short.iter().all(|r| r.skipped()));
        let constant = count_ones_bytes(&vec![0; WORDS]);
        assert_eq!(constant.len(), WINDOWS);
        assert!(constant.iter().all(|r| !r.skipped() && !r.passed()));
    }

    /// Streams from separately seeded PCG64 generators.
    const SMOKE_STREAMS: u64 = 8;

    /// The 200 window p-values of eight PCG64 streams pass a KS test and few
    /// fall below 0.01 (Binomial(200, 0.01) exceeds 7 with probability 0.001).
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "eight 6.4 million-word streams; runs under cargo test --release"
    )]
    fn null_streams_give_uniform_p_values() {
        let mut p: Vec<f64> = (0..SMOKE_STREAMS)
            .flat_map(|i| {
                let mut rng = Pcg64::new(u128::from(i), u128::from(SMOKE_STREAMS));
                count_ones_bytes(&rng.collect_u32s(WORDS))
            })
            .map(|r| r.p_value)
            .collect();
        let below = p.iter().filter(|&&x| x < 0.01).count();
        assert!(below <= 7, "{below} of {} below 0.01", p.len());
        let ks = ks_test(&mut p);
        assert!(ks > 1e-3, "KS p = {ks}");
    }
}
