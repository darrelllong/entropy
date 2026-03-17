//! SHA-256 hash-chain generator ("Squidward").
//!
//! `x_0 = SHA-256(seed)`, `x_{i+1} = SHA-256(x_i)`.
//!
//! Each 256-bit state is consumed as a sequential byte stream.  On `aarch64`
//! targets with FEAT_SHA2 (Apple Silicon and most modern ARM cores), every
//! SHA-256 call is dispatched to the hardware `vsha256*` NEON intrinsics via
//! the `aarch64-alt` crate; other targets fall back to `cryptography::Sha256`.
//!
//! Unlike SpongeBob (which uses a SHA3-512 chain and carries 512 bits of state
//! per step), SHA-256 has no XOF mode.  The state here is kept inline at
//! 32 bytes — the right size for the SHA-256 hardware path on aarch64.
//!
//! For uniform-width access (all `next_u32` or all `next_u64`) all 256 bits
//! are used; mixing widths at a refill boundary silently discards up to 7
//! trailing bytes before refilling.
//!
//! # References
//! * National Institute of Standards and Technology, "Secure Hash Standard
//!   (SHS)," *FIPS PUB 180-4*, August 2015.
//!   [pubs/NIST-FIPS-180-4.pdf]
//!
//! # Author
//! NIST (SHA-256 specification); Darrell Long (Rust implementation).

use cryptography::Sha256;

use super::{OsRng, Rng};

const BLOCK: usize = 32;

/// SHA-256 hash-chain generator.
#[derive(Clone, Debug)]
pub struct Squidward {
    state: [u8; BLOCK],
    offset: usize,
}

impl Squidward {
    /// Construct from arbitrary-length seed bytes.
    #[must_use]
    pub fn from_seed(seed: &[u8]) -> Self {
        Self {
            state: sha256(seed),
            offset: 0,
        }
    }

    /// Construct from 32 bytes drawn from the operating system RNG.
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        let mut seed = [0u8; BLOCK];
        for chunk in seed.chunks_exact_mut(4) {
            chunk.copy_from_slice(&os.next_u32().to_le_bytes());
        }
        Self::from_seed(&seed)
    }

    /// Fixed seed `00 01 … 1f` for reproducible benchmarks.
    #[must_use]
    pub fn with_test_seed() -> Self {
        let seed: [u8; BLOCK] = core::array::from_fn(|i| i as u8);
        Self::from_seed(&seed)
    }

    fn refill(&mut self) {
        self.state = sha256(&self.state);
        self.offset = 0;
    }

    #[inline]
    fn take_bytes<const N: usize>(&mut self) -> [u8; N] {
        const { assert!(N <= BLOCK, "chunk larger than Squidward state") }
        if self.offset + N > BLOCK {
            self.refill();
        }
        let out = self.state[self.offset..self.offset + N].try_into().unwrap();
        self.offset += N;
        out
    }
}

impl Default for Squidward {
    fn default() -> Self {
        Self::from_os_rng()
    }
}

impl Rng for Squidward {
    fn next_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take_bytes::<4>())
    }
    fn next_u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take_bytes::<8>())
    }
}

/// Compute SHA-256, preferring FEAT_SHA2 hardware on AArch64.
#[inline]
fn sha256(data: &[u8]) -> [u8; BLOCK] {
    #[cfg(target_arch = "aarch64")]
    if let Ok(d) = aarch64_alt::sha256_armv8::Sha256Armv8::digest(data) {
        return d;
    }
    Sha256::digest(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cryptography::Sha256;

    #[test]
    fn identical_seed_replays_identically() {
        let seed = b"Squidward likes deterministic tests";
        let mut a = Squidward::from_seed(seed);
        let mut b = Squidward::from_seed(seed);
        for _ in 0..32 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn pool_is_sha256_chain() {
        let seed = b"entropy::Squidward";
        let x0 = sha256(seed);
        let mut rng = Squidward::from_seed(seed);
        for chunk in x0.chunks_exact(8) {
            assert_eq!(
                rng.next_u64(),
                u64::from_le_bytes(chunk.try_into().unwrap())
            );
        }
    }

    #[test]
    fn refill_hashes_previous_state() {
        let seed = b"hash the state again";
        let x0 = sha256(seed);
        let x1 = sha256(&x0);
        let mut rng = Squidward::from_seed(seed);
        for _ in 0..4 {
            let _ = rng.next_u64();
        }
        assert_eq!(
            rng.next_u64(),
            u64::from_le_bytes(x1[0..8].try_into().unwrap())
        );
    }

    #[test]
    fn u32_and_u64_share_byte_stream() {
        let mut a = Squidward::from_seed(b"stream");
        let mut b = Squidward::from_seed(b"stream");
        for _ in 0..64 {
            let lo = a.next_u32() as u64;
            let hi = a.next_u32() as u64;
            assert_eq!(hi << 32 | lo, b.next_u64());
        }
    }

    #[test]
    fn hw_and_sw_paths_agree() {
        let data = b"consistency check across implementations";
        let sw = Sha256::digest(data);
        #[cfg(target_arch = "aarch64")]
        if let Ok(hw) = aarch64_alt::sha256_armv8::Sha256Armv8::digest(data) {
            assert_eq!(hw, sw);
        }
        let _ = sw;
    }
}
