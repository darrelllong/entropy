//! SHA3-512 hash-chain generator ("SpongeBob").
//!
//! The generator hashes arbitrary-length seed material into an initial
//! 512-bit state:
//!
//! `x_0 = SHA3-512(seed)`
//!
//! and then advances by re-hashing the previous state:
//!
//! `x_{i+1} = SHA3-512(x_i)`.
//!
//! Each state contributes up to 512 output bits.  The adapter exposes the
//! state as a sequential byte stream and refills by hashing the previous
//! 64-byte state once exhausted.  For uniform-width access (all `next_u32`
//! or all `next_u64`) all 512 bits are used; mixing widths at a refill
//! boundary silently discards up to 7 trailing bytes before refilling.
//!
//! On `aarch64` targets that expose FEAT_SHA3 (Apple Silicon and most modern
//! ARM cores), the underlying `cryptography::Sha3_512` call dispatches to
//! hardware EOR3/RAX1/BCAX `Keccak-f[1600]` intrinsics automatically.
//!
//! # References
//! * National Institute of Standards and Technology, "SHA-3 Standard:
//!   Permutation-Based Hash and Extendable-Output Functions,"
//!   *FIPS PUB 202*, August 2015. [pubs/NIST-FIPS-202.pdf]
//! * G. Bertoni, J. Daemen, M. Peeters, and G. Van Assche,
//!   "The Keccak Reference," Version 3.0, January 2011.
//!   <https://keccak.team/files/Keccak-reference-3.0.pdf>
//!   [Keccak permutation underlying SHA-3]
//!
//! # Author
//! NIST (SHA-3 specification); Darrell Long (Rust implementation).

use cryptography::Sha3_512;

use super::{OsRng, Rng};

const STATE_BYTES: usize = 64;

/// SHA3-512 hash-chain generator.
#[derive(Clone, Debug)]
pub struct SpongeBob {
    state: [u8; STATE_BYTES],
    offset: usize,
}

impl SpongeBob {
    /// Construct from arbitrary-length seed bytes.
    #[must_use]
    pub fn from_seed(seed: &[u8]) -> Self {
        Self {
            state: Sha3_512::digest(seed),
            offset: 0,
        }
    }

    /// Construct from 512 bits drawn from the operating system RNG.
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        let mut seed = [0u8; STATE_BYTES];
        for chunk in seed.chunks_exact_mut(4) {
            chunk.copy_from_slice(&os.next_u32().to_le_bytes());
        }
        Self::from_seed(&seed)
    }

    /// Fixed seed `00 01 … 3f` for reproducible benchmarks.
    #[must_use]
    pub fn with_test_seed() -> Self {
        let seed: [u8; STATE_BYTES] = core::array::from_fn(|i| i as u8);
        Self::from_seed(&seed)
    }

    fn refill(&mut self) {
        self.state = Sha3_512::digest(&self.state);
        self.offset = 0;
    }

    #[inline]
    fn take_bytes<const N: usize>(&mut self) -> [u8; N] {
        const { assert!(N <= STATE_BYTES, "chunk larger than SpongeBob state") }
        if self.offset + N > STATE_BYTES {
            self.refill();
        }
        let out = self.state[self.offset..self.offset + N].try_into().unwrap();
        self.offset += N;
        out
    }
}

impl Default for SpongeBob {
    fn default() -> Self {
        Self::from_os_rng()
    }
}

impl Rng for SpongeBob {
    fn next_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take_bytes::<4>())
    }
    fn next_u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take_bytes::<8>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_seed_replays_identically() {
        let seed = b"SpongeBob likes deterministic tests";
        let mut a = SpongeBob::from_seed(seed);
        let mut b = SpongeBob::from_seed(seed);
        for _ in 0..32 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn pool_is_sha3_512_chain() {
        let seed = b"entropy::SpongeBob";
        let x0 = Sha3_512::digest(seed);
        let mut rng = SpongeBob::from_seed(seed);
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
        let x0 = Sha3_512::digest(seed);
        let x1 = Sha3_512::digest(&x0);
        let mut rng = SpongeBob::from_seed(seed);
        for _ in 0..8 {
            let _ = rng.next_u64();
        }
        assert_eq!(
            rng.next_u64(),
            u64::from_le_bytes(x1[0..8].try_into().unwrap())
        );
    }

    #[test]
    fn u32_and_u64_share_byte_stream() {
        let mut a = SpongeBob::from_seed(b"stream");
        let mut b = SpongeBob::from_seed(b"stream");
        for _ in 0..64 {
            let lo = a.next_u32() as u64;
            let hi = a.next_u32() as u64;
            assert_eq!(hi << 32 | lo, b.next_u64());
        }
    }
}
