//! Blum-Blum-Shub pseudorandom number generator.
//!
//! The quadratic-residue generator of Blum, Blum, and Shub (1986):
//!
//! ```text
//! x₀    = seed² mod n,  n = p·q
//! xᵢ₊₁  = xᵢ² mod n,   output word = xᵢ as u32
//! ```
//!
//! where p ≡ q ≡ 3 (mod 4) are distinct primes (Blum integers).
//! Security rests on the hardness of integer factorisation.
//!
//! This implementation uses a u64 modulus (32-bit Blum primes) so that each
//! squaring is a single 128-bit multiply — fast enough for the test battery.
//! The lower 32 bits of each state word are used as output. Note: the tight
//! provably-secure per-step bit count for BBS is O(log₂ log₂ n) ≈ 6 bits for
//! a 64-bit modulus; the full 32-bit output exceeds the formal security proof
//! but is acceptable for a statistical test battery.
//!
//! # Reference
//! L. Blum, M. Blum, and M. Shub, "A Simple Unpredictable Pseudo-Random
//! Number Generator", *SIAM Journal on Computing* 15(2), pp. 364–383, 1986.
//!
//! # Author
//! Darrell Long (UC Santa Cruz).

use super::{primes::{is_probable_prime, mul_mod}, Rng};

/// Blum-Blum-Shub over a 64-bit modulus (32-bit Blum prime factors).
pub struct BlumBlumShub {
    n:     u64,
    state: u64,
}

impl BlumBlumShub {
    /// Construct a BBS generator.
    ///
    /// * `p`, `q` — distinct Blum primes (prime and ≡ 3 mod 4).
    /// * `seed`   — initial seed, 1 < seed < n, gcd(seed, n) = 1.
    ///
    /// # Panics
    /// Panics if the parameters do not satisfy the BBS preconditions.
    #[must_use]
    pub fn new(p: u32, q: u32, seed: u64) -> Self {
        let p64 = p as u64;
        let q64 = q as u64;
        assert!(p64 > 3 && q64 > 3, "p and q must be > 3");
        assert!(p != q, "p and q must be distinct");
        assert_eq!(p % 4, 3, "p must be ≡ 3 (mod 4)");
        assert_eq!(q % 4, 3, "q must be ≡ 3 (mod 4)");
        assert!(is_probable_prime(p64), "p must be prime");
        assert!(is_probable_prime(q64), "q must be prime");
        let n = p64.checked_mul(q64).expect("modulus overflow");
        assert!(seed > 1 && seed < n, "seed must be in (1, n)");
        // gcd check: seed is coprime to n iff not divisible by p or q.
        assert!(seed % p64 != 0 && seed % q64 != 0, "seed must be coprime to n");
        // Initial state: x₀ = seed² mod n.
        let state = mul_mod(seed, seed, n);
        Self { n, state }
    }

    /// Current state xᵢ.
    #[must_use]
    pub fn state(&self) -> u64 { self.state }
}

impl Rng for BlumBlumShub {
    /// Advance one BBS step and return the lower 32 bits of the new state.
    fn next_u32(&mut self) -> u32 {
        self.state = mul_mod(self.state, self.state, self.n);
        self.state as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Small-example check from the Wikipedia BBS article (n=11·23=253, seed=3).
    // x₀ = 3² mod 253 = 9.  Subsequent states: 81, 236, 82, 36, 31, 202, 64, 159, 1.
    #[test]
    fn small_example_states() {
        let mut bbs = BlumBlumShub::new(11, 23, 3);
        let expected = [81u64, 236, 36, 31, 202, 71, 234, 108, 26];
        for &exp in &expected {
            let _ = bbs.next_u32();
            assert_eq!(bbs.state(), exp);
        }
    }

    #[test]
    fn large_primes_produce_output() {
        // p = 2147483647 (2³¹−1, Mersenne prime, ≡ 3 mod 4)
        // q = 4294967291 (largest prime < 2³², ≡ 3 mod 4)
        let mut bbs = BlumBlumShub::new(2_147_483_647, 4_294_967_291, 1_234_567);
        // Just verify the generator runs and produces non-constant output.
        let first = bbs.next_u32();
        let second = bbs.next_u32();
        assert_ne!(first, second);
    }
}
