//! Linear Congruential Generators — several parameter sets, some notoriously weak.
//!
//! LCGs have the form `x_(n+1) = (a*x_n + c) mod m`.
//! These are parameterized mathematical generators, not necessarily faithful
//! libc APIs. Historical Unix libc wrappers live in [`super::c_stdlib`].
//!
//! # References
//! * K. Thompson and D. M. Ritchie, *Unix Programmer's Manual*, 7th Edition,
//!   Bell Laboratories, 1979.
//!   [Source of the `AnsiC` / System V `rand()` parameters a=1103515245, c=12345]
//! * S. K. Park and K. W. Miller, "Random number generators: good ones are
//!   hard to find," *Communications of the ACM* 31(10), pp. 1192–1201, 1988.
//!   DOI: 10.1145/63039.63042.
//!   [MINSTD: a=16807, c=0, m=2³¹−1]

use super::Rng;
use super::c_stdlib::PackedBits;

/// Which parameter set to use.
#[derive(Debug, Clone, Copy)]
pub enum LcgVariant {
    /// The classic ANSI C / old-POSIX sample LCG:
    /// `x = x * 1103515245 + 12345 (mod 2^31)`.
    ///
    /// This is the parameter set widely printed in manuals and sample code,
    /// but it is not glibc's actual `rand()` implementation.
    AnsiC,
    /// MINSTD (Park & Miller, 1988): a = 16_807, c = 0, m = 2³¹ − 1.
    /// Passes some tests but fails spectral and serial tests.
    Minstd,
    /// Borland C++ `rand()`: a = 22_695_477, c = 1, m = 2³².
    Borland,
    /// Microsoft Visual C `rand()`: a = 214_013, c = 2_531_011, m = 2³².
    /// Returns bits 30..16 — very poor quality.
    Msvc,
}

/// A 32-bit Linear Congruential Generator.
///
/// `next_u32` always returns a full 32-bit value: when a variant's raw
/// output is narrower than 32 bits (e.g. Borland/MSVC `rand()` return only
/// 15 bits), `next_u32` packs successive raw outputs together.  Use
/// [`Lcg32::next_raw`] when you want the bit-narrow C API value verbatim.
#[derive(Debug, Clone)]
pub struct Lcg32 {
    state: u64,
    a: u64,
    c: u64,
    m: u64,
    /// How many bits to right-shift the state before returning (some LCGs
    /// discard low bits).
    shift: u32,
    /// Optional output mask for libc-style generators that return fewer than
    /// 32 significant bits after shifting.
    output_mask: u32,
    /// Number of significant bits emitted per raw step (= popcount(output_mask)).
    raw_bits: u32,
    /// Bit accumulator — drives the packing path in `next_u32` for variants
    /// whose raw output is narrower than 32 bits.  Unused (and zero-cost) for
    /// 32-bit-wide variants.
    bits: PackedBits,
}

impl Lcg32 {
    pub fn new(variant: LcgVariant, seed: u64) -> Self {
        let (state, a, c, m, shift, output_mask) = match variant {
            LcgVariant::AnsiC => (seed & 0x7FFF_FFFF, 1_103_515_245, 12_345, 1 << 31, 0, u32::MAX),
            LcgVariant::Minstd => (
                if seed == 0 { 1 } else { seed % 2_147_483_647 },
                16_807,
                0,
                2_147_483_647,
                0,
                u32::MAX,
            ),
            // Borland and MSVC return 15-bit values: `(state >> 16) & 0x7FFF`.
            LcgVariant::Borland => (seed & 0xFFFF_FFFF, 22_695_477, 1, 1u64 << 32, 16, 0x7FFF),
            LcgVariant::Msvc => (seed & 0xFFFF_FFFF, 214_013, 2_531_011, 1u64 << 32, 16, 0x7FFF),
        };
        let raw_bits = output_mask.count_ones();
        Self {
            state,
            a,
            c,
            m,
            shift,
            output_mask,
            raw_bits,
            bits: PackedBits::default(),
        }
    }

    /// Convenience: ANSI C sample LCG with seed 1.
    pub fn ansi_c() -> Self {
        Self::new(LcgVariant::AnsiC, 1)
    }

    /// Convenience: MINSTD with seed 1.
    pub fn minstd() -> Self {
        Self::new(LcgVariant::Minstd, 1)
    }

    /// One raw LCG step.  Returns the bit-narrow value defined by the
    /// variant — e.g. the 15-bit `(state >> 16) & 0x7FFF` for Borland/MSVC.
    /// This matches the underlying C API; statistical-test consumers should
    /// prefer `next_u32` instead, which packs raws into a full 32-bit word.
    pub fn next_raw(&mut self) -> u32 {
        self.state = (self.a.wrapping_mul(self.state).wrapping_add(self.c)) % self.m;
        ((self.state >> self.shift) as u32) & self.output_mask
    }
}

impl Rng for Lcg32 {
    fn next_u32(&mut self) -> u32 {
        // Fast path: variants whose raw output already fills 32 bits
        // (AnsiC's 31-bit and MINSTD's 31-bit outputs are zero-extended
        // exactly as the C function would, preserving the documented
        // [0, 2^31) range collapse used as a negative control).
        if self.output_mask == u32::MAX {
            return self.next_raw();
        }
        // Pack raw_bits-wide raws into 32-bit words to keep the `next_u32`
        // contract (a 32-bit pseudo-random word).  Without this path a
        // 15-bit Borland/MSVC raw would leave the high 17 bits permanently
        // zero — the bug callers were hitting before this fix.
        while self.bits.bits < 32 {
            let raw = self.next_raw();
            self.bits.push(raw, self.raw_bits);
        }
        self.bits.pop_word()
    }
}

#[cfg(test)]
mod tests {
    use super::{Lcg32, LcgVariant};
    use crate::rng::Rng;

    #[test]
    fn msvc_raw_matches_known_seed_1_prefix() {
        // C-API faithfulness: 15-bit values from Microsoft Visual C `rand()`.
        let mut rng = Lcg32::new(LcgVariant::Msvc, 1);
        let got: Vec<u32> = (0..5).map(|_| rng.next_raw()).collect();
        assert_eq!(got, vec![41, 18_467, 6_334, 26_500, 19_169]);
    }

    #[test]
    fn msvc_next_u32_packs_raws_to_full_width() {
        // Statistical-test view: every u32 produced should occupy the full
        // 32-bit width — no permanently-zero high bits.  Across many calls
        // we should see at least one bit set in the high half.
        let mut rng = Lcg32::new(LcgVariant::Msvc, 1);
        let high_bits_seen = (0..256)
            .map(|_| rng.next_u32())
            .fold(0u32, |acc, w| acc | (w >> 16));
        assert!(
            high_bits_seen != 0,
            "Lcg32::Msvc next_u32 left the high 16 bits zero — packing regression"
        );
    }

    #[test]
    fn borland_next_u32_packs_raws_to_full_width() {
        let mut rng = Lcg32::new(LcgVariant::Borland, 1);
        let high_bits_seen = (0..256)
            .map(|_| rng.next_u32())
            .fold(0u32, |acc, w| acc | (w >> 16));
        assert!(
            high_bits_seen != 0,
            "Lcg32::Borland next_u32 left the high 16 bits zero — packing regression"
        );
    }
}
