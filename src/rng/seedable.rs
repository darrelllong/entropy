//! One seeding interface for the deterministic generators.
//!
//! [`Seedable::from_seed_bytes`] takes exactly [`Seedable::SEED_BYTES`] bytes
//! and reads each integer of the generator's state little-endian, in the
//! order its constructor takes them.  [`Seedable::seed_from_u64`] expands one
//! 64-bit value to those bytes with SplitMix64 (G. L. Steele, D. Lea and
//! C. H. Flood, "Fast splittable pseudorandom number generators," OOPSLA
//! 2014), so nearby seeds give unrelated states.  [`Seedable::from_os_rng`] draws
//! them from the operating system, and [`Seedable::try_from_os_rng`] reports
//! its failure instead of panicking.  The first two are value-stable.
//!
//! A byte string a generator cannot use (all zeros for xoshiro and
//! xoroshiro, whose all-zero state is a fixed point) is replaced by the
//! SplitMix64 expansion of 0, so every call returns a working generator.

use super::{
    os::{os_random, OS_FAILED},
    Jsf64, Mt19937, Pcg32, Pcg64, Sfc64, Xoroshiro128, Xoshiro256,
};
use crate::seed::splitmix64;
use std::io;

/// A generator constructible from a byte string of fixed length.
pub trait Seedable: Sized {
    /// Seed length in bytes.
    const SEED_BYTES: usize;

    /// The generator for `seed`.
    ///
    /// # Panics
    /// Panics if `seed` is not [`Self::SEED_BYTES`] long.
    fn from_seed_bytes(seed: &[u8]) -> Self;

    /// The generator for the SplitMix64 expansion of `seed`, eight
    /// little-endian bytes per step, truncated to [`Self::SEED_BYTES`].
    fn seed_from_u64(seed: u64) -> Self {
        let mut state = seed;
        let bytes: Vec<u8> = (0..Self::SEED_BYTES.div_ceil(8))
            .flat_map(|_| splitmix64(&mut state).to_le_bytes())
            .take(Self::SEED_BYTES)
            .collect();
        Self::from_seed_bytes(&bytes)
    }

    /// A generator seeded from the operating system.
    ///
    /// # Panics
    /// Panics if the operating system's entropy source fails;
    /// [`Self::try_from_os_rng`] returns the error instead.
    fn from_os_rng() -> Self {
        Self::try_from_os_rng().expect(OS_FAILED)
    }

    /// A generator seeded from the operating system.
    ///
    /// # Errors
    /// Any error from [`os_random`].
    fn try_from_os_rng() -> io::Result<Self> {
        #[cfg_attr(not(feature = "cryptography"), allow(unused_mut))]
        let mut bytes = vec![0u8; Self::SEED_BYTES];
        os_random(&mut bytes)?;
        let rng = Self::from_seed_bytes(&bytes);
        #[cfg(feature = "cryptography")]
        cryptography::zeroize_slice(&mut bytes);
        Ok(rng)
    }
}

/// Little-endian integers of `N` bytes from `seed`, which must be `count·N`
/// bytes long.
fn words<const N: usize>(seed: &[u8], count: usize) -> Vec<[u8; N]> {
    assert_eq!(seed.len(), count * N, "seed length");
    seed.chunks_exact(N)
        .map(|c| c.try_into().expect("chunk length"))
        .collect()
}

/// The SplitMix64 expansion of 0, for a byte string a generator cannot use.
fn replacement<const N: usize>() -> Vec<u8> {
    let mut state = 0;
    (0..N.div_ceil(8))
        .flat_map(|_| splitmix64(&mut state).to_le_bytes())
        .take(N)
        .collect()
}

impl Seedable for Pcg32 {
    const SEED_BYTES: usize = 16;
    fn from_seed_bytes(seed: &[u8]) -> Self {
        let w = words::<8>(seed, 2);
        Pcg32::new(u64::from_le_bytes(w[0]), u64::from_le_bytes(w[1]))
    }
}

impl Seedable for Pcg64 {
    const SEED_BYTES: usize = 32;
    fn from_seed_bytes(seed: &[u8]) -> Self {
        let w = words::<16>(seed, 2);
        Pcg64::new(u128::from_le_bytes(w[0]), u128::from_le_bytes(w[1]))
    }
}

impl Seedable for Xoshiro256 {
    const SEED_BYTES: usize = 32;
    fn from_seed_bytes(seed: &[u8]) -> Self {
        if seed.len() == 32 && seed.iter().all(|&b| b == 0) {
            return Self::from_seed_bytes(&replacement::<32>());
        }
        let w: Vec<u64> = words::<8>(seed, 4)
            .into_iter()
            .map(u64::from_le_bytes)
            .collect();
        Xoshiro256::new(w[0], w[1], w[2], w[3])
    }
}

impl Seedable for Xoroshiro128 {
    const SEED_BYTES: usize = 16;
    fn from_seed_bytes(seed: &[u8]) -> Self {
        if seed.len() == 16 && seed.iter().all(|&b| b == 0) {
            return Self::from_seed_bytes(&replacement::<16>());
        }
        let w: Vec<u64> = words::<8>(seed, 2)
            .into_iter()
            .map(u64::from_le_bytes)
            .collect();
        Xoroshiro128::new(w[0], w[1])
    }
}

