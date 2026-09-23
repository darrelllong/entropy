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
//! Bob Jenkins' "Small Fast" generator, the three-rotate (7, 13, 37) version
//! from the "64-bit variants" section of his page.  State: four 64-bit words,
//! no explicit counter.  Simpler than SFC64 but equally fast.
//!
//! # References
//! C. Doty-Humphrey, "PractRand", 2014, <http://pracrand.sourceforge.net/>:
//! SFC64 with shifts 11, 3 and 24, rotation 24, counter increment 1 and 18
//! warm-up rounds.
//!
//! B. Jenkins, "A small noncryptographic PRNG", 2007.
//! <http://burtleburtle.net/bob/rand/smallprng.html>
//! [pubs/jenkins-2007-smallprng.html]
//!
//! # Author
//! Chris Doty-Humphrey (SFC64); Bob Jenkins (JSF64).

use super::{streams::Streams, Rng, Seedable};
use crate::seed::splitmix64;

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
/// Four-word state.  Jenkins gives no guaranteed minimum period: for the
/// 32-bit version he expects an average cycle of about 2¹²⁶ results, and for
/// the 64-bit variants his page gives no figure.
pub struct Jsf64 {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

impl Jsf64 {
    /// The value Jenkins's `raninit` gives state word `a` before the seed
    /// fills `b`, `c` and `d`.  [pubs/jenkins-2007-smallprng.html]
    pub const INITIAL_A: u64 = 0xf1ea_5eed;

    /// Construct from a single 64-bit seed.
    ///
    /// Seeds as the page's `raninit` does
    /// (`a` = [`INITIAL_A`](Self::INITIAL_A), `b = c = d = seed`) and
    /// discards 20 outputs.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let mut rng = Self {
            a: Self::INITIAL_A,
            b: seed,
            c: seed,
            d: seed,
        };
        for _ in 0..20 {
            rng.step();
        }
        rng
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

/// A seed for stream `index` from a generator's state words: each word and
/// then the index is XORed into a SplitMix64 state and mixed, so that any
/// change to the state or the index changes every derived word.
fn derived_seed(words: &[u64], index: u64) -> impl FnMut() -> u64 {
    let mut mixer = 0u64;
    for &word in words.iter().chain(std::iter::once(&index)) {
        mixer ^= word;
        splitmix64(&mut mixer);
    }
    move || splitmix64(&mut mixer)
}

impl Streams for Sfc64 {
    /// A separate sequence, seeded by SplitMix64 from this generator's state
    /// and `index`: distinct for distinct indices, and reproducible, but not
    /// a segment of this sequence.  SFC64 is a chaotic map whose cycle
    /// structure is not known, so no jump exists and two streams cannot be
    /// proved disjoint; with cycles near 2¹²⁸ the chance that s streams of
    /// length n meet is about s²·n/2¹²⁸.
    fn stream(&self, index: u64) -> Self {
        let mut seed = derived_seed(&[self.a, self.b, self.c, self.counter], index);
        Self::new(seed(), seed(), seed())
    }
}

impl Streams for Jsf64 {
    /// A separate sequence, seeded by SplitMix64 from this generator's state
    /// and `index`, with the same standing as [`Sfc64`]'s streams: distinct
    /// and reproducible, not provably disjoint.
    fn stream(&self, index: u64) -> Self {
        let mut seed = derived_seed(&[self.a, self.b, self.c, self.d], index);
        Self::new(seed())
    }
}

#[cfg(test)]
mod tests {
    use super::super::streams::Streams;

    /// Streams are reproducible, distinct across indices and across seeds.
    #[test]
    fn streams_are_distinct_and_reproducible() {
        let base = Sfc64::new(1, 2, 3);
        let heads: Vec<u64> = (0..16).map(|k| base.stream(k).next_u64()).collect();
        assert_eq!(
            heads,
            (0..16)
                .map(|k| base.stream(k).next_u64())
                .collect::<Vec<_>>()
        );
        let mut sorted = heads.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), heads.len());
        assert_ne!(Sfc64::new(1, 2, 4).stream(0).next_u64(), heads[0]);
        let base = Jsf64::new(0xdead_beef);
        let a = base.stream(0).next_u64();
        assert_eq!(base.stream(0).next_u64(), a);
        assert_ne!(base.stream(1).next_u64(), a);
    }

    use super::*;

    /// Seed of the JSF64 tests, and of the known-answer vector below, which
    /// was checked at this seed against Jenkins's C.  `JSF64_PROBE_SEED` in
    /// `crate::seed`, the harness seed of `dump_rng` and `pilot_rng`, has the
    /// same value but stays a separate constant, so a harness change cannot
    /// move this vector.
    const JSF64_TEST_SEED: u64 = 0xdead_beef;

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
        let mut rng = Jsf64::new(JSF64_TEST_SEED);
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

    // Known-answer test: first three outputs of Sfc64::new(1, 2, 3) (shifts
    // 11/3/24, counter = 1, 18 warm-up rounds).
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

    // Known-answer test: first three outputs of Jsf64::new(JSF64_TEST_SEED)
    // (rotations 7/13/37, a = Jsf64::INITIAL_A, 20 warm-up rounds).
    #[test]
    fn jsf64_known_answer() {
        let mut rng = Jsf64::new(JSF64_TEST_SEED);
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
