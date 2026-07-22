//! SFC64 (Small Fast Counting) and JSF64 (Jenkins Small Fast) — 64-bit PRNGs.
//!
//! Both are simple chaotic generators with minimal state and excellent
//! statistical quality.  They pass BigCrush and PractRand; neither is
//! cryptographically secure.
//!
//! ## SFC64
//! A counter-assisted chaotic generator by Chris Doty-Humphrey (PractRand
//! author).  State: three 64-bit words plus an explicit counter.  The counter
//! guarantees the period is at least 2⁶⁴; in practice it exceeds 2¹⁹².
//!
//! ## JSF64
//! Bob Jenkins' "Small Fast" generator.  State: four 64-bit words, no
//! explicit counter.  Simpler than SFC64 but equally fast.
//!
//! # References
//! C. Doty-Humphrey, "PractRand" (SFC64 source), 2014.
//! <http://pracrand.sourceforge.net/>
//!
//! B. Jenkins, "A small noncryptographic PRNG", 2007.
//! <http://burtleburtle.net/bob/rand/smallprng.html>
//! [pubs/jenkins-2007-smallprng.html]
//!
//! # Author
//! Chris Doty-Humphrey (SFC64); Bob Jenkins (JSF64);
//! Darrell Long (Rust port).

use super::{OsRng, Rng};

// ── SFC64 ────────────────────────────────────────────────────────────────────

/// Small Fast Counting 64-bit generator.
///
/// Guaranteed period ≥ 2⁶⁴; typical period ≫ 2¹²⁸.
pub struct Sfc64 {
    a: u64,
    b: u64,
    c: u64,
    counter: u64,
}

impl Sfc64 {
    /// Construct from three 64-bit seeds.
    ///
    /// Runs 18 warm-up steps (per PractRand recommendation) to scatter the
    /// initial state away from any low-entropy seed.
    #[must_use]
    pub fn new(a: u64, b: u64, c: u64) -> Self {
        let mut rng = Self {
            a,
            b,
            c,
            counter: 1,
        };
        for _ in 0..18 {
            rng.step();
        }
        rng
    }

    /// Construct from 192 bits drawn from the operating system RNG.
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        Self::new(os.next_u64(), os.next_u64(), os.next_u64())
    }

    #[inline]
    fn step(&mut self) -> u64 {
        let tmp = self.a.wrapping_add(self.b).wrapping_add(self.counter);
        self.counter = self.counter.wrapping_add(1);
        self.a = self.b ^ (self.b >> 11);
        self.b = self.c.wrapping_add(self.c << 3);
        self.c = self.c.rotate_left(24).wrapping_add(tmp);
        tmp
    }
}

impl Default for Sfc64 {
    fn default() -> Self {
        Self::from_os_rng()
    }
}

impl Rng for Sfc64 {
    fn next_u32(&mut self) -> u32 {
        (self.step() >> 32) as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.step()
    }
}

// ── JSF64 ────────────────────────────────────────────────────────────────────

/// Jenkins Small Fast 64-bit generator.
///
/// Four-word state; period is at least 2⁶⁴ for all practical seeds.
pub struct Jsf64 {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

impl Jsf64 {
    /// Construct from a single 64-bit seed.
    ///
    /// Runs 20 warm-up steps per Jenkins' recommendation.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let mut rng = Self {
            a: 0xf1ea_5eed,
            b: seed,
            c: seed,
            d: seed,
        };
        for _ in 0..20 {
            rng.step();
        }
        rng
    }

    /// Construct from 64 bits drawn from the operating system RNG.
    #[must_use]
    pub fn from_os_rng() -> Self {
        Self::new(OsRng::new().next_u64())
    }

    #[inline]
    fn step(&mut self) -> u64 {
        let e = self.a.wrapping_sub(self.b.rotate_left(7));
        self.a = self.b ^ self.c.rotate_left(13);
        self.b = self.c.wrapping_add(self.d.rotate_left(37));
        self.c = self.d.wrapping_add(e);
        self.d = e.wrapping_add(self.a);
        self.d
    }
}

impl Default for Jsf64 {
    fn default() -> Self {
        Self::from_os_rng()
    }
}

impl Rng for Jsf64 {
    fn next_u32(&mut self) -> u32 {
        (self.step() >> 32) as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.step()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sfc64_advances() {
        let mut rng = Sfc64::new(1, 2, 3);
        let v0 = rng.next_u64();
        let v1 = rng.next_u64();
        assert_ne!(v0, v1);
    }

    #[test]
    fn sfc64_different_seeds_differ() {
        let mut a = Sfc64::new(1, 1, 1);
        let mut b = Sfc64::new(2, 2, 2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn jsf64_advances() {
        let mut rng = Jsf64::new(0xdeadbeef);
        let v0 = rng.next_u64();
        let v1 = rng.next_u64();
        assert_ne!(v0, v1);
    }

    #[test]
    fn jsf64_different_seeds_differ() {
        let mut a = Jsf64::new(1);
        let mut b = Jsf64::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn sfc64_next_u32_high_bits() {
        let mut a = Sfc64::new(7, 8, 9);
        let mut b = Sfc64::new(7, 8, 9);
        assert_eq!(a.next_u32(), (b.next_u64() >> 32) as u32);
    }

    // Known-answer test: first three outputs of Sfc64::new(1, 2, 3),
    // cross-checked against an independent Python replica of PractRand's
    // sfc64 (sfc.cpp: shifts 11/3/24, counter = 1, 18 warm-up rounds).
    #[test]
    fn sfc64_known_answer() {
        let mut rng = Sfc64::new(1, 2, 3);
        let expected: [u64; 3] = [
            0xbf36_b0b6_738f_81ed,
            0xcd52_7698_dd82_1546,
            0x8db8_6d5a_4db4_67e8,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u64(), e, "SFC64 KAT mismatch");
        }
    }

    // Known-answer test: first three outputs of Jsf64::new(0xdeadbeef),
    // cross-checked against an independent Python replica of Jenkins'
    // smallprng (64-bit rot 7/13/37 variant, a = 0xf1ea5eed, 20 warm-ups).
    #[test]
    fn jsf64_known_answer() {
        let mut rng = Jsf64::new(0xdead_beef);
        let expected: [u64; 3] = [
            0x2947_274c_9b14_f76b,
            0x10c1_0da7_6e5c_d72c,
            0x10ce_0695_c60a_16ed,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u64(), e, "JSF64 KAT mismatch");
        }
    }
}
