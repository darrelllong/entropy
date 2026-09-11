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
