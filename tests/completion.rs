//! Runs that cannot produce a requested result exit 3 and say so.
//!
//! A selected test on an empty corpus prints its suite's input status and a
//! MISSING line; an unsupported Lempel–Ziv replication count prints
//! UNSUPPORTED.  A run that completes, even with failures, exits 0.

use std::process::Command;

fn run(binary: &str, args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(binary)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("spawn {binary}: {e}"));
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn a_selected_test_without_a_result_is_missing() {
    let dir = std::env::temp_dir().join(format!("entropy_completion_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let empty = dir.join("empty.bin");
    std::fs::write(&empty, []).unwrap();
    let empty = empty.to_str().unwrap();
    let bin = env!("CARGO_BIN_EXE_run_tests");

    let (code, stdout, _) = run(bin, &["--corpus", empty, "--test", "nist::frequency"]);
    assert_eq!(code, Some(3), "{stdout}");
    assert!(stdout.contains("[SKIP] nist::input_segment"), "{stdout}");
    assert!(stdout.contains("[MISSING] nist::frequency"), "{stdout}");

    let (code, stdout, _) = run(
        bin,
        &["--corpus", empty, "--test", "nist::frequency", "--json"],
    );
    assert_eq!(code, Some(3), "{stdout}");
    assert!(stdout.contains(r#""status":"missing""#), "{stdout}");

    // Without a selection the insufficient suite is reported and the run is complete.
    let (code, stdout, _) = run(bin, &["--corpus", empty, "--suite", "nist"]);
    assert_eq!(code, Some(0), "{stdout}");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn unsupported_replications_exit_3() {
    let bin = env!("CARGO_BIN_EXE_testu01_lz");
    let (code, stdout, stderr) = run(
        bin,
        &["--rng", "MT19937", "--k", "6", "--replications", "10001"],
    );
    assert_eq!(code, Some(3), "{stdout}{stderr}");
    assert!(
        stdout.contains("[UNSUPPORTED] testu01::lzw_sum"),
        "{stdout}"
    );
    let (code, _, stderr) = run(
        bin,
        &["--rng", "MT19937", "--k", "6", "--replications", "20"],
    );
    assert_eq!(code, Some(0), "{stderr}");
}
