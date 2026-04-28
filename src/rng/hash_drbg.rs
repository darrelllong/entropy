//! Hash_DRBG — NIST SP 800-90A Rev. 1 §10.1.1, instantiated with SHA-256.
//!
//! A deterministic random bit generator whose security rests on the
//! one-wayness of SHA-256 (modelled as a random oracle).  Unlike HMAC_DRBG,
//! no keying material is used; the state is a single 440-bit value V (the
//! NIST-specified `seedlen` for SHA-256) and a constant C derived from V.
//!
//! Hashgen produces output by hashing an incrementing counter concatenated
//! with V, delivering 32 bytes per SHA-256 call.  After each generate
//! request V and C are updated per §10.1.1.5.
//!
//! For uniform-width access (all `next_u32` or all `next_u64`) all 256 bits
//! per block are used; mixing widths at a refill boundary silently discards
//! up to 7 trailing bytes before refilling.
//!
//! # Reseed interval
//! SP 800-90A §10.1.1 Table 2 specifies a reseed interval of 2⁴⁸ generate
//! calls for 256-bit security strength.  `reseed_counter` is incremented once
//! per generate call (in `finalise_generate`) and checked at the top of each
//! `refill`; the implementation panics if the limit is exceeded.  The test
//! battery never approaches this bound.
//!
//! # Backtracking resistance
//! This implementation provides **no backtracking resistance**.  Compromising
//! the process memory reveals V and C, which fully determines all past and
//! future output since instantiation.  "Security rests on SHA-256 one-wayness"
//! means one-way chaining, not forward secrecy.  Correct for a test harness;
//! do not copy into applications requiring backtracking or prediction
//! resistance.
//!
//! # Seedlen rationale (SP 800-90A Table 2)
//! For SHA-256 (outlen=256 bits, security_strength=256 bits):
//!   seedlen = 440 bits = 55 bytes.
//!
//! # References
//! NIST SP 800-90A Rev. 1, "Recommendation for Random Number Generation
//! Using Deterministic Random Bit Generators", §10.1.1, 2015.
//! [pubs/NIST-SP-800-90Ar1.pdf]
//!
//! # Author
//! NIST (specification); Darrell Long (Rust implementation).

use cryptography::Sha256;

use super::{OsRng, Rng};

const SEEDLEN: usize = 55; // 440 bits — Table 2, SHA-256 row
const OUTLEN: usize = 32; // SHA-256 output = 256 bits = 32 bytes
// Number of Hashgen blocks per generate call.  SP 800-90A §10.1.1.4 says
// all blocks are produced from a local data variable before the §10.1.1.5
// update is applied; GENERATE_BLOCKS controls the batch size.
const GENERATE_BLOCKS: usize = 8;
const GENERATE_SIZE: usize = OUTLEN * GENERATE_BLOCKS; // 256 bytes

/// Hash_DRBG instantiated with SHA-256 per NIST SP 800-90A §10.1.1.
pub struct HashDrbg {
    v: [u8; SEEDLEN],
    c: [u8; SEEDLEN],
    reseed_counter: u64,
    buf: [u8; GENERATE_SIZE], // buffered Hashgen output (one full generate call)
    offset: usize,
}

impl HashDrbg {
    /// Instantiate from OS entropy (entropy_input=55 B, nonce=16 B).
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        let mut seed = [0u8; SEEDLEN + 16];
        for chunk in seed.chunks_exact_mut(4) {
            chunk.copy_from_slice(&os.next_u32().to_le_bytes());
        }
        // V = Hash_df(entropy_input || nonce, 440 bits)
        let v = hash_df(&seed, SEEDLEN);
        // C = Hash_df(0x00 || V, 440 bits)
        let c = {
            let mut input = [0u8; 1 + SEEDLEN];
            input[0] = 0x00;
            input[1..].copy_from_slice(&v);
            hash_df(&input, SEEDLEN)
        };
        Self {
            v,
            c,
            reseed_counter: 1,
            buf: [0u8; GENERATE_SIZE],
            offset: GENERATE_SIZE, // force refill on first use
        }
    }

    /// Produce GENERATE_BLOCKS Hashgen blocks (§10.1.1.4) into buf, then
    /// update V once per §10.1.1.5.  The local data counter is snapshotted
    /// from V and incremented only within this call, not stored in the struct.
    fn refill(&mut self) {
        // SP 800-90A §10.1.1.4 step 1: enforce reseed interval.
        assert!(
            self.reseed_counter < (1u64 << 48),
            "Hash_DRBG: reseed interval (2⁴⁸) exceeded (SP 800-90A §10.1.1 Table 2)"
        );
        let mut data = self.v;
        for i in 0..GENERATE_BLOCKS {
            let block = Sha256::digest(&data);
            self.buf[i * OUTLEN..(i + 1) * OUTLEN].copy_from_slice(&block);
            add1_mod2seedlen(&mut data);
        }
        self.offset = 0;
        self.finalise_generate();
    }

    /// §10.1.1.5: update V after a generate call.
    fn finalise_generate(&mut self) {
        // H = Hash(0x03 || V)
        let h = {
            let mut input = [0u8; 1 + SEEDLEN];
            input[0] = 0x03;
            input[1..].copy_from_slice(&self.v);
            Sha256::digest(&input)
        };

        // V = (V + H + C + reseed_counter) mod 2^seedlen
        // Accumulate into V in-place using big-endian arithmetic.
        add_bytes_mod(&mut self.v, &h);
        add_bytes_mod(&mut self.v, &self.c.clone());
        add_u64_mod(&mut self.v, self.reseed_counter);
        self.reseed_counter = self.reseed_counter.wrapping_add(1);
    }

    fn take_bytes<const N: usize>(&mut self) -> [u8; N] {
        const { assert!(N <= OUTLEN, "chunk larger than SHA-256 output") }
        if self.offset + N > GENERATE_SIZE {
            self.refill();
        }
        let out = self.buf[self.offset..self.offset + N].try_into().unwrap();
        self.offset += N;
        out
    }
}

