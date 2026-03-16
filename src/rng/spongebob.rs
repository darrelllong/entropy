//! SHAKE256 XOF pool generator ("SpongeBob").
//!
//! Each refill absorbs the current 64-byte key into a fresh SHAKE256 sponge,
//! then squeezes 16 384 bytes into the output pool followed by 64 bytes into
//! the next key (key erasure).  Every squeezed block is a distinct Keccak-f[1600]
//! application to the previous 1600-bit sponge state — successive permutations,
//! not copies.
//!
//! Cost per 16 KiB refill:
//! - 1 absorb permutation  (key → sponge)
//! - 121 squeeze permutations  (136 B/permutation × 121 = 16 456 B; use 16 448)
//! ────────────────────────────────────────────────────────────────────────────
//! 122 Keccak-f[1600] calls for 16 384 bytes of output
//! vs 257 SHA3-512 calls for the equivalent chain design (wider rate wins).
//!
//! Pool size is one Apple Silicon page (16 KiB).  On Linux aarch64 with 4 KiB
//! pages, change `POOL` to `4 * 1024`.

use cryptography::{Shake256, Xof, Sha3_512};

use super::{OsRng, Rng};

/// Output pool size: one ARM page (Apple Silicon default 16 KiB).
const POOL: usize = 16 * 1024;   // 16 384 bytes

/// Key size: one SHA3-512 block (512 bits).
const KEY: usize = 64;

/// SHAKE256 rate = 136 bytes/permutation.
/// 121 squeezes × 136 = 16 456 bytes ≥ POOL + KEY = 16 448.
/// Two separate squeeze calls let us fill pool then key without a temp buffer.

/// SHAKE256 pool generator.
#[derive(Clone)]
pub struct SpongeBob {
    pool: Box<[u8; POOL]>,
    pos:  usize,
    key:  [u8; KEY],
}

impl SpongeBob {
    /// Construct from arbitrary-length seed bytes.
    #[must_use]
    pub fn from_seed(seed: &[u8]) -> Self {
        let mut sb = Self {
            pool: Box::new([0u8; POOL]),
            pos:  POOL,   // triggers immediate refill
            // Hash the seed to a fixed KEY-sized key so arbitrarily long seeds
            // map cleanly into the sponge without rate-block complications.
            key: Sha3_512::digest(seed),
        };
        sb.refill();
        sb
    }

    /// Construct with optional seed; `None` draws from OsRng.
    #[must_use]
    pub fn new(seed: Option<&[u8]>) -> Self {
        match seed {
            Some(s) => Self::from_seed(s),
            None    => Self::from_os_rng(),
        }
    }

    /// Construct from 64 bytes drawn from the operating system RNG.
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        let mut seed = [0u8; KEY];
        for chunk in seed.chunks_exact_mut(4) {
            chunk.copy_from_slice(&os.next_u32().to_le_bytes());
        }
        Self::from_seed(&seed)
    }

    /// Fixed seed `00 01 … 3f` for reproducible benchmarks.
    #[must_use]
    pub fn with_test_seed() -> Self {
        let seed: [u8; KEY] = core::array::from_fn(|i| i as u8);
        Self::from_seed(&seed)
    }

    /// Absorb `self.key` into a fresh SHAKE256 sponge, squeeze POOL bytes
    /// directly into `self.pool`, then squeeze KEY bytes into `self.key`.
    /// SHAKE256 continues from where the previous squeeze left off, so the
    /// key is derived from the sponge state *after* all pool output — it
    /// never appears in the pool (forward secrecy).
    fn refill(&mut self) {
        let key = self.key;           // copy before overwriting
        let mut xof = Shake256::new();
        xof.update(&key);
        xof.squeeze(&mut *self.pool); // 16 384 bytes directly into pool (heap)
        xof.squeeze(&mut self.key);   // 64 bytes continuing the same sponge
        self.pos = 0;
    }

    #[inline]
    fn take_bytes<const N: usize>(&mut self) -> [u8; N] {
        if self.pos + N > POOL {
            self.refill();
        }
        let out = self.pool[self.pos..self.pos + N].try_into().unwrap();
        self.pos += N;
        out
    }
}

impl core::fmt::Debug for SpongeBob {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SpongeBob").field("pos", &self.pos).finish()
    }
}

impl Default for SpongeBob {
    fn default() -> Self { Self::from_os_rng() }
}

impl Rng for SpongeBob {
    fn next_u32(&mut self) -> u32 { u32::from_le_bytes(self.take_bytes::<4>()) }
    fn next_u64(&mut self) -> u64 { u64::from_le_bytes(self.take_bytes::<8>()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_seed_replays_identically() {
        let seed = b"SpongeBob likes deterministic tests";
        let mut a = SpongeBob::from_seed(seed);
        let mut b = SpongeBob::from_seed(seed);
        for _ in 0..256 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_produce_different_output() {
        let mut a = SpongeBob::from_seed(b"seed-A");
        let mut b = SpongeBob::from_seed(b"seed-B");
        let va: Vec<u32> = (0..32).map(|_| a.next_u32()).collect();
        let vb: Vec<u32> = (0..32).map(|_| b.next_u32()).collect();
        assert_ne!(va, vb);
    }

    #[test]
    fn pool_matches_shake256_squeeze() {
        use cryptography::Shake256;
        let seed = b"pool-check";
        let key = Sha3_512::digest(seed);

        let mut expected_pool = vec![0u8; POOL];
        let mut expected_key  = [0u8; KEY];
        let mut xof = Shake256::new();
        xof.update(&key);
        xof.squeeze(&mut expected_pool);
        xof.squeeze(&mut expected_key);

        let rng = SpongeBob::from_seed(seed);
        assert_eq!(&*rng.pool, expected_pool.as_slice());
        assert_eq!(rng.key,    expected_key);
    }

    #[test]
    fn key_is_not_in_pool() {
        // After refill, the stored key must not appear as any KEY-sized window.
        let rng = SpongeBob::from_seed(b"forward-secrecy");
        for chunk in rng.pool.chunks_exact(KEY) {
            assert_ne!(chunk, rng.key.as_slice());
        }
    }

    #[test]
    fn output_spans_pool_boundary() {
        let mut a = SpongeBob::from_seed(b"boundary");
        let mut b = SpongeBob::from_seed(b"boundary");
        for _ in 0..(POOL / 8) { a.next_u64(); }
        for _ in 0..(POOL / 8) { b.next_u64(); }
        // Both refilled; subsequent outputs must still agree.
        assert_eq!(a.next_u64(), b.next_u64());
        assert_eq!(a.next_u64(), b.next_u64());
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
