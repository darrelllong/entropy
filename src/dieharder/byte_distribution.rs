//! DIEHARDER test 205 — dab_bytedistrib.
//!
//! Chi-square test on the 256-bucket histogram of byte values (0x00–0xFF).
//! Tests that each possible byte value appears with probability 1/256.
//!
//! # Author
//! David Bauer, *Dieharder* (2006), test `dab_bytedistrib`.

use crate::{math::igamc, result::TestResult};

const N_BYTES: usize = 2_560_000; // 10 000 bytes per bucket expected

/// Run the byte distribution test.
///
/// # Author
/// David Bauer, Dieharder (2006), `dab_bytedistrib`.
pub fn byte_distribution(words: &[u32]) -> TestResult {
    let bytes_available = words.len() * 4;
    if bytes_available < N_BYTES {
        return TestResult::insufficient("dieharder::byte_distribution", "not enough bytes");
    }

    let mut counts = [0u32; 256];
    let mut byte_count = 0usize;
    'outer: for &w in words {
        for &b in &w.to_le_bytes() {
            counts[b as usize] += 1;
            byte_count += 1;
            if byte_count >= N_BYTES { break 'outer; }
        }
    }

    let expected = byte_count as f64 / 256.0;
    let chi_sq: f64 = counts.iter().map(|&c| (c as f64 - expected).powi(2) / expected).sum();

    let p_value = igamc(127.5, chi_sq / 2.0); // df = 255

    TestResult::with_note(
        "dieharder::byte_distribution",
        p_value,
        format!("bytes={byte_count}, expected/bucket={expected:.1}, χ²={chi_sq:.4}"),
    )
}
