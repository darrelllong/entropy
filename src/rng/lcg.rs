//! Linear Congruential Generators — several parameter sets, some notoriously weak.
//!
//! LCGs have the form `x_(n+1) = (a*x_n + c) mod m`.
//! These are parameterized mathematical generators, not necessarily faithful
//! libc APIs. Historical Unix libc wrappers live in [`super::c_stdlib`].

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
    Msvc,
}

/// A 32-bit Linear Congruential Generator.
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
}

impl Lcg32 {
    pub fn new(variant: LcgVariant, seed: u64) -> Self {
        match variant {
            LcgVariant::AnsiC => Self {
                state: seed & 0x7FFF_FFFF,
                a: 1_103_515_245,
                c: 12_345,
                m: 1 << 31,
                shift: 0,
                output_mask: u32::MAX,
            },
            LcgVariant::Minstd => Self {
                state: if seed == 0 { 1 } else { seed % 2_147_483_647 },
                a: 16_807,
                c: 0,
                m: 2_147_483_647,
                shift: 0,
                output_mask: u32::MAX,
            },
            LcgVariant::Borland => Self {
                state: seed & 0xFFFF_FFFF,
                a: 22_695_477,
                c: 1,
                m: 1 << 32,
                // Historical Borland rand() returns (state >> 16) & 0x7FFF (15 bits).
                shift: 16,
                output_mask: 0x7FFF,
            },
            LcgVariant::Msvc => Self {
                state: seed & 0xFFFF_FFFF,
                a: 214_013,
                c: 2_531_011,
                m: 1 << 32,
                shift: 16,
                output_mask: 0x7FFF,
            },
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
}

impl Rng for Lcg32 {
    fn next_u32(&mut self) -> u32 {
        self.state = (self.a.wrapping_mul(self.state).wrapping_add(self.c)) % self.m;
        ((self.state >> self.shift) as u32) & self.output_mask
    }
}

#[cfg(test)]
mod tests {
    use super::{Lcg32, LcgVariant};
    use crate::rng::Rng;

    #[test]
    fn msvc_variant_matches_known_seed_1_prefix() {
        let mut rng = Lcg32::new(LcgVariant::Msvc, 1);
        let got: Vec<u32> = (0..5).map(|_| rng.next_u32()).collect();
        assert_eq!(got, vec![41, 18_467, 6_334, 26_500, 19_169]);
    }
}
