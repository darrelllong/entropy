//! Deterministic seeding helpers shared across all auxiliary probe binaries.
//!
//! [`splitmix64`] is the Vigna 64-bit finalizer used here only to expand a
//! single-word seed into wider key material before constructing the RNG under
//! test.  It is **not** itself the generator being evaluated.
//!
//! [`seed_material`] converts a `u64` seed into an arbitrary-width byte array
//! using [`splitmix64`].  The XOR with `0xa076_1d64_78bd_642f` (the first
//! Weyl-sequence prime from wyhash, Wang Yi, 2019) ensures that seed = 0 does
//! not produce the all-zeros splitmix64 state.
//!
//! Seed derivation is part of experimental reproducibility: having one
//! definition, one comment, and one set of tests reduces the risk that a
//! future bug fix lands in some probe binaries but not others.
//!
//! ## Cipher test keys
//!
//! [`K16`], [`K32`], [`IV8`], and [`IV16`] are the fixed keys and IVs used to
//! construct cipher-based RNGs in the test harness.  They are defined here
//! once and imported wherever needed.
//!
//! **These constants are NOT suitable for any production use.**  Sequential
//! byte strings are present in every published test-vector corpus and would
//! immediately compromise any real cryptographic deployment.

/// One step of the Vigna splitmix64 mixer.
///
/// Advances `state` by one splitmix64 step and returns the mixed output word.
pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Derive `N` bytes of seed material from a single 64-bit seed.
///
/// Expands `seed` via repeated [`splitmix64`] calls, writing 8 bytes per
/// iteration until `N` bytes are filled (big-endian word order).  The XOR
/// with `0xa076_1d64_78bd_642f` (wyhash wyp0 prime) before the first call
/// ensures that seed = 0 yields a non-trivial initial state.
pub fn seed_material<const N: usize>(seed: u64) -> [u8; N] {
    let mut state = seed ^ 0xa076_1d64_78bd_642f;
    let mut out = [0u8; N];
    let mut pos = 0usize;
    while pos < N {
        let word = splitmix64(&mut state).to_be_bytes();
        let take = (N - pos).min(8);
        out[pos..pos + take].copy_from_slice(&word[..take]);
        pos += take;
    }
    out
}

/// Build an `N`-byte array whose elements are `0, 1, 2, …, N-1 (mod 256)`.
///
/// Used to construct fixed test keys for cipher-based RNGs so that the key
/// bytes appear only once in the codebase rather than being repeated at each
/// call site as an inline hex literal.
///
/// # Warning — NOT FOR PRODUCTION USE
///
/// Sequential byte strings `[0x00, 0x01, …, N-1]` are present in every
/// published test-vector corpus.  Any cipher initialised with these values in
/// a real deployment would be immediately broken.  Use a cryptographically
/// secure random source for all production key material.
pub const fn sequential_bytes<const N: usize>() -> [u8; N] {
    let mut out = [0u8; N];
    let mut i = 0;
    while i < N {
        out[i] = i as u8;
        i += 1;
    }
    out
}

// ── Cipher test keys ─────────────────────────────────────────────────────────
//
// Defined here once; imported by main.rs, pilot_rng.rs, and any future binary
// that instantiates cipher-based RNGs.  See the module-level warning above.

/// 128-bit test key: `[0x00, 0x01, …, 0x0f]`.  NOT FOR PRODUCTION USE.
pub const K16: [u8; 16] = sequential_bytes();
/// 256-bit test key: `[0x00, 0x01, …, 0x1f]`.  NOT FOR PRODUCTION USE.
pub const K32: [u8; 32] = sequential_bytes();
/// 64-bit test IV:  `[0x00, 0x01, …, 0x07]`.  NOT FOR PRODUCTION USE.
pub const IV8: [u8; 8] = sequential_bytes();
/// 128-bit test IV: `[0x00, 0x01, …, 0x0f]`.  NOT FOR PRODUCTION USE.
pub const IV16: [u8; 16] = sequential_bytes();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix64_nonzero_seed_zero() {
        // seed = 0 must not produce all-zeros output.
        let out: [u8; 8] = seed_material(0);
        assert!(out.iter().any(|&b| b != 0));
    }

    #[test]
    fn splitmix64_deterministic() {
        let mut s1 = 42u64;
        let mut s2 = 42u64;
        assert_eq!(splitmix64(&mut s1), splitmix64(&mut s2));
    }

    #[test]
    fn seed_material_nonzero_for_seed_zero() {
        let out: [u8; 32] = seed_material(0);
        assert!(out.iter().any(|&b| b != 0));
    }

    #[test]
    fn seed_material_deterministic() {
        let a: [u8; 16] = seed_material(99);
        let b: [u8; 16] = seed_material(99);
        assert_eq!(a, b);
    }

    #[test]
    fn sequential_bytes_correct() {
        let b: [u8; 4] = sequential_bytes();
        assert_eq!(b, [0x00, 0x01, 0x02, 0x03]);
        let b16: [u8; 16] = sequential_bytes();
        assert_eq!(b16[0], 0x00);
        assert_eq!(b16[15], 0x0f);
        // Verify wrapping at 256.
        let b257: [u8; 257] = sequential_bytes();
        assert_eq!(b257[255], 0xff);
        assert_eq!(b257[256], 0x00);
    }
}
