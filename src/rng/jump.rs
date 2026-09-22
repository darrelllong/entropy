//! Moving a linear generator forward by any number of steps, from a
//! polynomial derived here.
//!
//! A generator whose state update is linear over GF(2) — the xoshiro,
//! xoroshiro and xorshift families and the Mersenne Twister are, their output
//! scramblers are not — advances by one step through a matrix M over GF(2).
//! If m(x) annihilates M, m(M) = 0, then every power of M is a polynomial in
//! M of degree below deg m, and advancing J steps means applying
//!
//! x^J mod m(x),
//!
//! read as a polynomial in M: XOR the states reached at those step counts
//! whose coefficient is 1.  That is fewer than deg m ordinary steps, however
//! large J is, and J itself only costs a square-and-multiply.
//!
//! Nothing here is tabulated.  m(x) is recovered from the generator itself:
//! one bit of the state, read after each of 2·b steps for a state of b bits,
//! is a sequence satisfying the state's linear recurrence, and the
//! Berlekamp–Massey algorithm (J. L. Massey, "Shift-register synthesis and
//! BCH decoding," *IEEE Transactions on Information Theory* 15(1), 1969)
//! returns the shortest recurrence the sequence satisfies, whose reciprocal
//! annihilates M on the part of the state the recurrence reaches.  A state
//! can carry bits the recurrence never reads — the low bits of the Mersenne
//! Twister's first word are overwritten before anything depends on them —
//! and M sends such bits to zero in one step, so multiplying by x^(b − D) for
//! a recurrence of degree D annihilates them too and the product annihilates
//! all of M.  The derivation runs once per generator type and is cached.
//!
//! The consequence for a caller is a reproducible partition of one stream:
//! positioning k segments in from the same seed gives the k-th disjoint
//! segment, whatever order the workers run in.

/// A polynomial over GF(2), little-endian: bit i of `limbs[j]` is the
/// coefficient of x^(64j + i).
pub(crate) type Poly = Vec<u64>;

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

/// The 64 bits of `bits` from position `from`, low bit first; positions past
/// the end read as zero.
fn window(bits: &[u64], from: usize) -> u64 {
    let (limb, offset) = (from / LIMB_BITS, from % LIMB_BITS);
    let low = bits.get(limb).copied().unwrap_or(0) >> offset;
    if offset == 0 {
        low
    } else {
        low | bits.get(limb + 1).copied().unwrap_or(0) << (LIMB_BITS - offset)
    }
}

