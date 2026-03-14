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

/// Expected value μ and variance σ² of log2(A_n) for each L.
///
/// Table 3 from NIST SP 800-22 §2.9.7 and Table 1 from Maurer (1992).
/// Index is L; entry 0 is unused.
const EXPECTED_LOG_GAP_STATS: [(f64, f64); 17] = [
    (0.0, 0.0), // L=0 unused
    (0.7326495, 0.690),
    (1.5374383, 1.338),
    (2.4016068, 1.901),
    (3.3112247, 2.358),
    (4.2534266, 2.705),
    (5.217705249861, 2.954032399382),
    (6.196250654102, 3.125391868609),
    (7.183665553492, 3.238662160971),
    (8.176424757913, 3.311200879481),
    (9.172324308195, 3.356456906974),
    (10.170032291923, 3.384087030672),
    (11.168764874402, 3.400654145108),
    (12.168070314219, 3.410438009177),
    (13.167692567118, 3.416141821798),
    (14.167488448576, 3.419430397755),
    (15.167378763638, 3.421308343033),
];

const PARAMETRIC_LS: [usize; 11] = [6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
const PARAMETRIC_NAMES: [&str; 11] = [
    "maurer::universal_l06",
    "maurer::universal_l07",
    "maurer::universal_l08",
    "maurer::universal_l09",
    "maurer::universal_l10",
    "maurer::universal_l11",
    "maurer::universal_l12",
    "maurer::universal_l13",
    "maurer::universal_l14",
    "maurer::universal_l15",
    "maurer::universal_l16",
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
    universal_with_l(bits, l, q, "nist::universal")
}

/// Run Maurer's original parametric family for all recommended L values that fit
/// into the available sample, using Q = 10 * 2^L and K = floor(n / L) - Q.
///
/// This preserves the legacy NIST-shaped single result above while exposing the
/// more sensitive higher-L settings discussed in Maurer (1992).
pub fn universal_parametric_all(bits: &[u8]) -> Vec<TestResult> {
    PARAMETRIC_LS
        .into_iter()
        .zip(PARAMETRIC_NAMES)
        .map(|(l, name)| {
            let q = 10 * (1usize << l);
            universal_with_l(bits, l, q, name)
        })
        .collect()
}

fn universal_with_l(bits: &[u8], l: usize, q: usize, name: &'static str) -> TestResult {
    let n = bits.len();
    let n_blocks = n / l;
    if n_blocks <= q {
        return TestResult::insufficient(
            name,
            &format!("n too small for L={l}, Q={q} (need > {} bits)", (q + 1) * l),
        );
    }
    let k = n_blocks - q;

    let f_n = universal_statistic(bits, l, q, k);
    let (mu, sigma2) = EXPECTED_LOG_GAP_STATS[l];
    let sigma = universal_sigma(l, k, sigma2);
    let p_value = erfc((f_n - mu).abs() / (sigma * SQRT_2));

    TestResult::with_note(
        name,
        p_value,
        format!("n={n}, L={l}, Q={q}, K={k}, f_n={f_n:.4}, μ={mu:.4}, σ={sigma:.6}"),
    )
}

fn universal_statistic(bits: &[u8], l: usize, q: usize, k: usize) -> f64 {
    // Build the initialisation table: last occurrence of each L-bit pattern.
    let mut table = vec![0usize; 1usize << l];

    for i in 0..q {
        let pattern = bits_to_index(&bits[i * l..(i + 1) * l]);
        table[pattern] = i + 1;
    }

    let mut sum = 0.0f64;
    for i in q..q + k {
        let pattern = bits_to_index(&bits[i * l..(i + 1) * l]);
        let gap = i + 1 - table[pattern];
        sum += (gap as f64).log2();
        table[pattern] = i + 1;
    }
    let f_n = sum / k as f64;
    f_n
}

fn universal_sigma(l: usize, k: usize, sigma2: f64) -> f64 {
    // SP 800-22 Rev. 1a §2.9.4:
    // c(L, K) = 0.7 - 0.8/L + (1.6 + 12.8/L) * K^(-4/L)
    let l = l as f64;
    let k = k as f64;
    let c = 0.7 - 0.8 / l + (1.6 + 12.8 / l) * k.powf(-4.0 / l);
    c * (sigma2 / k).sqrt()
}

/// Interpret an L-bit slice (values 0/1) as a big-endian index.
fn bits_to_index(bits: &[u8]) -> usize {
    bits.iter().fold(0usize, |acc, &b| (acc << 1) | b as usize)
}

#[cfg(test)]
mod tests {
    use super::{universal, universal_parametric_all, universal_sigma, EXPECTED_LOG_GAP_STATS};

    #[test]
    fn published_table_covers_l16() {
        let (mu, var) = EXPECTED_LOG_GAP_STATS[16];
        assert!((mu - 15.167378763638).abs() < 1e-12);
        assert!((var - 3.421308343033).abs() < 1e-12);
    }

    #[test]
    fn uses_nist_correction_factor() {
        let sigma = universal_sigma(7, 1_000, EXPECTED_LOG_GAP_STATS[7].1);
        assert!((sigma - 0.036445141413707395).abs() < 1e-12);
    }

    #[test]
    fn parametric_family_marks_unavailable_l_values_as_skipped() {
        let bits = vec![0u8; 1_000_000];
        let results = universal_parametric_all(&bits);
        assert_eq!(results.len(), 11);
        assert!(results.iter().any(|r| r.name == "maurer::universal_l10" && !r.skipped()));
        assert!(results.iter().any(|r| r.name == "maurer::universal_l13" && r.skipped()));
    }

    #[test]
    fn nist_wrapper_still_runs_on_minimum_supported_size() {
        let bits = vec![0u8; 387_840];
        let result = universal(&bits);
        assert_eq!(result.name, "nist::universal");
        assert!(!result.skipped());
    }
}
