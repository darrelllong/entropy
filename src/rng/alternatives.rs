//! Generators with a specified defect, for measuring the power of the tests.
//!
//! Each adapter wraps a generator and changes its words in one defined way:
//!
//! - [`Biased`]: every bit is 1 with probability ½ + 2^−(k+1), independently,
//!   from the OR of one word with the AND of k more;
//! - [`StuckLowBits`]: the low `bits` bits of every word are 0;
//! - [`RepeatedBlocks`]: each block of `len` words is emitted twice;
//! - [`LaggedMsb`]: the most significant bit of each word copies bit 30 of the
//!   word before it, the other bits independent;
//! - [`ShortPeriod`]: the first `period` words repeat forever.
//!
//! A test that fails on one of these has detected that defect at the battery's
//! sample size; one that passes has not.

use super::Rng;

/// Every bit 1 with probability ½ + 2^−(k+1).
pub struct Biased<R> {
    inner: R,
    k: u32,
}

impl<R> Biased<R> {
    /// Bias 2^−(k+1) above one half; `k` ≥ 1.
    ///
    /// # Panics
    /// Panics if `k` is 0.
    pub fn new(inner: R, k: u32) -> Self {
        assert!(k >= 1, "k must be at least 1");
        Self { inner, k }
    }
}

impl<R: Rng> Rng for Biased<R> {
    fn next_u32(&mut self) -> u32 {
        let base = self.inner.next_u32();
        let and = (0..self.k).fold(u32::MAX, |acc, _| acc & self.inner.next_u32());
        base | and
    }
}

/// The low `bits` bits of every word cleared.
pub struct StuckLowBits<R> {
    inner: R,
    mask: u32,
}

impl<R> StuckLowBits<R> {
    /// Clear the low `bits` bits, 1 ≤ `bits` ≤ 31.
    ///
    /// # Panics
    /// Panics if `bits` is outside 1 ..= 31.
    pub fn new(inner: R, bits: u32) -> Self {
        assert!((1..=31).contains(&bits), "bits must be in 1..=31");
        Self {
            inner,
            mask: u32::MAX << bits,
        }
    }
}

impl<R: Rng> Rng for StuckLowBits<R> {
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32() & self.mask
    }
}

/// Each block of `len` words emitted twice.
pub struct RepeatedBlocks<R> {
    inner: R,
    block: Vec<u32>,
    next: usize,
    second_pass: bool,
}

impl<R> RepeatedBlocks<R> {
    /// Blocks of `len` ≥ 1 words.
    ///
    /// # Panics
    /// Panics if `len` is 0.
    pub fn new(inner: R, len: usize) -> Self {
        assert!(len >= 1, "len must be at least 1");
        Self {
            inner,
            block: vec![0; len],
            next: len,
            second_pass: true,
        }
    }
}

impl<R: Rng> Rng for RepeatedBlocks<R> {
    fn next_u32(&mut self) -> u32 {
        if self.next == self.block.len() {
            self.next = 0;
            self.second_pass = !self.second_pass;
            if !self.second_pass {
                for w in &mut self.block {
                    *w = self.inner.next_u32();
                }
            }
        }
        let w = self.block[self.next];
        self.next += 1;
        w
    }
}

/// The most significant bit of each word equal to bit 30 of the previous word.
pub struct LaggedMsb<R> {
    inner: R,
    previous: Option<u32>,
}

impl<R> LaggedMsb<R> {
    /// Wrap `inner`; the first word is unchanged.
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            previous: None,
        }
    }
}

impl<R: Rng> Rng for LaggedMsb<R> {
    fn next_u32(&mut self) -> u32 {
        let w = self.inner.next_u32();
        let out = match self.previous {
            Some(p) => (w & 0x7fff_ffff) | ((p >> 30) & 1) << 31,
            None => w,
        };
        self.previous = Some(out);
        out
    }
}

/// The first `period` words of the generator, repeated forever.
pub struct ShortPeriod {
    words: Vec<u32>,
    next: usize,
}

impl ShortPeriod {
    /// Record `period` ≥ 1 words of `inner`.
    ///
    /// # Panics
    /// Panics if `period` is 0.
    pub fn new(mut inner: impl Rng, period: usize) -> Self {
        assert!(period >= 1, "period must be at least 1");
        Self {
            words: inner.collect_u32s(period),
            next: 0,
        }
    }
}

impl Rng for ShortPeriod {
    fn next_u32(&mut self) -> u32 {
        let w = self.words[self.next];
        self.next = (self.next + 1) % self.words.len();
        w
    }
}

#[cfg(test)]
mod tests {
    use super::{Biased, LaggedMsb, RepeatedBlocks, ShortPeriod, StuckLowBits};
    use crate::rng::{CounterRng, Pcg64, Rng};

    /// The fraction of one bits approaches ½ + 2^−(k+1).
    #[test]
    fn biased_bits_have_the_specified_mean() {
        let mut r = Biased::new(Pcg64::new(1, 1), 3);
        let n = 200_000;
        let ones: u64 = (0..n).map(|_| u64::from(r.next_u32().count_ones())).sum();
        let mean = ones as f64 / (32 * n) as f64;
        let sd = (0.25 / (32 * n) as f64).sqrt();
        assert!((mean - 0.5625).abs() < 4.0 * sd, "{mean}");
    }

    #[test]
    fn stuck_low_bits_are_zero() {
        let mut r = StuckLowBits::new(CounterRng::new(0x0000_00ff), 4);
        assert!((0..100).all(|_| r.next_u32() & 0xf == 0));
    }

    /// Words 1 … 3 then 1 … 3 again, then 4 … 6 twice.
    #[test]
    fn blocks_repeat_once() {
        let mut r = RepeatedBlocks::new(CounterRng::new(1), 3);
        assert_eq!(r.collect_u32s(12), [1, 2, 3, 1, 2, 3, 4, 5, 6, 4, 5, 6]);
    }

    #[test]
    fn lagged_msb_copies_bit_30_of_the_previous_word() {
        let mut r = LaggedMsb::new(Pcg64::new(3, 3));
        let words = r.collect_u32s(1_000);
        assert!(words.windows(2).all(|w| (w[0] >> 30) & 1 == w[1] >> 31));
        // The top bit still varies.
        assert!(words.iter().any(|w| w >> 31 == 0) && words.iter().any(|w| w >> 31 == 1));
    }

    #[test]
    fn short_period_repeats() {
        let mut r = ShortPeriod::new(CounterRng::new(7), 4);
        assert_eq!(r.collect_u32s(9), [7, 8, 9, 10, 7, 8, 9, 10, 7]);
    }
}
