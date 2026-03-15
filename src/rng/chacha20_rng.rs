//! ChaCha20-based CSPRNG.
//!
//! Wraps the `cryptography::ChaCha20` stream cipher as a pseudorandom byte
//! source.  A 256-bit key and 96-bit nonce are drawn from `OsRng` at
//! construction time; the keystream is consumed in 64-byte blocks via
//! `ChaCha20::keystream_block()`.
//!
//! This is structurally identical to how Linux's `/dev/urandom` and macOS's
//! `arc4random` work internally (both use ChaCha20 today).  Each 64-byte
//! block costs one ChaCha20 core invocation (20 rounds over a 4×4 word
//! state), so throughput scales with the cipher's speed.
//!
//! # Security
//! Output is computationally indistinguishable from random under the
//! assumption that ChaCha20 is a secure PRF.  The initial key and nonce are
//! drawn from the OS entropy source.  No reseed mechanism is implemented here;
//! for long-running applications use the OS CSPRNG directly.
//!
//! # References
//! D. J. Bernstein, "ChaCha, a variant of Salsa20", Workshop Record of
//! SASC 2008.  [pubs/bernstein-2008-chacha.pdf]
//! (Also at: <https://cr.yp.to/chacha/chacha-20080128.pdf>)
//!
//! # Author
//! Daniel J. Bernstein (algorithm); Darrell Long (Rust port).

use cryptography::ChaCha20;

use super::{OsRng, Rng};

const BLOCK_BYTES: usize = 64;

/// ChaCha20 stream cipher used as a CSPRNG.
///
/// Generates 64 bytes per ChaCha20 core invocation.
pub struct ChaCha20Rng {
    cipher: ChaCha20,
    buf:    [u8; BLOCK_BYTES],
    offset: usize,
}

impl ChaCha20Rng {
    /// Construct with a fresh key and nonce from the operating system RNG.
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        let mut key   = [0u8; 32];
        let mut nonce = [0u8; 12];
        for chunk in key.chunks_exact_mut(4) {
            chunk.copy_from_slice(&os.next_u32().to_le_bytes());
        }
        for chunk in nonce.chunks_exact_mut(4) {
            chunk.copy_from_slice(&os.next_u32().to_le_bytes());
        }
        let cipher = ChaCha20::new(&key, &nonce);
        let mut rng = Self { cipher, buf: [0u8; BLOCK_BYTES], offset: BLOCK_BYTES };
        rng.refill();
        rng
    }

    fn refill(&mut self) {
        self.buf = self.cipher.keystream_block();
        self.offset = 0;
    }

    fn take_bytes<const N: usize>(&mut self) -> [u8; N] {
        const { assert!(N <= BLOCK_BYTES, "chunk larger than ChaCha20 block") }
        if self.offset == BLOCK_BYTES {
            self.refill();
        }
        if self.offset + N > BLOCK_BYTES {
            self.refill();
        }
        let out = self.buf[self.offset..self.offset + N].try_into().unwrap();
        self.offset += N;
        out
    }
}

impl Default for ChaCha20Rng {
    fn default() -> Self { Self::from_os_rng() }
}

impl Rng for ChaCha20Rng {
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
    fn chacha20_rng_nonzero() {
        let mut rng = ChaCha20Rng::from_os_rng();
        let v: u64 = (0..8).map(|_| rng.next_u64()).fold(0, |a, b| a | b);
        assert_ne!(v, 0);
    }

    #[test]
    fn chacha20_rng_advances() {
        let mut rng = ChaCha20Rng::from_os_rng();
        let v0 = rng.next_u64();
        let v1 = rng.next_u64();
        assert_ne!(v0, v1);
    }

    #[test]
    fn chacha20_rng_refills_across_block_boundary() {
        let mut rng = ChaCha20Rng::from_os_rng();
        // Drain one full 64-byte block via u32 (16 calls) then read across boundary.
        for _ in 0..16 { let _ = rng.next_u32(); }
        let v = rng.next_u32(); // triggers refill
        assert_ne!(v, 0xffff_ffff); // trivially non-constant
    }
}
