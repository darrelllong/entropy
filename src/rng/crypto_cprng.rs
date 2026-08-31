//! Adapters for CPRNG / DRBG implementations provided by the sibling
//! `cryptography` crate.
//!
//! `CryptoCtrDrbg` wraps `CtrDrbgAes256`, an AES-256-CTR DRBG conforming to
//! NIST SP 800-90A Rev. 1 §10.2.  The underlying DRBG implementation lives in
//! the `cryptography` crate and is not reproduced here.
//!
//! # References
//! * National Institute of Standards and Technology, "Recommendation for
//!   Random Number Generation Using Deterministic Random Bit Generators,"
//!   *NIST SP 800-90A Rev. 1*, June 2015, §10.2 (CTR_DRBG).
//!   [pubs/NIST-SP-800-90Ar1.pdf]
//! * National Institute of Standards and Technology, "Advanced Encryption
//!   Standard (AES)," *FIPS PUB 197 Update 1*, 2023.
//!   [pubs/NIST-FIPS-197.pdf]
//!
//! # Author
//! NIST (CTR_DRBG specification); Darrell Long (Rust adapter).

use crate::rng::Rng;
use cryptography::{Csprng, CtrDrbgAes256};

const CTR_DRBG_SEED_LEN: usize = 48;

/// Thin adapter exposing `cryptography::CtrDrbgAes256` through this crate's
/// word-oriented `Rng` trait.
pub struct CryptoCtrDrbg {
    inner: CtrDrbgAes256,
}

impl CryptoCtrDrbg {
    /// Deterministic constructor from exactly 48 bytes of seed material.
    #[must_use]
    pub fn new(seed_material: &[u8; CTR_DRBG_SEED_LEN]) -> Self {
        Self {
            inner: CtrDrbgAes256::new(seed_material),
        }
    }

    /// Fixed test seed so benchmark and battery runs are reproducible.
    #[must_use]
    pub fn with_test_seed() -> Self {
        let seed = core::array::from_fn::<u8, CTR_DRBG_SEED_LEN, _>(|i| i as u8);
        Self::new(&seed)
    }
}

impl Rng for CryptoCtrDrbg {
    /// Return the next 32-bit word from the DRBG byte stream.
    ///
    /// Note: words are decoded **big-endian** — a deliberate deviation from
    /// the little-endian convention of the crate's other byte-backed
    /// generators (see [`crate::rng::Rng`]).
    ///
    /// Each call draws exactly 4 bytes via `fill_bytes` with no buffering on
    /// this side, and `next_u64` is not overridden (it assembles two
    /// `next_u32` calls per the trait default).  The DRBG-invocation cadence
    /// per word therefore follows whatever internal buffering the sibling
    /// `cryptography` crate's `CtrDrbgAes256` performs.
    fn next_u32(&mut self) -> u32 {
        let mut out = [0u8; 4];
        self.inner.fill_bytes(&mut out);
        u32::from_be_bytes(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic-output regression pin for this adapter.  The CTR_DRBG
    /// algorithm itself is validated by the sibling `cryptography` crate; this
    /// guards the adapter's own contract — the fixed test seed, the
    /// big-endian word decoding, and the pinned crate version producing a
    /// stable stream.  A flipped endianness, a changed `fill_bytes` cadence, or
    /// a behavioural change in `CtrDrbgAes256` would all trip it.
    #[test]
    fn ctr_drbg_test_seed_output_is_stable() {
        let mut r = CryptoCtrDrbg::with_test_seed();
        let got: Vec<u32> = (0..6).map(|_| r.next_u32()).collect();
        assert_eq!(
            got,
            [
                0x0615_5023,
                0x7bad_a89b,
                0xd8ec_6ea3,
                0x9ed7_5d53,
                0xb370_2781,
                0xca89_6921,
            ]
        );
    }

    /// The adapter decodes big-endian: the first word's bytes must be the
    /// first four keystream bytes in big-endian order (documents the deliberate
    /// deviation from the crate's LE convention).
    #[test]
    fn ctr_drbg_decodes_big_endian() {
        let mut r = CryptoCtrDrbg::with_test_seed();
        let w = r.next_u32();
        assert_eq!(w.to_be_bytes()[0], 0x06, "high byte first ⇒ big-endian");
    }
}
