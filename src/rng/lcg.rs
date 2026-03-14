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
        (self.state >> self.shift) as u32
    }
}
