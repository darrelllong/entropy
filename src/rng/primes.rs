//! Minimal number-theory helpers for the toy CSPRNG wrappers.
//!
//! All arithmetic is bounded to values ≤ u64; intermediate products are
//! promoted to u128.  The Miller-Rabin test uses the first twelve small primes
//! as witnesses, which is deterministic and correct for all n < 3.3 × 10²⁴.
//!
//! Ported from Darrell Long's *cryptography* crate (`src/cprng/primes.rs`).

const WITNESSES: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

/// Modular multiplication: (a · b) mod m.  Requires a, b, m < 2^63.
#[must_use]
pub fn mul_mod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

/// Modular exponentiation: base^exp mod modulus.
#[must_use]
pub fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    let mut result = 1u64;
    base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 {
            result = mul_mod(result, base, modulus);
        }
        base = mul_mod(base, base, modulus);
        exp >>= 1;
    }
    result
}

/// Deterministic Miller-Rabin primality test for n < 2^63.
#[must_use]
pub fn is_probable_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n.is_multiple_of(2) {
        return false;
    }
    for &w in &WITNESSES {
        if n == w {
            return true;
        }
        if n.is_multiple_of(w) {
            return false;
        }
    }
    // Write n − 1 = d · 2^s with d odd.
    let mut d = n - 1;
    let mut s = 0u32;
    while d.is_multiple_of(2) {
        d >>= 1;
        s += 1;
    }

    'outer: for &a in &WITNESSES {
        if a >= n {
            continue;
        }
        let mut x = mod_pow(a, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }
        for _ in 1..s {
            x = mul_mod(x, x, n);
            if x == n - 1 {
                continue 'outer;
            }
        }
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_known_primes() {
        for p in [2u64, 3, 5, 7, 2147483647, 4294967291] {
            assert!(is_probable_prime(p), "{p} should be prime");
        }
    }

    #[test]
    fn rejects_composites() {
        for c in [0u64, 1, 4, 9, 15, 2147483646, 4294967295] {
            assert!(!is_probable_prime(c), "{c} should be composite");
        }
    }
}
