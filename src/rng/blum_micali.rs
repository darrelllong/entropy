//! Blum-Micali pseudorandom bit generator.
//!
//! The discrete-log based one-bit generator of Blum and Micali (1984):
//!
//! ```text
//! xᵢ₊₁ = g^xᵢ mod p
//! bitᵢ  = 1 if xᵢ₊₁ ≤ (p−1)/2, else 0
//! ```
//!
//! Security rests on the hardness of the discrete logarithm problem.
//!
//! For the test battery, 32 generator steps are packed into each 32-bit output
//! word.  The implementation requires p to be a prime < 2^32, g to be an
//! element of large multiplicative order, and x₀ ∈ (0, p).
//!
//! # Reference
//! M. Blum and S. Micali, "How to Generate Cryptographically Strong Sequences
//! of Pseudo-Random Bits", *SIAM Journal on Computing* 13(4), pp. 850–864, 1984.
//!
//! # Author
//! Darrell Long (UC Santa Cruz).

use super::{primes::is_probable_prime, Rng};

// ── Mersenne-prime fast arithmetic ───────────────────────────────────────────
//
// For a Mersenne prime p = 2^k − 1 with p < 2^32, any two values a, b < p
// satisfy a·b < p² < 2^64, so the full product fits in a u64.  Reduction then
// exploits the identity: x ≡ (x >> k) + (x & (p)) (mod p).
//
// This replaces the generic `mul_mod` (u128 path) with three u64 operations.

/// Modular multiplication for p = 2^k − 1, assuming a, b < p.
#[inline]
fn mersenne_mul(a: u64, b: u64, p: u64, k: u32) -> u64 {
    let t = a * b;              // fits in u64 because a*b < p^2 < 2^(2k) ≤ 2^64
    let r = (t >> k) + (t & p);
    if r >= p { r - p } else { r }
}

/// Modular exponentiation base^exp mod p for Mersenne prime p = 2^k − 1.
#[inline]
fn mersenne_pow(mut base: u64, mut exp: u64, p: u64, k: u32) -> u64 {
    let mut result = 1u64;
    base %= p;
    while exp > 0 {
        if exp & 1 == 1 {
            result = mersenne_mul(result, base, p, k);
        }
        base = mersenne_mul(base, base, p, k);
        exp >>= 1;
    }
    result
}

/// Return the Mersenne exponent k if p == 2^k − 1 and k ≤ 32, else None.
fn mersenne_k(p: u64) -> Option<u32> {
    // Check if p+1 is a power of two.
    let p1 = p + 1;
    if p1.is_power_of_two() {
        let k = p1.trailing_zeros();
        if k <= 32 { Some(k) } else { None }
    } else {
        None
    }
}

/// Blum-Micali pseudorandom bit generator.
pub struct BlumMicali {
    p:     u64,        // prime modulus
    g:     u64,        // generator (element of large order in Zp*)
    state: u64,        // current xᵢ
    k:     Option<u32>, // Mersenne exponent if p = 2^k−1, else None
}

impl BlumMicali {
    /// Construct a Blum-Micali generator.
    ///
    /// * `p`    — prime modulus (prime and < 2^32).
    /// * `g`    — generator, 1 < g < p.
    /// * `seed` — initial state x₀, 0 < seed < p.
    ///
    /// # Panics
    /// Panics if the parameters violate the stated preconditions.
    #[must_use]
    pub fn new(p: u64, g: u64, seed: u64) -> Self {
        assert!(p > 2 && p < (1u64 << 32), "p must be in (2, 2³²)");
        assert!(is_probable_prime(p), "p must be prime");
        assert!(g > 1 && g < p, "g must be in (1, p)");
        assert!(seed > 0 && seed < p, "seed must be in (0, p)");
        let k = mersenne_k(p);
        Self { p, g, state: seed, k }
    }

    /// Advance one step and return the output bit.
    pub fn next_bit(&mut self) -> u8 {
        self.state = match self.k {
            Some(k) => mersenne_pow(self.g, self.state, self.p, k),
            None    => super::primes::mod_pow(self.g, self.state, self.p),
        };
        u8::from(self.state <= (self.p - 1) / 2)
    }
}

impl Rng for BlumMicali {
    /// Pack 32 generator bits (MSB first) into a u32.
    fn next_u32(&mut self) -> u32 {
        let mut word = 0u32;
        for _ in 0..32 {
            word = (word << 1) | self.next_bit() as u32;
        }
        word
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Small reference: p = 23, g = 5, seed = 3.
    // x₁ = 5³ mod 23 = 125 mod 23 = 10;  10 ≤ 11 → bit = 1
    // x₂ = 5¹⁰ mod 23 = 9765625 mod 23 = 9;  9 ≤ 11 → bit = 1
    // x₃ = 5⁹ mod 23 = 1953125 mod 23 = 1;  1 ≤ 11 → bit = 1
    // (Matches the cryptography crate's own tests.)
    #[test]
    fn small_reference_first_3_bits() {
        let mut bm = BlumMicali::new(23, 5, 3);
        assert_eq!(bm.next_bit(), 1);
        assert_eq!(bm.next_bit(), 1);
        assert_eq!(bm.next_bit(), 1);
    }

    #[test]
    fn larger_prime_runs() {
        // p = 2147483647 (2³¹−1), g = 7, seed = 42
        let mut bm = BlumMicali::new(2_147_483_647, 7, 42);
        let first  = bm.next_u32();
        let second = bm.next_u32();
        assert_ne!(first, second);
    }
}
