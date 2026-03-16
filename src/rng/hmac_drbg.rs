//! HMAC_DRBG — NIST SP 800-90A Rev. 1 §10.1.2, instantiated with HMAC-SHA-256.
//!
//! A deterministic random bit generator whose security rests on the
//! pseudorandomness of HMAC-SHA-256.  The state is a key K and a value V
//! (each 32 bytes); every `generate` call advances V via `V = HMAC(K, V)` and
//! then re-keys.
//!
//! This implementation:
//! - Seeds K and V from 384 bits of OS entropy (entropy_input=32 B,
//!   nonce=16 B) with an empty personalization string.
//! - Generates output in 32-byte blocks (one HMAC invocation per block).
//! - Implements the standard `HMAC_DRBG_Update` and `Generate` procedures
//!   without additional input or explicit reseed (suitable for the test battery).
//!
//! For uniform-width access (all `next_u32` or all `next_u64`) all 256 bits
//! per block are used; mixing widths at a refill boundary silently discards
//! up to 7 trailing bytes before refilling.
//!
//! # References
//! NIST SP 800-90A Rev. 1, "Recommendation for Random Number Generation
//! Using Deterministic Random Bit Generators", §10.1.2, 2015.
//! [pubs/NIST-SP-800-90Ar1.pdf]
//!
//! # Author
//! NIST (specification); Darrell Long (Rust implementation).

use cryptography::{Hmac, Sha256};

use super::{OsRng, Rng};

const OUT: usize = 32; // HMAC-SHA-256 output length (bytes)

/// HMAC_DRBG instantiated with HMAC-SHA-256 per NIST SP 800-90A §10.1.2.
pub struct HmacDrbg {
    k:      [u8; OUT],
    v:      [u8; OUT],
    buf:    [u8; OUT],
    offset: usize,
}

impl HmacDrbg {
    /// Instantiate from OS entropy (entropy_input=32 B, nonce=16 B).
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        let mut seed = [0u8; 48]; // 32-byte entropy_input + 16-byte nonce
        for chunk in seed.chunks_exact_mut(4) {
            chunk.copy_from_slice(&os.next_u32().to_le_bytes());
        }
        // Initial K=0x00…, V=0x01…, then Update(seed_material).
        let mut drbg = Self {
            k:      [0x00u8; OUT],
            v:      [0x01u8; OUT],
            buf:    [0u8; OUT],
            offset: OUT, // force refill on first use
        };
        drbg_update(&mut drbg.k, &mut drbg.v, Some(&seed));
        drbg
    }

    fn refill(&mut self) {
        // Generate step: advance V, buffer it, then re-key per §10.1.2.4.
        let mac = hmac_sha256(&self.k, &self.v);
        self.v.copy_from_slice(&mac);
        self.buf = self.v;
        drbg_update(&mut self.k, &mut self.v, None);
        self.offset = 0;
    }

    fn take_bytes<const N: usize>(&mut self) -> [u8; N] {
        const { assert!(N <= OUT, "chunk larger than HMAC-SHA-256 output") }
        if self.offset == OUT {
            self.refill();
        }
        if self.offset + N > OUT {
            self.refill();
        }
        let out = self.buf[self.offset..self.offset + N].try_into().unwrap();
        self.offset += N;
        out
    }
}

// ── SP 800-90A §10.1.2.2 HMAC_DRBG_Update ─────────────────────────────────

/// Stack scratch buffer capacity: V (32 B) + separator (1 B) + provided_data.
/// The SP 800-90A instantiation in this file passes at most 48 bytes of seed
/// material, giving a maximum message of 81 bytes; 128 B gives ample margin.
const SCRATCH: usize = 128;

fn drbg_update(k: &mut [u8; OUT], v: &mut [u8; OUT], provided_data: Option<&[u8]>) {
    let pd = provided_data.unwrap_or(&[]);
    debug_assert!(OUT + 1 + pd.len() <= SCRATCH, "drbg_update: provided_data too long");

    // K = HMAC(K, V || 0x00 [|| provided_data])
    let mut msg = [0u8; SCRATCH];
    msg[..OUT].copy_from_slice(v);
    msg[OUT] = 0x00;
    msg[OUT + 1..OUT + 1 + pd.len()].copy_from_slice(pd);
    let mac = hmac_sha256(k, &msg[..OUT + 1 + pd.len()]);
    k.copy_from_slice(&mac);

    // V = HMAC(K, V)
    let mac = hmac_sha256(k, v);
    v.copy_from_slice(&mac);

    if provided_data.is_some() {
        // K = HMAC(K, V || 0x01 || provided_data)
        msg[..OUT].copy_from_slice(v);
        msg[OUT] = 0x01;
        // pd slice and length unchanged — reuse msg[OUT+1..] already written
        let mac = hmac_sha256(k, &msg[..OUT + 1 + pd.len()]);
        k.copy_from_slice(&mac);

        // V = HMAC(K, V)
        let mac = hmac_sha256(k, v);
        v.copy_from_slice(&mac);
    }
}

#[inline]
fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; OUT] {
    let mac = Hmac::<Sha256>::compute(key, data);
    mac.try_into().expect("HMAC-SHA-256 output is always 32 bytes")
}

impl Default for HmacDrbg {
    fn default() -> Self { Self::from_os_rng() }
}

impl Rng for HmacDrbg {
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
    fn hmac_drbg_nonzero() {
        let mut rng = HmacDrbg::from_os_rng();
        let v: u64 = (0..8).map(|_| rng.next_u64()).fold(0, |a, b| a | b);
        assert_ne!(v, 0);
    }

    #[test]
    fn hmac_drbg_advances() {
        let mut rng = HmacDrbg::from_os_rng();
        let v0 = rng.next_u64();
        let v1 = rng.next_u64();
        assert_ne!(v0, v1);
    }

    #[test]
    fn hmac_drbg_update_changes_state() {
        // Two fresh instances from OS RNG should produce different streams.
        let mut a = HmacDrbg::from_os_rng();
        let mut b = HmacDrbg::from_os_rng();
        // With 256-bit entropy it's astronomically unlikely these collide.
        assert_ne!(a.next_u64(), b.next_u64());
    }
}
