//! Integration test for the `dump_rng` binary.
//!
//! Build the binary with `cargo test --release --test dump_rng`; cargo
//! invokes the test runner which spawns the compiled binary.  We exercise
//! every name listed by `dump_rng --list` for a small word count, verify
//! the byte count, and check the failure paths (unknown name, missing
//! argument).
//!
//! Notes
//! -----
//! - We use `count = 4` for every name (16 bytes).  Even Dual_EC_DRBG, the
//!   slowest generator, finishes one P-256 block in well under a second.
//! - The binary path is exposed via the `CARGO_BIN_EXE_dump_rng` env var
//!   that cargo sets for integration tests (stable since Rust 1.43).

use std::process::{Command, Stdio};

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_dump_rng")
}

#[test]
fn list_returns_one_name_per_line_and_no_blanks() {
    let out = Command::new(binary())
        .arg("--list")
        .output()
        .expect("spawn dump_rng --list");
    assert!(out.status.success(), "--list failed: {}",
            String::from_utf8_lossy(&out.stderr));
    let names: Vec<&str> = std::str::from_utf8(&out.stdout)
        .expect("UTF-8 stdout")
        .lines()
        .collect();
    assert!(!names.is_empty(), "--list returned no names");
    for n in &names {
        assert!(!n.is_empty(), "blank name in --list output");
        assert!(n.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "non-canonical name {n:?}");
    }
}

#[test]
fn every_listed_name_dumps_4_words() {
    let out = Command::new(binary()).arg("--list").output().unwrap();
    let names = std::str::from_utf8(&out.stdout).unwrap();
    for name in names.lines() {
        let r = Command::new(binary())
            .args([name, "4"])
            .output()
            .unwrap_or_else(|e| panic!("spawn {name}: {e}"));
        assert!(r.status.success(),
                "{name} exited nonzero: stderr={}",
                String::from_utf8_lossy(&r.stderr));
        assert_eq!(r.stdout.len(), 16, "{name} produced {} bytes (expected 16)",
                   r.stdout.len());
    }
}

#[test]
fn count_zero_is_valid_and_produces_no_output() {
    let r = Command::new(binary())
        .args(["pcg64", "0"])
        .output()
        .unwrap();
    assert!(r.status.success());
    assert!(r.stdout.is_empty());
}

#[test]
fn unknown_name_exits_nonzero_with_diagnostic() {
    let r = Command::new(binary())
        .args(["totally_bogus_rng", "1"])
        .stdout(Stdio::null())
        .output()
        .unwrap();
    assert!(!r.status.success(), "unknown name should exit nonzero");
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(err.contains("unknown RNG"), "diagnostic missing: {err}");
}

#[test]
fn missing_args_exits_nonzero_with_usage() {
    let r = Command::new(binary())
        .stdout(Stdio::null())
        .output()
        .unwrap();
    assert!(!r.status.success());
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(err.contains("usage"), "usage missing: {err}");
}

/// Fixed-seed reproducibility check.  This is the only test that catches
/// a swapped dispatch arm (e.g. `pcg64` accidentally wired to `Pcg32`).
/// The expected hex blobs were captured against this commit; if any
/// generator's seeding or output ordering changes, regenerate them
/// deliberately.
#[test]
fn fixed_seed_first_words_are_stable() {
    fn check(name: &str, count: u32, expected_hex: &str) {
        let r = Command::new(binary())
            .args([name, &count.to_string()])
            .output()
            .unwrap_or_else(|e| panic!("spawn {name}: {e}"));
        assert!(r.status.success(),
                "{name} exited nonzero: stderr={}",
                String::from_utf8_lossy(&r.stderr));
        let got = r.stdout.iter().map(|b| format!("{b:02x}")).collect::<String>();
        assert_eq!(got, expected_hex, "{name} first-words drift");
    }
    // Counter (seed=0): little-endian u32 = 0, 1, 2, 3.
    check("counter", 4,
          "00000000010000000200000003000000");
    // Constant (value=0xDEAD_DEAD): four LE copies.
    check("constant", 4,
          "addeaddeaddeaddeaddeaddeaddeadde");
    // PCG64 with (state=1, seq=1): 8 LE u32s.
    check("pcg64", 8,
          "f05505c0842f69d4b0090fbb04c96ae212028683ed01361c27a04f5e8de230b9");
    // MT19937 seed=19650218.
    check("mt19937", 4,
          "5eb99d8ad605bd1c932ffbf86ff1cfe6");
    // Xorshift32 seed=1.
    check("xorshift32", 4,
          "2120040001060804c5a8cc9d4f995512");
}

#[test]
fn non_numeric_count_exits_nonzero() {
    let r = Command::new(binary())
        .args(["pcg64", "not-a-number"])
        .stdout(Stdio::null())
        .output()
        .unwrap();
    assert!(!r.status.success());
}
