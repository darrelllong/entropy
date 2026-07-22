//! Locks the `dump_rng` and `pilot_rng` name registries together.
//!
//! `dump_rng` documents that its names match `pilot_rng`; nothing enforced
//! that until now.  These tests spawn both binaries (paths provided by cargo
//! via `CARGO_BIN_EXE_*`, stable since Rust 1.43 — same pattern as
//! `tests/dump_rng.rs`) and assert:
//!
//! 1. the two `--list` outputs are identical, and
//! 2. every listed name actually dispatches in `pilot_rng`.
//!
//! `PILOT_RNG_WORDS` is kept tiny so the whole sweep stays fast even for the
//! slow generators (Dual_EC_DRBG needs two P-256 scalar multiplications per
//! 30-byte block, which is expensive in unoptimised test builds).

use std::process::Command;

fn list(binary: &str) -> Vec<String> {
    let out = Command::new(binary)
        .arg("--list")
        .output()
        .unwrap_or_else(|e| panic!("spawn {binary} --list: {e}"));
    assert!(
        out.status.success(),
        "--list failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let names: Vec<String> = std::str::from_utf8(&out.stdout)
        .expect("UTF-8 stdout")
        .lines()
        .map(str::to_owned)
        .collect();
    assert!(!names.is_empty(), "{binary} --list returned no names");
    names
}

#[test]
fn dump_rng_and_pilot_rng_list_the_same_names() {
    assert_eq!(
        list(env!("CARGO_BIN_EXE_dump_rng")),
        list(env!("CARGO_BIN_EXE_pilot_rng")),
        "dump_rng --list and pilot_rng --list must stay identical"
    );
}

#[test]
fn every_dump_rng_name_is_dispatchable_by_pilot_rng() {
    for name in list(env!("CARGO_BIN_EXE_dump_rng")) {
        let r = Command::new(env!("CARGO_BIN_EXE_pilot_rng"))
            .arg(&name)
            .env("PILOT_RNG_WORDS", "64")
            .output()
            .unwrap_or_else(|e| panic!("spawn pilot_rng {name}: {e}"));
        assert!(
            r.status.success(),
            "pilot_rng {name} exited nonzero: {}",
            String::from_utf8_lossy(&r.stderr)
        );
        // Output contract: a single parseable MW/s line on stdout.
        let out = String::from_utf8_lossy(&r.stdout);
        assert!(
            out.trim().parse::<f64>().is_ok(),
            "pilot_rng {name} stdout is not a number: {out:?}"
        );
    }
}

#[test]
fn pilot_rng_rejects_invalid_workload_env() {
    let r = Command::new(env!("CARGO_BIN_EXE_pilot_rng"))
        .arg("counter")
        .env("PILOT_RNG_WORDS", "ten")
        .output()
        .expect("spawn pilot_rng");
    assert!(
        !r.status.success(),
        "invalid PILOT_RNG_WORDS must be a hard error"
    );
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(
        err.contains("PILOT_RNG_WORDS"),
        "diagnostic should name the variable: {err}"
    );
}
