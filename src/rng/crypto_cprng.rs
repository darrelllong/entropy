//! Adapters for CPRNG / DRBG implementations provided by the sibling
//! `cryptography` crate.

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
    fn next_u32(&mut self) -> u32 {
        let mut out = [0u8; 4];
        self.inner.fill_bytes(&mut out);
        u32::from_be_bytes(out)
    }
}