impl Seedable for Sfc64 {
    const SEED_BYTES: usize = 24;
    fn from_seed_bytes(seed: &[u8]) -> Self {
        let w: Vec<u64> = words::<8>(seed, 3)
            .into_iter()
            .map(u64::from_le_bytes)
            .collect();
        Sfc64::new(w[0], w[1], w[2])
    }
}

impl Seedable for Jsf64 {
    const SEED_BYTES: usize = 8;
    fn from_seed_bytes(seed: &[u8]) -> Self {
        Jsf64::new(u64::from_le_bytes(words::<8>(seed, 1)[0]))
    }
}

impl Seedable for Mt19937 {
    const SEED_BYTES: usize = 4;
    fn from_seed_bytes(seed: &[u8]) -> Self {
        Mt19937::new(u32::from_le_bytes(words::<4>(seed, 1)[0]))
    }
}

#[cfg(feature = "cryptography")]
impl Seedable for super::ChaCha20Rng {
    /// A 32-byte key then a 12-byte nonce, starting at block 0.
    const SEED_BYTES: usize = 44;
    fn from_seed_bytes(seed: &[u8]) -> Self {
        assert_eq!(seed.len(), 44, "seed length");
        let key: [u8; 32] = seed[..32].try_into().expect("key");
        let nonce: [u8; 12] = seed[32..].try_into().expect("nonce");
        super::ChaCha20Rng::new(&key, &nonce, 0)
    }
}

#[cfg(feature = "cryptography")]
impl Seedable for super::FastKeyErasureRng {
    const SEED_BYTES: usize = 32;
    fn from_seed_bytes(seed: &[u8]) -> Self {
        super::FastKeyErasureRng::new(seed.try_into().expect("seed length"))
    }
}

#[cfg(test)]
mod tests {
    use super::Seedable;
    use crate::rng::{Jsf64, Mt19937, Pcg32, Pcg64, Rng, Sfc64, Xoroshiro128, Xoshiro256};

    /// Seed bytes are the constructor's integers, little-endian.
    #[test]
    fn seed_bytes_are_little_endian_constructor_arguments() {
        let bytes: Vec<u8> = (1..=32).collect();
        let w = |i: usize| u64::from_le_bytes(bytes[8 * i..8 * i + 8].try_into().unwrap());
        assert_eq!(
            Xoshiro256::from_seed_bytes(&bytes).next_u64(),
            Xoshiro256::new(w(0), w(1), w(2), w(3)).next_u64()
        );
        assert_eq!(
            Sfc64::from_seed_bytes(&bytes[..24]).next_u64(),
            Sfc64::new(w(0), w(1), w(2)).next_u64()
        );
        assert_eq!(
            Pcg32::from_seed_bytes(&bytes[..16]).next_u32(),
            Pcg32::new(w(0), w(1)).next_u32()
        );
        let big = |i: usize| u128::from_le_bytes(bytes[16 * i..16 * i + 16].try_into().unwrap());
        assert_eq!(
            Pcg64::from_seed_bytes(&bytes).next_u64(),
            Pcg64::new(big(0), big(1)).next_u64()
        );
        assert_eq!(
            Jsf64::from_seed_bytes(&bytes[..8]).next_u64(),
            Jsf64::new(w(0)).next_u64()
        );
        assert_eq!(
            Mt19937::from_seed_bytes(&bytes[..4]).next_u32(),
            Mt19937::new(u32::from_le_bytes([1, 2, 3, 4])).next_u32()
        );
    }

    /// All-zero seeds, which xoshiro and xoroshiro reject, still work.
    #[test]
    fn zero_seeds_give_working_generators() {
        assert_ne!(Xoshiro256::from_seed_bytes(&[0; 32]).next_u64(), 0);
        assert_ne!(Xoroshiro128::from_seed_bytes(&[0; 16]).next_u64(), 0);
    }

    /// seed_from_u64 is value-stable: SplitMix64 from 0 begins
    /// 0xe220a8397b1dcdaf (Steele, Lea and Flood's mixer).
    #[test]
    fn seed_from_u64_expands_with_splitmix64() {
        let mut state = 0;
        assert_eq!(crate::seed::splitmix64(&mut state), 0xe220_a839_7b1d_cdaf);
        let expected = Jsf64::new(0xe220_a839_7b1d_cdaf).next_u64();
        assert_eq!(Jsf64::seed_from_u64(0).next_u64(), expected);
        assert_ne!(
            Pcg64::seed_from_u64(1).next_u64(),
            Pcg64::seed_from_u64(2).next_u64()
        );
        assert!(Xoshiro256::try_from_os_rng().is_ok());
    }

    #[test]
    #[should_panic(expected = "seed length")]
    fn wrong_seed_lengths_panic() {
        let _ = Pcg32::from_seed_bytes(&[0; 15]);
    }
}
