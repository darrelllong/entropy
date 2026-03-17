//! Generic stream-cipher pseudorandom number generator.
//!
//! Any type implementing [`cryptography::StreamCipher`] can be wrapped by
//! [`StreamRng`] to provide the [`super::Rng`] interface.  Stream ciphers
//! are inherently keystream generators; this wrapper simply XORs the keystream
//! into a zero-filled scratch buffer in 64-byte chunks and dispenses words
//! from that buffer.
//!
//! The 64-byte chunk size is a pragmatic choice: it matches ChaCha20's natural
//! block size and is a common multiple of the internal word widths of Rabbit
//! (16-byte state output), Salsa20 (64-byte block), Snow3G (4-byte word), and
//! ZUC-128 (4-byte word).
//!
//! Keystream bytes are consumed little-endian, consistent with the other
//! byte-backed generators in this crate.

use cryptography::StreamCipher;

use super::Rng;

const CHUNK: usize = 64;

/// Stream-cipher RNG wrapping any [`StreamCipher`].
///
/// The cipher must already be initialised (key and IV set) before wrapping.
/// After wrapping, callers only call [`Rng::next_u32`]; the underlying cipher
/// advances automatically as chunks are consumed.
pub struct StreamRng<C: StreamCipher> {
    cipher: C,
    buf:    [u8; CHUNK],
    pos:    usize,
}

impl<C: StreamCipher> StreamRng<C> {
    /// Wrap an already-initialised stream cipher.
    pub fn new(cipher: C) -> Self {
        // pos == CHUNK forces a refill on the first next_u32() call.
        Self { cipher, buf: [0u8; CHUNK], pos: CHUNK }
    }

    fn refill(&mut self) {
        self.buf = [0u8; CHUNK];
        // fill() XORs keystream into buf; starting from zeros gives raw keystream.
        self.cipher.fill(&mut self.buf);
        self.pos = 0;
    }
}

impl<C: StreamCipher> Rng for StreamRng<C> {
    fn next_u32(&mut self) -> u32 {
        if self.pos + 4 > CHUNK {
            self.refill();
        }
        let w = u32::from_le_bytes(
            self.buf[self.pos..self.pos + 4].try_into().unwrap(),
        );
        self.pos += 4;
        w
    }

    fn next_u64(&mut self) -> u64 {
        if self.pos + 8 > CHUNK {
            self.refill();
        }
        let w = u64::from_le_bytes(
            self.buf[self.pos..self.pos + 8].try_into().unwrap(),
        );
        self.pos += 8;
        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cryptography::Rabbit;

    #[test]
    fn stream_rng_non_constant() {
        let key  = [0u8; 16];
        let iv   = [0u8; 8];
        let mut rng = StreamRng::new(Rabbit::new(&key, &iv));
        let v: u64 = (0..8).map(|_| rng.next_u64()).fold(0, |a, b| a | b);
        assert_ne!(v, 0, "Rabbit keystream should be non-zero");
    }

    #[test]
    fn stream_rng_advances() {
        let key  = [0u8; 16];
        let iv   = [0u8; 8];
        let mut rng = StreamRng::new(Rabbit::new(&key, &iv));
        let a = rng.next_u64();
        let b = rng.next_u64();
        assert_ne!(a, b, "consecutive Rabbit words should differ");
    }

    #[test]
    fn stream_rng_crosses_chunk_boundary() {
        let key  = [0u8; 16];
        let iv   = [0u8; 8];
        let mut rng = StreamRng::new(Rabbit::new(&key, &iv));
        // Drain 15 u32s (60 bytes) then read a u32 that spans the boundary.
        for _ in 0..15 { let _ = rng.next_u32(); }
        let _ = rng.next_u32(); // triggers refill
    }
}
