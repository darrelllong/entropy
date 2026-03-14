//! Pure-Rust reimplementations of two macOS C standard-library generators.
//!
//! We replicate the algorithms precisely so we can test them without calling
//! into C (which would require `unsafe`, violating `#![forbid(unsafe_code)]`).
//!
//! Both generators are expected to **fail** many tests in this suite.
//!
//! # `CRand` — ISO C `rand()` as implemented in macOS / glibc
//!
//! The macOS libc uses a 31-bit LCG:
//!   state = (state × 1_103_515_245 + 12_345) mod 2³¹
//! and returns `state >> 16` (15 bits, RAND_MAX = 32 767).
//! We pack four calls' worth of low 8 bits to build a 32-bit word.
//!
//! # `Rand48` — POSIX `mrand48()` / `drand48()` family
//!
//! 48-bit LCG with the POSIX-mandated parameters:
//!   a = 0x5DEECE66D,  c = 0xB,  m = 2⁴⁸
//! `mrand48()` returns the upper 32 bits of the state as a signed integer;
//! here we return those 32 bits directly.

use super::Rng;

// ── CRand ─────────────────────────────────────────────────────────────────────

/// Pure-Rust implementation of macOS / glibc `rand()`.
///
/// The macOS C `rand()` has RAND_MAX = 32 767 (15 useful bits).
/// Expected to **fail** the NIST spectral test and most DIEHARD tests.
#[derive(Debug, Clone)]
pub struct CRand {
    state: u32,
}

impl CRand {
    pub fn new(seed: u32) -> Self {
        Self { state: seed & 0x7FFF_FFFF }
    }

    fn next_raw(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1_103_515_245).wrapping_add(12_345) & 0x7FFF_FFFF;
        self.state >> 16 // 15 bits, matching RAND_MAX = 32767
    }
}

impl Rng for CRand {
    fn next_u32(&mut self) -> u32 {
        // Pack four 8-bit slices from successive rand() calls.  Using the low
        // 8 bits of each call gives a somewhat better spread than using only
        // the top bits, but the generator is still linear and weak.
        let b0 = self.next_raw() & 0xFF;
        let b1 = self.next_raw() & 0xFF;
        let b2 = self.next_raw() & 0xFF;
        let b3 = self.next_raw() & 0xFF;
        (b3 << 24) | (b2 << 16) | (b1 << 8) | b0
    }
}

// ── Rand48 ────────────────────────────────────────────────────────────────────

/// Pure-Rust implementation of POSIX `mrand48()`.
///
/// 48-bit LCG: a = 0x5DEECE66D, c = 0xB, m = 2⁴⁸.
/// Returns the upper 32 bits of the 48-bit state (matching `mrand48`).
/// Better than `rand()` but still linear — expected to fail spectral and
/// serial tests.
#[derive(Debug, Clone)]
pub struct Rand48 {
    state: u64,
}

const RAND48_A: u64 = 0x5DEECE66D;
const RAND48_C: u64 = 0xB;
const RAND48_M: u64 = 1 << 48;

impl Rand48 {
    pub fn new(seed: u64) -> Self {
        // srand48 sets the high 32 bits; the low 16 bits are fixed at 0x330E.
        Self { state: (seed << 16) | 0x330E }
    }
}

impl Rng for Rand48 {
    fn next_u32(&mut self) -> u32 {
        self.state = (RAND48_A.wrapping_mul(self.state).wrapping_add(RAND48_C)) % RAND48_M;
        // mrand48 returns the upper 32 bits as a signed long; we return as u32.
        (self.state >> 16) as u32
    }
}