// ── Helper: Hash_df (SP 800-90A §10.3.1) ─────────────────────────────────────

/// Derive `out_bytes` bytes from `input` using SHA-256 (seedlen=440 bits).
fn hash_df(input: &[u8], out_bytes: usize) -> [u8; SEEDLEN] {
    let bits = (out_bytes * 8) as u32;
    let num_blocks = out_bytes.div_ceil(OUTLEN);
    let mut temp = [0u8; OUTLEN * 2]; // enough for 2 SHA-256 blocks (covers 55 B)
    for i in 0..num_blocks {
        let counter = (i + 1) as u8;
        // Hash(counter || bits_as_4_be_bytes || input)
        let mut msg = Vec::with_capacity(5 + input.len());
        msg.push(counter);
        msg.extend_from_slice(&bits.to_be_bytes());
        msg.extend_from_slice(input);
        let block = Sha256::digest(&msg);
        let start = i * OUTLEN;
        let end = start + OUTLEN;
        temp[start..end].copy_from_slice(&block);
    }
    let mut out = [0u8; SEEDLEN];
    out.copy_from_slice(&temp[..SEEDLEN]);
    out
}

// ── Arithmetic helpers (big-endian mod 2^seedlen) ────────────────────────────

/// data = (data + 1) mod 2^seedlen (big-endian, in-place).
fn add1_mod2seedlen(data: &mut [u8; SEEDLEN]) {
    let mut carry = 1u16;
    for b in data.iter_mut().rev() {
        carry += *b as u16;
        *b = carry as u8;
        carry >>= 8;
    }
}

/// data = (data + addend[..]) mod 2^seedlen, addend is right-aligned.
fn add_bytes_mod(data: &mut [u8; SEEDLEN], addend: &[u8]) {
    let mut carry = 0u16;
    let data_len = data.len();
    let add_len = addend.len();
    for i in (0..data_len).rev() {
        let add_byte = if i + add_len >= data_len {
            addend[i + add_len - data_len]
        } else {
            0
        };
        carry += data[i] as u16 + add_byte as u16;
        data[i] = carry as u8;
        carry >>= 8;
    }
}

/// data = (data + n) mod 2^seedlen, n is a 64-bit counter.
fn add_u64_mod(data: &mut [u8; SEEDLEN], n: u64) {
    let bytes = n.to_be_bytes();
    add_bytes_mod(data, &bytes);
}

impl Default for HashDrbg {
    fn default() -> Self {
        Self::from_os_rng()
    }
}

impl Rng for HashDrbg {
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
    fn hash_drbg_nonzero() {
        let mut rng = HashDrbg::from_os_rng();
        let v: u64 = (0..8).map(|_| rng.next_u64()).fold(0, |a, b| a | b);
        assert_ne!(v, 0);
    }

    #[test]
    fn hash_drbg_advances() {
        let mut rng = HashDrbg::from_os_rng();
        let v0 = rng.next_u64();
        let v1 = rng.next_u64();
        assert_ne!(v0, v1);
    }

    #[test]
    fn hash_df_length() {
        // Hash_df output must be exactly SEEDLEN bytes.
        let out = hash_df(b"test input", SEEDLEN);
        assert_eq!(out.len(), SEEDLEN);
    }

    #[test]
    fn add1_wraps() {
        let mut v = [0xffu8; SEEDLEN];
        add1_mod2seedlen(&mut v);
        assert!(v.iter().all(|&b| b == 0)); // 2^440 ≡ 0
    }
}
