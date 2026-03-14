//! Linear Congruential Generators — several parameter sets, some notoriously weak.
//!
//! LCGs have the form  xₙ₊₁ = (a·xₙ + c) mod m.
//! The `bad` variant uses the parameters from the classic glibc `rand()`, which
//! famously fails the NIST spectral test and several DIEHARD tests.

use super::Rng;

/// Which parameter set to use.
#[derive(Debug, Clone, Copy)]
pub enum LcgVariant {
    /// glibc `rand()`: a = 1_103_515_245, c = 12_345, m = 2³¹.
    /// Returns bits 30..16 of the state — notoriously weak.
    GlibcRand,
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
}

impl Lcg32 {
    pub fn new(variant: LcgVariant, seed: u64) -> Self {
        match variant {
            LcgVariant::GlibcRand => Self {
                state: seed & 0x7FFF_FFFF,
                a: 1_103_515_245,
                c: 12_345,
                m: 1 << 31,
                shift: 0,
            },
            LcgVariant::Minstd => Self {
                state: if seed == 0 { 1 } else { seed % 2_147_483_647 },
                a: 16_807,
                c: 0,
                m: 2_147_483_647,
                shift: 0,
            },
            LcgVariant::Borland => Self {
                state: seed & 0xFFFF_FFFF,
                a: 22_695_477,
                c: 1,
                m: 1 << 32,
                shift: 0,
            },
            LcgVariant::Msvc => Self {
                state: seed & 0xFFFF_FFFF,
                a: 214_013,
                c: 2_531_011,
                m: 1 << 32,
                shift: 16,
            },
        }
    }

    /// Convenience: glibc rand with seed 1.
    pub fn glibc() -> Self {
        Self::new(LcgVariant::GlibcRand, 1)
    }

    /// Convenience: MINSTD with seed 1.
    pub fn minstd() -> Self {
        Self::new(LcgVariant::Minstd, 1)
    }
}

impl Rng for Lcg32 {
    fn next_u32(&mut self) -> u32 {
        self.state = (self.a.wrapping_mul(self.state).wrapping_add(self.c)) % self.m;
        (self.state >> self.shift) as u32
    }
}
