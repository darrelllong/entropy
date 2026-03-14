//! NIST SP 800-22 §2.9 — Maurer's "Universal Statistical" Test.
//!
//! Tests compressibility: if the sequence is compressible it is not random.
//! Builds a table of the most recent position of each L-bit pattern, then
//! sums the log₂ of gaps between recurrences.
//!
//! Recommended defaults (Table 5, SP 800-22 §2.9.7):
//!   L = 7, Q = 1280, for n ≥ 387 840.
//!   L = 15 for n ≥ 1 900 000 gives higher power.

use crate::{math::erfc, result::TestResult};
use std::f64::consts::SQRT_2;

/// Expected value μ and variance σ² of the test statistic for each L.
///
/// Table 3 from SP 800-22 §2.9.7.  Index is L − 1.
const EXPECTED: [(f64, f64); 16] = [
    (0.0,       0.0    ), // L=0 unused
    (0.7326495, 0.690), // L=1
    (1.5374383, 1.338), // L=2
    (2.4016068, 1.901), // L=3
    (3.3112247, 2.358), // L=4
    (4.2534266, 2.705), // L=5
    (5.2177052, 2.954), // L=6
    (6.1962507, 3.125), // L=7
    (7.1836656, 3.238), // L=8
    (8.1764248, 3.311), // L=9
    (9.1723243, 3.356), // L=10
    (10.170032, 3.384), // L=11
    (11.168765, 3.401), // L=12
    (12.168070, 3.410), // L=13
    (13.167693, 3.416), // L=14
    (14.167488, 3.419), // L=15
];

/// Choose L automatically based on n.
fn choose_l(n: usize) -> usize {
    // From Table 2 in SP 800-22 §2.9.7 and the NIST reference C implementation.
    // These thresholds are n_min = (Q + K) * L = 10*2^L * L + 1000*L.
    match n {
        n if n >= 1_059_061_760 => 16,
        n if n >= 496_435_200   => 15,
        n if n >= 231_669_760   => 14,
        n if n >= 107_560_960   => 13,
        n if n >= 49_643_520    => 12,
        n if n >= 22_753_280    => 11,
        n if n >= 10_342_400    => 10,
        n if n >= 4_654_080     => 9,
        n if n >= 2_068_480     => 8,
        n if n >= 904_960       => 7,
        n if n >= 387_840       => 6,
        n if n >= 165_120       => 5,
        _ => 0, // too small
    }
}

/// Run Maurer's universal statistical test.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.9.
pub fn universal(bits: &[u8]) -> TestResult {
    let n = bits.len();
    let l = choose_l(n);
    if l == 0 {
        return TestResult::insufficient("nist::universal", "n too small (need ≥ 2560)");
    }

    let q = 10 * (1usize << l); // initialisation blocks
    let k = n / l - q;          // test blocks

    if k == 0 {
        return TestResult::insufficient("nist::universal", "not enough blocks after init");
    }

    // Build the initialisation table: last occurrence of each L-bit pattern.
    let table_size = 1usize << l;
    let mut table = vec![0usize; table_size];

    for i in 0..q {
        let pattern = bits_to_index(&bits[i * l..(i + 1) * l]);
        table[pattern] = i + 1;
    }

    // Accumulate sum of log₂(gap) over test blocks.
    let mut sum = 0.0f64;
    for i in q..q + k {
        let pattern = bits_to_index(&bits[i * l..(i + 1) * l]);
        let gap = i + 1 - table[pattern];
        sum += (gap as f64).log2();
        table[pattern] = i + 1;
    }

    let f_n = sum / k as f64;
    let (mu, sigma2) = EXPECTED[l];
    // Variance correction factor from SP 800-22 §2.9.4.
    let c = 0.7 - 0.8 / l as f64 + (4.0 + 32.0 / l as f64) * (q as f64).powf(-3.0 / l as f64);
    let sigma = c * (sigma2 / k as f64).sqrt();

    let p_value = erfc((f_n - mu).abs() / (sigma * SQRT_2));

    TestResult::with_note(
        "nist::universal",
        p_value,
        format!("n={n}, L={l}, Q={q}, K={k}, f_n={f_n:.4}, μ={mu:.4}"),
    )
}

/// Interpret an L-bit slice (values 0/1) as a big-endian index.
fn bits_to_index(bits: &[u8]) -> usize {
    bits.iter().fold(0usize, |acc, &b| (acc << 1) | b as usize)
}
