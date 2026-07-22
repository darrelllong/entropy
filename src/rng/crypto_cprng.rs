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
