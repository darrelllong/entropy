//! WyRand — Wang Yi's ultra-fast 64-bit PRNG (wyhash final version 3, 2021).
//!
//! A single 64-bit counter advanced by a fixed Weyl-sequence increment, mixed
//! through a 128-bit multiply-xorfolded finaliser.  The multiply step provides
//! excellent avalanche; the generator passes BigCrush, PractRand > 8 TiB, and
//! NIST SP 800-22 at typical sample sizes.
//!
//! The constants below are the wyhash **final version 3** wyrand parameters —
//! the classic, most widely deployed variant.  Later releases (v4.2/v4.3)
//! switched to different constants (`0x2d358dccaa6c78a5` / `0x8bb84b93962eacc9`)
//! and are deliberately not used here.
//!
//! Not cryptographically secure; state is trivially invertible.
//!
//! # References
//! Wang Yi, "wyhash and wyrand", final version 3, 2021.
//! <https://github.com/wangyi-fudan/wyhash>
//!
//! # Author
//! Wang Yi (algorithm); Darrell Long (Rust port).

use super::{OsRng, Rng};

// Weyl-sequence constant (from wyhash final v3 source).
const WYRAND_INC: u64 = 0xa076_1d64_78bd_642f;
// Mix constant.
const WYRAND_MIX: u64 = 0xe703_7ed1_a0b4_28db;

/// Ultra-fast 64-bit PRNG based on a Weyl sequence and 128-bit multiply mix.
///
/// Period: 2⁶⁴.
pub struct WyRand {
    state: u64,
}

impl WyRand {
    /// Construct from an explicit 64-bit seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Construct from a 64-bit value drawn from the operating system RNG.
    #[must_use]
    pub fn from_os_rng() -> Self {
        Self::new(OsRng::new().next_u64())
    }

    #[inline]
    fn step(&mut self) -> u64 {
        self.state = self.state.wrapping_add(WYRAND_INC);
        wymix(self.state, self.state ^ WYRAND_MIX)
    }
}

/// The wyhash 128-bit multiply-xorfolded mixer.
#[inline(always)]
fn wymix(a: u64, b: u64) -> u64 {
    let m = (a as u128).wrapping_mul(b as u128);
    ((m >> 64) as u64) ^ (m as u64)
}

impl Default for WyRand {
    fn default() -> Self {
        Self::from_os_rng()
    }
}

impl Rng for WyRand {
    fn next_u32(&mut self) -> u32 {
        (self.step() >> 32) as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.step()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wyrand_nonzero_output() {
        let mut rng = WyRand::new(12345);
        let v: u64 = (0..8).map(|_| rng.next_u64()).fold(0, |a, b| a | b);
        assert_ne!(v, 0);
    }

    #[test]
    fn wyrand_different_seeds_differ() {
        let mut a = WyRand::new(1);
        let mut b = WyRand::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn wyrand_sequence_advances() {
        let mut rng = WyRand::new(0);
        let v0 = rng.next_u64();
        let v1 = rng.next_u64();
        assert_ne!(v0, v1);
    }

    #[test]
    fn wyrand_next_u32_high_bits() {
        let mut a = WyRand::new(42);
        let mut b = WyRand::new(42);
        assert_eq!(a.next_u32(), (b.next_u64() >> 32) as u32);
    }

    // Known-answer test: first three outputs cross-checked against an
    // independent Python replica of wyhash final v3's wyrand
    // (seed += 0xa0761d6478bd642f; wymix(seed, seed ^ 0xe7037ed1a0b428db)).
    #[test]
    fn wyrand_known_answer() {
        let mut rng = WyRand::new(12345);
        let expected: [u64; 3] = [
            0x3440_f9f4_6981_0c7b,
            0xc794_9c1f_4870_8594,
            0xda98_0e92_2b5f_67f8,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u64(), e, "WyRand final v3 KAT mismatch");
        }

        let mut rng0 = WyRand::new(0);
        let expected0: [u64; 3] = [
            0x111c_b3a7_8f59_a58e,
            0xceab_d938_ff4e_856d,
            0x61fb_5131_8f47_d2a4,
        ];
        for &e in &expected0 {
            assert_eq!(rng0.next_u64(), e, "WyRand final v3 KAT (seed 0) mismatch");
        }
    }
}
