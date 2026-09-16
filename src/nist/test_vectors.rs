//! Worked-example inputs from SP 800-22 Rev. 1a, shared by the unit tests.

/// ε of the §2.1.8, §2.3.8, §2.6.8, §2.12.8 and §2.13.8 examples (n = 100).
pub(crate) const EPSILON_100: &str = concat!(
    "11001001000011111101101010100010001000010110100011",
    "00001000110100110001001100011001100010100010111000",
);

/// Parse a string of `0` and `1` characters into bits.
pub(crate) fn bits(s: &str) -> Vec<u8> {
    s.bytes()
        .map(|c| match c {
            b'0' => 0,
            b'1' => 1,
            _ => panic!("not a bit: {}", c as char),
        })
        .collect()
}

/// The first 10⁶ bits of the binary expansion of e: the input of the §2.5.8,
/// §2.8.8, §2.10.8, §2.11.8, §2.14.8 and §2.15.8 examples and of the second
/// table of Appendix B.
///
/// The digits start with the `10` of e's integer part, e = 10.1011011111…₂,
/// and are packed most significant bit first: digit j is bit 7 − (j mod 8) of
/// byte ⌊j/8⌋, so the file is 125 000 bytes.  Equivalently, the file is the
/// big-endian binary representation of ⌊e·2^(10⁶ − 2)⌋, which a test
/// recomputes.
const E_FIXTURE: &[u8] = include_bytes!("../../tests/data/e_1e6_bits.bin");

/// Number of bits in the e fixture.
pub(crate) const E_BITS: usize = 1_000_000;

/// The first `n` bits of the binary expansion of e, one bit per element.
pub(crate) fn e_bits(n: usize) -> Vec<u8> {
    assert!(n <= E_BITS, "the e fixture holds {E_BITS} bits");
    (0..n)
        .map(|j| (E_FIXTURE[j / 8] >> (7 - j % 8)) & 1)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{e_bits, E_BITS, E_FIXTURE};
    use rump::BigUint;

    /// Σ_{i=a+1..b} a!/i! as P/Q with Q = (a + 1)(a + 2)…b, by binary splitting:
    /// P(a, b) = P(a, m)·Q(m, b) + P(m, b) and Q(a, b) = Q(a, m)·Q(m, b).
    fn split(a: u64, b: u64) -> (BigUint, BigUint) {
        if b == a + 1 {
            return (BigUint::one(), BigUint::from_u64(b));
        }
        let m = (a + b) / 2;
        let (p_am, q_am) = split(a, m);
        let (p_mb, q_mb) = split(m, b);
        (p_am.mul(&q_mb).add(&p_mb), q_am.mul(&q_mb))
    }

    /// ⌊e·2^(k−2)⌋, whose k binary digits are the first k digits of e.
    ///
    /// e = 1 + Σ_{i≥1} 1/i!, cut at i = K with K! > 2^(k+64), so the omitted
    /// tail is below 2^−(k+63) and cannot change the floor unless e·2^(k−2) is
    /// that close to an integer.
    fn e_digits(k: usize) -> BigUint {
        let mut terms = 1u64;
        let mut log2_factorial = 0.0f64;
        while log2_factorial < (k + 64) as f64 {
            terms += 1;
            log2_factorial += (terms as f64).log2();
        }
        let (p, q) = split(0, terms);
        let mut numerator = p.add(&q);
        numerator.shl_bits(k - 2);
        numerator.div_rem(&q).0
    }

    /// SP 800-22 prints no digits of e, but §2.11.8 prints the symbol counts
    /// of these 10⁶ bits: 499 971 zeros and 500 029 ones and, reading the
    /// sequence cyclically, 250 116 of 00, 249 855 each of 01 and 10, and
    /// 250 174 of 11.
    #[test]
    fn e_fixture_has_the_counts_printed_in_section_2_11_8() {
        assert_eq!(E_FIXTURE.len() * 8, E_BITS);
        let e = e_bits(E_BITS);
        let ones = e.iter().filter(|&&b| b == 1).count();
        assert_eq!((E_BITS - ones, ones), (499_971, 500_029));
        let mut pairs = [0usize; 4];
        for (&a, &b) in e.iter().zip(e.iter().cycle().skip(1)) {
            pairs[usize::from((a << 1) | b)] += 1;
        }
        assert_eq!(pairs, [250_116, 249_855, 249_855, 250_174]);
    }

    /// The first 2¹² bits against e computed here, in every build.
    #[test]
    fn e_fixture_prefix_is_e() {
        let k = 1 << 12;
        let digits = e_digits(k);
        assert_eq!(digits.bits(), k);
        for (j, &b) in e_bits(k).iter().enumerate() {
            assert_eq!(b == 1, digits.bit(k - 1 - j), "bit {j} of e");
        }
    }

    /// The whole fixture is ⌊e·2^(10⁶ − 2)⌋.
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "10⁶-bit arithmetic; runs under cargo test --release"
    )]
    fn e_fixture_is_e() {
        let digits = e_digits(E_BITS);
        assert_eq!(digits.to_be_bytes_padded(E_FIXTURE.len()), E_FIXTURE);
    }
}
