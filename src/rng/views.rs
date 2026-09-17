//! Projections of a generator's output onto the 32-bit words the tests read.
//!
//! The 64-bit generators ([`super::Pcg64`], [`super::Xoshiro256`],
//! [`super::Xoroshiro128`], [`super::Sfc64`], [`super::Jsf64`] and
//! [`super::Xorshift64`]) return the high half of each
//! 64-bit output from `next_u32` and discard the low half, so the batteries
//! see only that projection.  A defect confined to the low bits is invisible
//! to them.  These adapters choose the view explicitly:
//!
//! - [`HighHalf`]: the high 32 bits of each 64-bit output (the default view);
//! - [`LowHalf`]: the low 32 bits;
//! - [`FullWord`]: both halves of each output, high then low;
//! - [`BitReversed`]: each 32-bit word of the inner generator bit-reversed, so
//!   bit-order-sensitive tests read its bits in the opposite order.
//!
//! Views of the same raw outputs are not independent experiments; results
//! from them share data.

use super::Rng;

/// The high 32 bits of each 64-bit output.
pub struct HighHalf<R>(pub R);

impl<R: Rng> Rng for HighHalf<R> {
    fn next_u32(&mut self) -> u32 {
        (self.0.next_u64() >> 32) as u32
    }
}

/// The low 32 bits of each 64-bit output.
pub struct LowHalf<R>(pub R);

impl<R: Rng> Rng for LowHalf<R> {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u64() as u32
    }
}

/// Both halves of each 64-bit output, the high half first.
pub struct FullWord<R> {
    inner: R,
    low: Option<u32>,
}

impl<R> FullWord<R> {
    /// Wrap `inner`.
    pub fn new(inner: R) -> Self {
        Self { inner, low: None }
    }
}

impl<R: Rng> Rng for FullWord<R> {
    fn next_u32(&mut self) -> u32 {
        if let Some(low) = self.low.take() {
            return low;
        }
        let word = self.inner.next_u64();
        self.low = Some(word as u32);
        (word >> 32) as u32
    }
}

/// Each 32-bit word of the inner generator with its bits reversed.
pub struct BitReversed<R>(pub R);

impl<R: Rng> Rng for BitReversed<R> {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32().reverse_bits()
    }
}

#[cfg(test)]
mod tests {
    use super::{BitReversed, FullWord, HighHalf, LowHalf};
    #[cfg(feature = "batteries")]
    use crate::nist::frequency::frequency;
    use crate::rng::Rng;

    /// Emits 0x0123_4567_89ab_cdef, then that plus one, and so on.
    struct Counter64(u64);

    impl Rng for Counter64 {
        fn next_u32(&mut self) -> u32 {
            (self.next_u64() >> 32) as u32
        }
        fn next_u64(&mut self) -> u64 {
            let x = self.0;
            self.0 = self.0.wrapping_add(1);
            x
        }
    }

    const START: u64 = 0x0123_4567_89ab_cdef;

    #[test]
    fn views_select_the_documented_bits() {
        assert_eq!(HighHalf(Counter64(START)).next_u32(), 0x0123_4567);
        assert_eq!(LowHalf(Counter64(START)).next_u32(), 0x89ab_cdef);
        let mut full = FullWord::new(Counter64(START));
        assert_eq!(
            [full.next_u32(), full.next_u32(), full.next_u32()],
            [0x0123_4567, 0x89ab_cdef, 0x0123_4567]
        );
        assert_eq!(
            BitReversed(HighHalf(Counter64(START))).next_u32(),
            0x0123_4567u32.reverse_bits()
        );
    }

    /// Good high bits over constant low bits: the default view passes a
    /// frequency test and the low view fails it.
    #[cfg(feature = "batteries")]
    #[test]
    fn low_half_view_exposes_a_low_bit_defect() {
        struct BrokenLow(crate::rng::Pcg64);
        impl Rng for BrokenLow {
            fn next_u32(&mut self) -> u32 {
                (self.next_u64() >> 32) as u32
            }
            fn next_u64(&mut self) -> u64 {
                self.0.next_u64() & !u64::from(u32::MAX)
            }
        }
        let high = HighHalf(BrokenLow(crate::rng::Pcg64::new(1, 1))).collect_bits(100_000);
        let low = LowHalf(BrokenLow(crate::rng::Pcg64::new(1, 1))).collect_bits(100_000);
        assert!(frequency(&high).passed());
        assert!(frequency(&low).p_value < 1e-10);
    }
}
