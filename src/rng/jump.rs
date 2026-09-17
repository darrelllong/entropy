//! Jumping a linear generator forward by a power of two, from a polynomial
//! derived here.
//!
//! A generator whose state update is linear over GF(2) — xoshiro256 and
//! xoroshiro128 are, the scrambler that turns the state into output is not —
//! advances by one step through a matrix M over GF(2).  Let c(x) be the
//! characteristic polynomial of M.  By Cayley–Hamilton c(M) = 0, so every
//! power of M is a polynomial in M of degree below deg c, and advancing 2ᵏ
//! steps means applying
//!
//! x^(2ᵏ) mod c(x),
//!
//! read as a polynomial in M: XOR the states reached at those step counts
//! whose coefficient is 1.  That is fewer than deg c ordinary steps, however
//! large k is.
//!
//! Nothing here is tabulated.  c(x) is recovered from the generator itself:
//! one bit of the state, read after each of 2·deg c steps, is a sequence
//! satisfying that linear recurrence, and the Berlekamp–Massey algorithm (J.
//! L. Massey, "Shift-register synthesis and BCH decoding," *IEEE Transactions
//! on Information Theory* 15(1), 1969) returns the shortest recurrence the
//! sequence satisfies.  When that minimal polynomial has the full degree, it
//! is the characteristic polynomial.  The derivation runs once per generator
//! type and costs a few hundred thousand word operations; the result is
//! cached.
//!
//! The consequence for a caller is a reproducible partition of one stream:
//! jumping k times from the same seed gives the k-th disjoint segment,
//! whatever order the workers run in.

/// A polynomial over GF(2), little-endian: bit i of `limbs[j]` is the
/// coefficient of x^(64j + i).
type Poly = Vec<u64>;

/// Bits in a limb.
const LIMB_BITS: usize = u64::BITS as usize;

/// The coefficient of xⁱ.
fn bit(poly: &[u64], i: usize) -> bool {
    poly.get(i / LIMB_BITS)
        .is_some_and(|w| w >> (i % LIMB_BITS) & 1 == 1)
}

/// Set the coefficient of xⁱ to 1.
fn set_bit(poly: &mut Poly, i: usize) {
    let limb = i / LIMB_BITS;
    if poly.len() <= limb {
        poly.resize(limb + 1, 0);
    }
    poly[limb] |= 1 << (i % LIMB_BITS);
}

/// The degree, or `None` for the zero polynomial.
fn degree(poly: &[u64]) -> Option<usize> {
    poly.iter()
        .rposition(|&w| w != 0)
        .map(|j| j * LIMB_BITS + (LIMB_BITS - 1 - poly[j].leading_zeros() as usize))
}

/// `a ^= b << shift`, growing `a` as needed.
fn xor_shifted(a: &mut Poly, b: &[u64], shift: usize) {
    let Some(top) = degree(b) else { return };
    let needed = (top + shift) / LIMB_BITS + 1;
    if a.len() < needed {
        a.resize(needed, 0);
    }
    let (limbs, bits) = (shift / LIMB_BITS, shift % LIMB_BITS);
    for (j, &w) in b.iter().enumerate() {
        if w == 0 {
            continue;
        }
        a[j + limbs] ^= w << bits;
        if bits > 0 && j + limbs + 1 < a.len() {
            a[j + limbs + 1] ^= w >> (LIMB_BITS - bits);
        }
    }
}

/// `a mod m`, for a monic `m` of known degree: subtract (XOR) shifted copies
/// of `m` from the top down.
fn reduce(a: &mut Poly, m: &[u64], m_degree: usize) {
    while let Some(top) = degree(a) {
        if top < m_degree {
            break;
        }
        xor_shifted(a, m, top - m_degree);
    }
    while a.last() == Some(&0) {
        a.pop();
    }
}

/// `a·b mod m`, by shift and add.
fn mul_mod(a: &[u64], b: &[u64], m: &[u64], m_degree: usize) -> Poly {
    let mut product: Poly = Vec::new();
    for i in 0..=degree(b).unwrap_or(0) {
        if bit(b, i) {
            xor_shifted(&mut product, a, i);
        }
    }
    reduce(&mut product, m, m_degree);
    product
}

/// The shortest linear recurrence over GF(2) that `bits` satisfies, as the
/// connection polynomial c(x) = 1 + c₁x + … + c_Lx^L (Massey, 1969).
///
/// The sequence must be at least twice as long as the recurrence for the
/// answer to be the true minimal polynomial.
fn berlekamp_massey(bits: &[bool]) -> Poly {
    let mut c: Poly = vec![1]; // current connection polynomial
    let mut b: Poly = vec![1]; // the one before the last length change
    let mut l = 0usize; // its degree, the recurrence's length
    let mut shift = 1usize; // steps since that change
    for (n, &bit_n) in bits.iter().enumerate() {
        // Discrepancy: does the recurrence predict bits[n]?
        let mut discrepancy = bit_n;
        for i in 1..=l {
            if bit(&c, i) && bits[n - i] {
                discrepancy = !discrepancy;
            }
        }
        if !discrepancy {
            shift += 1;
            continue;
        }
        let previous = c.clone();
        xor_shifted(&mut c, &b, shift);
        if 2 * l <= n {
            l = n + 1 - l;
            b = previous;
            shift = 1;
        } else {
            shift += 1;
        }
    }
    c
}