/// The shortest linear recurrence over GF(2) that `sequence` satisfies, as
/// the connection polynomial c(x) = 1 + c₁x + … + c_Lx^L (Massey, 1969).
///
/// The sequence must be at least twice as long as the recurrence for the
/// answer to be the true minimal polynomial.  Each step's discrepancy
/// Σᵢ cᵢ sₙ₋ᵢ is a parity of ANDed words: the sequence is held reversed, so
/// the terms of one discrepancy sit at consecutive positions, and `window`
/// reads them 64 at a time.
fn berlekamp_massey(sequence: &[bool]) -> Poly {
    let length = sequence.len();
    // reversed[j] = sequence[length − 1 − j], so sequence[n − i] is
    // reversed[(length − 1 − n) + i].
    let mut reversed: Poly = vec![0; length.div_ceil(LIMB_BITS)];
    for (j, &s) in sequence.iter().rev().enumerate() {
        if s {
            reversed[j / LIMB_BITS] |= 1 << (j % LIMB_BITS);
        }
    }
    let mut c: Poly = vec![1]; // current connection polynomial
    let mut b: Poly = vec![1]; // the one before the last length change
    let mut l = 0usize; // its degree, the recurrence's length
    let mut shift = 1usize; // steps since that change
    for n in 0..length {
        let from = length - 1 - n;
        let discrepancy = c.iter().enumerate().fold(0u32, |acc, (w, &cw)| {
            acc ^ (cw & window(&reversed, from + w * LIMB_BITS)).count_ones()
        }) & 1
            == 1;
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

/// The reciprocal x^`degree`·p(1/x) of a polynomial of that degree.
///
/// Berlekamp–Massey returns the connection polynomial c, with c₀ = 1, for
/// which sₙ = Σᵢ cᵢ sₙ₋ᵢ.  Reading that at n = m + L shows the coefficient of
/// s_{m+j} to be c_{L−j}, so the polynomial that annihilates the update
/// matrix on the states the sequence reaches is c with its coefficients
/// reversed.
fn reciprocal(poly: &[u64], degree: usize) -> Poly {
    let mut out: Poly = Vec::new();
    for i in 0..=degree {
        if bit(poly, i) {
            set_bit(&mut out, degree - i);
        }
    }
    out
}

/// A polynomial that annihilates the linear update `step` of an N-word state,
/// derived from the generator: bit 0 of the first word, read after each of
/// 2·64N steps from a state of all ones, satisfies the state's recurrence;
/// Berlekamp–Massey returns the shortest one, of degree D; its reciprocal is
/// multiplied by x^(64N − D) for the bits the recurrence never reads, which
/// one step sends to zero.
///
/// # Panics
/// Panics if the recurrence is longer than the state, which cannot happen
/// for a linear `step`, or empty.
pub(crate) fn annihilating_polynomial<const N: usize>(step: fn(&mut [u64; N])) -> Poly {
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
    let length = degree(&connection).expect("a recurrence of positive length");
    assert!(
        length <= bits_of_state,
        "the recurrence is longer than the state, so the update is not linear"
    );
    let mut annihilator: Poly = Vec::new();
    xor_shifted(
        &mut annihilator,
        &reciprocal(&connection, length),
        bits_of_state - length,
    );
    annihilator
}

/// x^(`steps`·2^`shift`) mod `modulus`, by square-and-multiply on `steps`
/// followed by `shift` squarings, so that a count of steps larger than a
/// `u128` — segment k of 2¹²⁸ steps each — is still one exponentiation.
pub(crate) fn power(modulus: &[u64], steps: u128, shift: u32) -> Poly {
    let modulus_degree = degree(modulus).expect("a nonzero modulus");
    let mut result: Poly = vec![1];
    if steps != 0 {
        for i in (0..u128::BITS - steps.leading_zeros()).rev() {
            result = mul_mod(&result, &result, modulus, modulus_degree);
            if steps >> i & 1 == 1 {
                // Multiply by x: shift up one and reduce.
                let mut shifted: Poly = Vec::new();
                xor_shifted(&mut shifted, &result, 1);
                reduce(&mut shifted, modulus, modulus_degree);
                result = shifted;
            }
        }
    }
    for _ in 0..shift {
        result = mul_mod(&result, &result, modulus, modulus_degree);
    }
    result
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

/// Advance a linear state by `steps`·2^`shift` steps: [`power`] then
/// [`apply`], the whole jump in one call.
pub(crate) fn advance_linear<const N: usize>(
    state: &mut [u64; N],
    modulus: &[u64],
    steps: u128,
    shift: u32,
    step: fn(&mut [u64; N]),
) {
    apply(state, &power(modulus, steps, shift), step);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A four-bit LFSR, whose recurrence Berlekamp–Massey recovers and whose
    /// jumps agree with stepping.  The whole pipeline in miniature, including
    /// the x^(64 − 4) factor for the sixty bits the LFSR never touches.
    #[test]
    fn a_small_generator_jumps_as_it_steps() {
        /// s ← (s >> 1) | (feedback << 3), the LFSR of x⁴ + x + 1.
        fn step(state: &mut [u64; 1]) {
            let s = state[0] & 0xf;
            let feedback = (s ^ (s >> 1)) & 1;
            state[0] = (s >> 1) | (feedback << 3);
        }
        let modulus = annihilating_polynomial::<1>(step);
        // x⁶⁰(x⁴ + x + 1): x⁶⁴ in the second limb, x⁶¹ + x⁶⁰ at the top of
        // the first.
        assert_eq!(modulus, vec![0b11 << 60, 1]);

        for steps in [0u128, 1, 5, 32, 1_000_003] {
            let mut stepped = [0b1001u64];
            for _ in 0..steps {
                step(&mut stepped);
            }
            let mut jumped = [0b1001u64];
            advance_linear(&mut jumped, &modulus, steps, 0, step);
            assert_eq!(jumped, stepped, "{steps} steps");
        }
        // Two shifted jumps compose: 3·2⁴ then 3·2⁴ is 96 steps.
        let mut twice = [0b1001u64];
        advance_linear(&mut twice, &modulus, 3, 4, step);
        advance_linear(&mut twice, &modulus, 3, 4, step);
        let mut once = [0b1001u64];
        advance_linear(&mut once, &modulus, 96, 0, step);
        assert_eq!(twice, once);
    }

    /// Berlekamp–Massey on sequences with known shortest recurrences, one
    /// long enough that the word-parallel discrepancy crosses limb boundaries.
    #[test]
    fn berlekamp_massey_finds_the_shortest_recurrence() {
        // s_n = s_{n-1} ^ s_{n-4}: connection polynomial 1 + x + x⁴.
        let mut bits = vec![true, false, false, false];
        for n in 4..300 {
            let next = bits[n - 1] ^ bits[n - 4];
            bits.push(next);
        }
        let c = berlekamp_massey(&bits);
        assert_eq!(degree(&c), Some(4));
        assert!(bit(&c, 0) && bit(&c, 1) && !bit(&c, 2) && !bit(&c, 3) && bit(&c, 4));
        // s_n = s_{n-3} ^ s_{n-70}: a recurrence past one limb.
        let mut long = vec![false; 70];
        long[0] = true;
        long[37] = true;
        for n in 70..400 {
            let next = long[n - 3] ^ long[n - 70];
            long.push(next);
        }
        let c = berlekamp_massey(&long);
        assert_eq!(degree(&c), Some(70));
        assert!(bit(&c, 0) && bit(&c, 3) && bit(&c, 70));
        assert_eq!(c.iter().map(|w| w.count_ones()).sum::<u32>(), 3);
    }

    /// The polynomial arithmetic: (x³ + x)(x² + 1) mod (x⁴ + x + 1) = x²,
    /// x⁰ is 1 whatever the shift, and x^(2·2³) = x¹⁶ = x mod (x⁴ + x + 1).
    #[test]
    fn polynomial_arithmetic_is_modular() {
        let m = vec![0b1_0011u64];
        let a = vec![0b1010u64];
        let b = vec![0b0101u64];
        assert_eq!(mul_mod(&a, &b, &m, 4), vec![0b100]);
        assert_eq!(power(&m, 0, 7), vec![1]);
        assert_eq!(power(&m, 2, 3), vec![0b10]);
    }
}
