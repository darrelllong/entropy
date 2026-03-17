//! Deterministic seeding helpers shared across all auxiliary probe binaries.
//!
//! [`splitmix64`] is the Vigna 64-bit finalizer used here only to expand a
//! single-word seed into wider key material before constructing the RNG under
//! test.  It is **not** itself the generator being evaluated.
//!
//! [`seed_material`] converts a `u64` seed into an arbitrary-width byte array
//! using [`splitmix64`].  The XOR with `0xa076_1d64_78bd_642f` (the bit-
//! reversal of Knuth's FKS golden-ratio constant) ensures that seed = 0 does
//! not produce the all-zeros splitmix64 state.
//!
//! Seed derivation is part of experimental reproducibility: having one
//! definition, one comment, and one set of tests reduces the risk that a
//! future bug fix lands in some probe binaries but not others.

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
/// with `0xa076_1d64_78bd_642f` before the first call ensures that seed = 0
/// yields a non-trivial initial state.
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
pub const fn sequential_bytes<const N: usize>() -> [u8; N] {
    let mut out = [0u8; N];
    let mut i = 0;
    while i < N {
        out[i] = (i & 0xff) as u8;
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix64_nonzero_seed_zero() {
        // seed = 0 must not produce all-zeros output.
        let mut s = 0u64 ^ 0xa076_1d64_78bd_642f;
        assert_ne!(splitmix64(&mut s), 0);
    }

    #[test]
    fn splitmix64_deterministic() {
        let mut s1 = 42u64;
        let mut s2 = 42u64;
        assert_eq!(splitmix64(&mut s1), splitmix64(&mut s2));
    }

    #[test]
    fn seed_material_length() {
        let out: [u8; 55] = seed_material(12345);
        assert_eq!(out.len(), 55);
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
}
