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

use super::c_stdlib::PackedBits;
use super::Rng;

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
    ///
    /// The same generator as [`WindowsMsvcRand`](super::WindowsMsvcRand), the
    /// libc-wrapper view: for any seed the two give identical `next_raw` and
    /// `next_u32` streams, and their tests share one known-answer vector.
    Msvc,
}

/// A 32-bit Linear Congruential Generator.
///
/// `next_u32` packs successive raw outputs together for variants whose raw
/// output is 15 bits wide (Borland/MSVC `rand()`).  The 31-bit AnsiC and
/// MINSTD raws are deliberately NOT packed: they are zero-extended, leaving
/// bit 31 always 0, exactly as consuming the C API naïvely would — these
/// variants serve as negative controls with a documented [0, 2³¹) range
/// collapse.  Use [`Lcg32::next_raw`] for the bit-narrow C API value verbatim.
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
    /// Construct the chosen [`LcgVariant`] from a 64-bit seed; the seed is
    /// reduced to the variant's state range (masked or taken mod m).
    pub fn new(variant: LcgVariant, seed: u64) -> Self {
        let (state, a, c, m, shift, output_mask) = match variant {
            LcgVariant::AnsiC => (
                seed & 0x7FFF_FFFF,
                1_103_515_245,
                12_345,
                1 << 31,
                0,
                u32::MAX,
            ),
            LcgVariant::Minstd => (
                // Reduce BEFORE the zero guard: MINSTD has c = 0, so a state
                // of 0 (any seed ≡ 0 mod 2³¹−1, not just seed == 0) would be
                // a permanent fixed point.
                match seed % 2_147_483_647 {
                    0 => 1,
                    s => s,
                },
                16_807,
                0,
                2_147_483_647,
                0,
                u32::MAX,
            ),
            // Borland and MSVC return 15-bit values: `(state >> 16) & 0x7FFF`.
            LcgVariant::Borland => (seed & 0xFFFF_FFFF, 22_695_477, 1, 1u64 << 32, 16, 0x7FFF),
            LcgVariant::Msvc => (
                seed & 0xFFFF_FFFF,
                214_013,
                2_531_011,
                1u64 << 32,
                16,
                0x7FFF,
            ),
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
    use crate::rng::c_stdlib::MSVC_RAND_SEED_1_PREFIX;
    use crate::rng::{Rng, WindowsMsvcRand};

    #[test]
    fn msvc_raw_matches_known_seed_1_prefix() {
        // C-API faithfulness: 15-bit values from Microsoft Visual C `rand()`.
        let mut rng = Lcg32::new(LcgVariant::Msvc, 1);
        let got = MSVC_RAND_SEED_1_PREFIX.map(|_| rng.next_raw());
        assert_eq!(got, MSVC_RAND_SEED_1_PREFIX);
    }

    /// `LcgVariant::Msvc` and `WindowsMsvcRand` are one generator: identical
    /// raw values and identical packed words for several seeds.
    #[test]
    fn msvc_variant_and_windows_msvc_rand_agree() {
        for seed in [0u32, 1, 12_345, u32::MAX] {
            let mut lcg = Lcg32::new(LcgVariant::Msvc, u64::from(seed));
            let mut crt = WindowsMsvcRand::new(seed);
            for _ in 0..64 {
                assert_eq!(lcg.next_raw(), crt.next_raw(), "next_raw, seed {seed}");
            }
            let mut lcg = Lcg32::new(LcgVariant::Msvc, u64::from(seed));
            let mut crt = WindowsMsvcRand::new(seed);
            for _ in 0..64 {
                assert_eq!(lcg.next_u32(), crt.next_u32(), "next_u32, seed {seed}");
            }
        }
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

    /// AnsiC with seed 1: `x = 1103515245·x + 12345 mod 2³¹`, returned whole
    /// by both `next_raw` and `next_u32`.  Values from an independent replica
    /// of the recurrence.
    #[test]
    fn ansi_c_seed_1_kat() {
        let expected: [u32; 8] = [
            1_103_527_590,
            377_401_575,
            662_824_084,
            1_147_902_781,
            2_035_015_474,
            368_800_899,
            1_508_029_952,
            486_256_185,
        ];
        let mut raw = Lcg32::new(LcgVariant::AnsiC, 1);
        assert_eq!(expected.map(|_| raw.next_raw()), expected);
        let mut words = Lcg32::ansi_c();
        assert_eq!(expected.map(|_| words.next_u32()), expected);
    }

    /// Borland with seed 1: `x = 22695477·x + 1 mod 2³²` with raw output bits
    /// 30..16, and `next_u32` packing successive 15-bit raws least-significant
    /// first.  Values from an independent replica of the recurrence and of
    /// the packing rule.
    #[test]
    fn borland_seed_1_kat() {
        let raws: [u32; 8] = [346, 130, 10_982, 1_090, 11_656, 7_117, 17_595, 6_415];
        let words: [u32; 6] = [
            0x8041_015a,
            0x8088_4ab9,
            0xecde_6ad8,
            0xa432_1f12,
            0xcb3c_cb59,
            0xdf37_1bc8,
        ];
        let mut raw = Lcg32::new(LcgVariant::Borland, 1);
        assert_eq!(raws.map(|_| raw.next_raw()), raws);
        let mut packed = Lcg32::new(LcgVariant::Borland, 1);
        assert_eq!(words.map(|_| packed.next_u32()), words);
    }
}

#[cfg(test)]
mod minstd_seed_tests {
    use super::*;

    /// Regression: a seed that is a nonzero multiple of 2³¹−1 reduced to
    /// state 0, and with c = 0 the generator was stuck at zero forever.
    #[test]
    fn minstd_never_seeds_to_the_zero_fixed_point() {
        for seed in [0u64, 2_147_483_647, 2 * 2_147_483_647] {
            let mut rng = Lcg32::new(LcgVariant::Minstd, seed);
            let a = rng.next_u32();
            let b = rng.next_u32();
            assert!(a != 0 || b != 0, "stuck at zero for seed {seed}");
        }
        // Known first step from state 1: 16807.
        let mut rng = Lcg32::new(LcgVariant::Minstd, 2_147_483_647);
        assert_eq!(rng.next_u32(), 16_807);
    }
}
