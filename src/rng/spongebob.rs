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
//! Each state contributes up to 512 output bits. The `entropy` test harness is
//! word-oriented, so this adapter exposes the state as a sequential byte
//! stream and refills by hashing the previous 64-byte state once exhausted.
//! For uniform-width access (all `next_u32` or all `next_u64`) all 512 bits
//! are used; mixing widths at a refill boundary silently discards up to 7
//! trailing bytes of the current block before refilling.

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
    /// Construct from optional variable-length seed bytes.
    ///
    /// When `seed` is `None`, a fresh 512-bit seed is drawn from `OsRng`.
    #[must_use]
    pub fn new(seed: Option<&[u8]>) -> Self {
        match seed {
            Some(seed) => Self::from_seed(seed),
            None => Self::from_os_rng(),
        }
    }

    /// Construct deterministically from caller-supplied seed bytes.
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

    /// Fixed 64-byte seed so benchmarks and test runs are reproducible.
    #[must_use]
    pub fn with_test_seed() -> Self {
        let seed = core::array::from_fn::<u8, STATE_BYTES, _>(|i| i as u8);
        Self::from_seed(&seed)
    }

    fn refill(&mut self) {
        self.state = Sha3_512::digest(&self.state);
        self.offset = 0;
    }

    fn take_bytes<const N: usize>(&mut self) -> [u8; N] {
        const { assert!(N <= STATE_BYTES, "chunk larger than SpongeBob state") }
        if self.offset == STATE_BYTES {
            self.refill();
        }
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
        u32::from_be_bytes(self.take_bytes::<4>())
    }

    fn next_u64(&mut self) -> u64 {
        u64::from_be_bytes(self.take_bytes::<8>())
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
    fn next_u64_reads_digest_chunks_in_order() {
        let seed = b"entropy::SpongeBob";
        let digest = Sha3_512::digest(seed);
        let mut rng = SpongeBob::from_seed(seed);

        for chunk in digest.chunks_exact(8) {
            let expected = u64::from_be_bytes(chunk.try_into().unwrap());
            assert_eq!(rng.next_u64(), expected);
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
            u64::from_be_bytes(x1[0..8].try_into().unwrap())
        );
    }

    #[test]
    fn next_u32_splits_the_same_byte_stream() {
        let seed = b"u32 stream";
        let digest = Sha3_512::digest(seed);
        let mut rng = SpongeBob::from_seed(seed);

        for chunk in digest.chunks_exact(4) {
            let expected = u32::from_be_bytes(chunk.try_into().unwrap());
            assert_eq!(rng.next_u32(), expected);
        }
    }
}