/// The characteristic polynomial of a linear state update, derived from the
/// generator: bit 0 of the first state word, read after each step from a
/// state of all ones, satisfies the recurrence, and Berlekamp–Massey returns
/// the shortest one it satisfies.
///
/// # Panics
/// Panics if that minimal polynomial is shorter than the state, which means
/// the sampled bit does not exercise the whole state — a different generator
/// would need a different functional.
pub(crate) fn characteristic_polynomial<const N: usize>(step: fn(&mut [u64; N])) -> Poly {
    let bits_of_state = N * LIMB_BITS;
    let mut state = [u64::MAX; N];
    let sequence: Vec<bool> = (0..2 * bits_of_state)
        .map(|_| {
            let bit = state[0] & 1 == 1;
            step(&mut state);
            bit
        })
        .collect();
    let connection = berlekamp_massey(&sequence);
    let length = degree(&connection);
    assert_eq!(
        length,
        Some(bits_of_state),
        "the sampled bit does not span the state"
    );
    reciprocal(&connection, bits_of_state)
}

/// The reciprocal x^`degree`·p(1/x) of a polynomial of that degree.
///
/// Berlekamp–Massey returns the connection polynomial c, with c₀ = 1, for
/// which sₙ = Σᵢ cᵢ sₙ₋ᵢ.  Reading that at n = m + L shows the coefficient of
/// s_{m+j} to be c_{L−j}, so the polynomial that annihilates the update
/// matrix — the characteristic polynomial, once c has the full degree — is c
/// with its coefficients reversed.
fn reciprocal(poly: &[u64], degree: usize) -> Poly {
    let mut out: Poly = Vec::new();
    for i in 0..=degree {
        if bit(poly, i) {
            set_bit(&mut out, degree - i);
        }
    }
    out
}

/// x^(2^`exponent`) mod `characteristic`, by repeated squaring.
pub(crate) fn jump_polynomial(characteristic: &[u64], exponent: u32) -> Poly {
    let degree = degree(characteristic).expect("a nonzero characteristic polynomial");
    let mut power: Poly = Vec::new();
    set_bit(&mut power, 1); // x
    for _ in 0..exponent {
        power = mul_mod(&power, &power, characteristic, degree);
    }
    power
}

/// Apply a polynomial in the update matrix to `state`: XOR the states reached
/// at the step counts whose coefficient is 1.
pub(crate) fn apply<const N: usize>(state: &mut [u64; N], poly: &[u64], step: fn(&mut [u64; N])) {
    let mut jumped = [0u64; N];
    let mut walked = *state;
    for i in 0..=degree(poly).unwrap_or(0) {
        if bit(poly, i) {
            for (out, word) in jumped.iter_mut().zip(walked) {
                *out ^= word;
            }
        }
        step(&mut walked);
    }
    *state = jumped;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A four-bit LFSR, whose recurrence Berlekamp–Massey recovers and whose
    /// jump by 2⁵ agrees with 32 steps.  The whole pipeline in miniature.
    #[test]
    fn a_small_generator_jumps_as_it_steps() {
        /// s ← (s >> 1) | (feedback << 3), the LFSR of x⁴ + x + 1.
        fn step(state: &mut [u64; 1]) {
            let s = state[0] & 0xf;
            let feedback = (s ^ (s >> 1)) & 1;
            state[0] = (s >> 1) | (feedback << 3);
        }
        let bits: Vec<bool> = {
            let mut state = [0b1001u64];
            (0..16)
                .map(|_| {
                    let bit = state[0] & 1 == 1;
                    step(&mut state);
                    bit
                })
                .collect()
        };
        let connection = berlekamp_massey(&bits);
        assert_eq!(degree(&connection), Some(4), "the LFSR's own degree");
        let c = reciprocal(&connection, 4);

        let mut stepped = [0b1001u64];
        for _ in 0..1u64 << 5 {
            step(&mut stepped);
        }
        let mut jumped = [0b1001u64];
        apply(&mut jumped, &jump_polynomial(&c, 5), step);
        assert_eq!(jumped, stepped);
    }

    /// Berlekamp–Massey on a sequence with a known shortest recurrence.
    #[test]
    fn berlekamp_massey_finds_the_shortest_recurrence() {
        // s_n = s_{n-1} ^ s_{n-4}: connection polynomial 1 + x + x⁴.
        let mut bits = vec![true, false, false, false];
        for n in 4..64 {
            let next = bits[n - 1] ^ bits[n - 4];
            bits.push(next);
        }
        let c = berlekamp_massey(&bits);
        assert_eq!(degree(&c), Some(4));
        assert!(bit(&c, 0) && bit(&c, 1) && !bit(&c, 2) && !bit(&c, 3) && bit(&c, 4));
    }

    /// The polynomial arithmetic: (x³ + x)(x² + 1) = x⁵ + x, and reduction
    /// modulo x⁴ + x + 1 leaves x² + x + … , which stepping confirms below.
    #[test]
    fn polynomial_arithmetic_is_modular() {
        let m = vec![0b1_0011u64]; // x⁴ + x + 1
        let a = vec![0b1010u64]; // x³ + x
        let b = vec![0b0101u64]; // x² + 1
        let product = mul_mod(&a, &b, &m, 4);
        // x⁵ + x mod (x⁴ + x + 1) = x·(x⁴) + x = x(x + 1) + x = x²
        assert_eq!(product, vec![0b100]);
        assert_eq!(degree(&product), Some(2));
    }
}
