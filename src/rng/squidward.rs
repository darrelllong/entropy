//! SHA-256 hash-chain generator ("Squidward").
//!
//! Identical in design to SpongeBob but uses SHA-256 (256-bit / 32-byte state)
//! instead of SHA3-512.  On `aarch64` targets with FEAT_SHA2 (Apple Silicon
//! and most modern ARM cores) the inner compression step is offloaded to the
//! hardware `vsha256*` instructions via the `aarch64-alt` crate, making
//! Squidward substantially faster than SpongeBob on those machines.  On every
//! other target the pure-Rust `cryptography::Sha256` baseline is used
//! transparently.
//!
//! `x_0 = SHA-256(seed)`
//! `x_{i+1} = SHA-256(x_i)`
//!
//! Each 256-bit state block is exposed as a sequential byte stream.  For
//! uniform-width access (all `next_u32` or all `next_u64`) every bit of each
//! block is used.  Mixing widths at a refill boundary silently discards up to
//! 7 trailing bytes before refilling; see the same note in SpongeBob.

use cryptography::Sha256;

use super::{OsRng, Rng};

const STATE_BYTES: usize = 32;

/// SHA-256 hash-chain generator.
#[derive(Clone, Debug)]
pub struct Squidward {
    state: [u8; STATE_BYTES],
    offset: usize,
}

impl Squidward {
    /// Construct from optional variable-length seed bytes.
    ///
    /// When `seed` is `None`, a fresh 256-bit seed is drawn from `OsRng`.
    #[must_use]
    pub fn new(seed: Option<&[u8]>) -> Self {
        match seed {
            Some(s) => Self::from_seed(s),
            None => Self::from_os_rng(),
        }
    }

    /// Construct deterministically from caller-supplied seed bytes.
    #[must_use]
    pub fn from_seed(seed: &[u8]) -> Self {
        Self {
            state: sha256(seed),
            offset: 0,
        }
    }

    /// Construct from 256 bits drawn from the operating system RNG.
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        let mut seed = [0u8; STATE_BYTES];
        for chunk in seed.chunks_exact_mut(4) {
            chunk.copy_from_slice(&os.next_u32().to_le_bytes());
        }
        Self::from_seed(&seed)
    }

    /// Fixed 32-byte seed so benchmarks and test runs are reproducible.
    #[must_use]
    pub fn with_test_seed() -> Self {
        let seed: [u8; STATE_BYTES] = core::array::from_fn(|i| i as u8);
        Self::from_seed(&seed)
    }

    fn refill(&mut self) {
        self.state = sha256(&self.state);
        self.offset = 0;
    }

    fn take_bytes<const N: usize>(&mut self) -> [u8; N] {
        const { assert!(N <= STATE_BYTES, "chunk larger than Squidward state") }
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

impl Default for Squidward {
    fn default() -> Self {
        Self::from_os_rng()
    }
}

impl Rng for Squidward {
    fn next_u32(&mut self) -> u32 {
        u32::from_be_bytes(self.take_bytes::<4>())
    }

    fn next_u64(&mut self) -> u64 {
        u64::from_be_bytes(self.take_bytes::<8>())
    }
}

/// Compute SHA-256, preferring hardware acceleration on AArch64.
///
/// On `aarch64` targets the ARM FEAT_SHA2 path (`vsha256*` instructions) is
/// tried first via a runtime capability check; any other target falls through
/// to the pure-Rust `cryptography::Sha256` implementation.
#[inline]
fn sha256(data: &[u8]) -> [u8; 32] {
    #[cfg(target_arch = "aarch64")]
    if let Ok(digest) = aarch64_alt::sha256_armv8::Sha256Armv8::digest(data) {
        return digest;
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
    fn next_u64_reads_digest_chunks_in_order() {
        let seed = b"entropy::Squidward";
        let digest = sha256(seed);
        let mut rng = Squidward::from_seed(seed);
        for chunk in digest.chunks_exact(8) {
            let expected = u64::from_be_bytes(chunk.try_into().unwrap());
            assert_eq!(rng.next_u64(), expected);
        }
    }

    #[test]
    fn refill_hashes_previous_state() {
        let seed = b"hash the state again";
        let x0 = sha256(seed);
        let x1 = sha256(&x0);
        let mut rng = Squidward::from_seed(seed);
        // Exhaust the first 32-byte block (4 × u64).
        for _ in 0..4 {
            let _ = rng.next_u64();
        }
        // Next read must come from x1.
        assert_eq!(
            rng.next_u64(),
            u64::from_be_bytes(x1[0..8].try_into().unwrap())
        );
    }

    #[test]
    fn next_u32_splits_the_same_byte_stream() {
        let seed = b"u32 stream";
        let digest = sha256(seed);
        let mut rng = Squidward::from_seed(seed);
        for chunk in digest.chunks_exact(4) {
            let expected = u32::from_be_bytes(chunk.try_into().unwrap());
            assert_eq!(rng.next_u32(), expected);
        }
    }

    #[test]
    fn hw_and_sw_paths_agree() {
        // On aarch64 with FEAT_SHA2, verify the hardware path matches pure Rust.
        let data = b"consistency check across implementations";
        let sw = Sha256::digest(data);
        #[cfg(target_arch = "aarch64")]
        if let Ok(hw) = aarch64_alt::sha256_armv8::Sha256Armv8::digest(data) {
            assert_eq!(hw, sw);
        }
        // On non-aarch64 the test passes trivially (hw path unavailable).
        let _ = sw;
    }
}
