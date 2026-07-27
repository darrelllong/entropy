//! Test runner: exercises every test against several RNGs available on macOS,
//! distributing RNGs across a pool of `min(cores, RNG count)` worker threads.
//!
//! # Usage
//!
//! ```text
//! cargo run --release [-- [OPTIONS]]
//!
//! Options:
//!   --suite nist|diehard|dieharder   Run only this battery (repeatable).
//!   --test  <name>                   Show only tests whose name contains <name>
//!                                    (the selected batteries still run in full).
//!                                    If <name> starts with a known suite prefix
//!                                    (nist::, diehard::, dieharder::, or maurer::,
//!                                    which the NIST battery emits) only that
//!                                    battery is generated, saving time.
//!   --rng   <label>                  Run only RNGs whose label contains <label>
//!                                    (repeatable).
//!   --quick                          Use reduced sample counts in DIEHARD/DIEHARDER.
//!   --fail-on-fail                   Exit 1 if any shown test FAILed.
//!
//! Exit codes: 0 = ran to completion; 1 = usage error, or FAILs under
//! `--fail-on-fail`; 2 = an RNG task panicked (results incomplete).
//!   --help                           Print this message and exit.
//! ```
//!
//! Examples:
//! ```text
//! cargo run --release                              # full battery, all RNGs
//! cargo run --release -- --suite nist              # NIST only
//! cargo run --release -- --test nist::frequency    # one test (NIST only generated)
//! cargo run --release -- --test frequency          # any test containing "frequency"
//! cargo run --release -- --suite diehard --quick   # DIEHARD with reduced counts
//! ```

use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use cryptography::{
    Camellia128, Cast128, Grasshopper, Rabbit, Salsa20, Seed as SeedCipher, Serpent128, Sm4,
    Snow3g, Twofish128, Zuc128,
};
use entropy::rng::{
    AesCtr, BlockCtrRng, BsdRandCompat, BsdRandom, ChaCha20Rng,
    ConstantRng, CounterRng, CryptoCtrDrbg, DualEcDrbg, HashDrbg, HmacDrbg, Jsf64, Lcg32,
    LcgVariant,
    LinuxLibcRandom, Mt19937, OsRng, Pcg32, Pcg64, Rand48, Rng, Sfc64, SpongeBob, Squidward,
    StreamRng, SystemVRand, WindowsDotNetRandom, WindowsMsvcRand, WindowsVb6Rnd, WyRand,
    Xoroshiro128, Xorshift32, Xorshift64, Xoshiro256,
};
use entropy::seed::{IV16, IV8, K16, K32};
use entropy::{diehard, dieharder, nist, result::TestResult};
use std::thread;

// ── Configuration ─────────────────────────────────────────────────────────────

// 16 M bits: enough for the signed-random-walk in random_excursions to
// complete ~3 191 zero-crossing cycles (J = √(2n/π) >> 500 minimum) for any
// non-degenerate generator, and for every slot of the parametric Maurer
// family (L=5..16, maurer::universal_l*) to run.  Caveat: at this size the
// L=15 and L=16 slots run with K ≈ 739 k and ≈ 345 k blocks — far below the
// K ≥ 1000·2^L (32.8 M / 65.5 M) calibration assumption — so their power is
// degraded; treat those two slots as indicative only.
const NIST_N: usize = 16_000_000;
const DIEHARD_N: usize = 16_000_000;

// ── CLI args ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Suite {
    Nist,
    Diehard,
    Dieharder,
}

#[derive(Clone)]
struct Args {
    quick: bool,
    suites: HashSet<Suite>,      // empty = all three
    test_filter: Option<String>, // substring match on TestResult::name
    rng_filters: Vec<String>,    // substring match on RNG label
    fail_on_fail: bool,          // exit nonzero if any shown test FAILed
}

impl Args {
    fn parse() -> Self {
        let mut quick = false;
        let mut explicit_suites: HashSet<Suite> = HashSet::new();
        let mut test_filter: Option<String> = None;
        let mut rng_filters: Vec<String> = Vec::new();
        let mut fail_on_fail = false;

        let argv: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0;
        while i < argv.len() {
            match argv[i].as_str() {
                "--quick" => quick = true,
                "--fail-on-fail" => fail_on_fail = true,
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--suite" => {
                    i += 1;
                    let v = match argv.get(i) {
                        Some(v) => v.as_str(),
                        None => die("--suite requires an argument"),
                    };
                    match v {
                        "nist" => {
                            explicit_suites.insert(Suite::Nist);
                        }
                        "diehard" => {
                            explicit_suites.insert(Suite::Diehard);
                        }
                        "dieharder" => {
                            explicit_suites.insert(Suite::Dieharder);
                        }
                        other => die(&format!(
                            "unknown suite '{other}' — use: nist, diehard, dieharder"
                        )),
                    }
                }
                "--test" => {
                    i += 1;
                    match argv.get(i) {
                        Some(v) => test_filter = Some(v.clone()),
                        None => die("--test requires an argument"),
                    }
                }
                "--rng" => {
                    i += 1;
                    match argv.get(i) {
                        Some(v) => rng_filters.push(v.clone()),
                        None => die("--rng requires an argument"),
                    }
                }
                other => die(&format!(
                    "unknown option '{other}' — run with --help for usage"
                )),
            }
            i += 1;
        }

        // If --suite was not given but --test has a suite prefix, infer the suite
        // so we don't generate unnecessary random data for other batteries.
        let suites = if !explicit_suites.is_empty() {
            explicit_suites
        } else if let Some(ref pat) = test_filter {
            let mut inferred = HashSet::new();
            // maurer:: slots are emitted by nist::run_all, so they belong to
            // the NIST battery for inference purposes.
            if pat.starts_with("nist::") || pat.starts_with("maurer::") {
                inferred.insert(Suite::Nist);
            } else if pat.starts_with("dieharder::") {
                inferred.insert(Suite::Dieharder);
            } else if pat.starts_with("diehard::") {
                inferred.insert(Suite::Diehard);
            }
            // No prefix → run all suites so we catch the test wherever it lives.
            inferred
        } else {
            HashSet::new() // empty = all three
        };

        Args {
            quick,
            suites,
            test_filter,
            rng_filters,
            fail_on_fail,
        }
    }

    fn run_suite(&self, s: &Suite) -> bool {
        self.suites.is_empty() || self.suites.contains(s)
    }

    fn matches(&self, name: &str) -> bool {
        self.test_filter
            .as_ref()
            .is_none_or(|pat| name.contains(pat.as_str()))
    }

    fn matches_rng(&self, label: &str) -> bool {
        self.rng_filters.is_empty() || self.rng_filters.iter().any(|pat| label.contains(pat))
    }
}

fn print_usage() {
    // NOTE: no `\`-continuations here — they strip the leading whitespace
    // that indents the flag descriptions' continuation lines.
    println!(
        "\
Usage: run_tests [--quick] [--suite nist|diehard|dieharder] [--test <name>] [--rng <label>] [--fail-on-fail] [--help]

 --suite         Run only this battery.  Repeatable: --suite nist --suite diehard.
 --test          Show only tests whose name contains <name>.
                 The selected batteries still run in full; this filters output.
                 Prefix nist::/diehard::/dieharder:: (or maurer::, emitted by
                 the NIST battery) also limits which battery runs.
 --rng           Run only RNGs whose label contains <label>. Repeatable.
 --quick         Reduced sample counts in DIEHARD/DIEHARDER (faster, less sensitive).
 --fail-on-fail  Exit 1 if any shown test FAILed.  Without it, exit 0 only
                 means the battery ran to completion.  Negative-control RNGs
                 (BAD…, Constant, Counter, Dual_EC_DRBG) are expected to FAIL,
                 so combine this with --rng for single-RNG CI runs.

 Exit codes: 0 = ran to completion (tests may still have FAILed unless
 --fail-on-fail); 1 = usage error, or FAILs with --fail-on-fail;
 2 = an RNG task panicked and its results are missing.

 Examples:
  run_tests                              # full battery, all RNGs
  run_tests --suite nist                 # NIST SP 800-22 only
  run_tests --test nist::frequency       # single test (NIST only generated)
  run_tests --rng Windows                # only the Windows generators
  run_tests --test frequency             # all tests containing \"frequency\"
  run_tests --suite diehard --quick"
    );
}

fn die(msg: &str) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(1);
}

// ── RNG descriptors ───────────────────────────────────────────────────────────

struct RngResults {
    name: &'static str,
    nist_n: usize,
    nist: Vec<TestResult>,
    diehard: Vec<TestResult>,
    dieharder: Vec<TestResult>,
    /// True when this generator is *intrinsically* restricted to the NIST
    /// suite (e.g. Dual_EC, too slow for DIEHARD/DIEHARDER), as opposed to a
    /// user `--suite`/`--test` filter that happened to select NIST only.
    nist_only: bool,
}

type RunFn = Box<dyn FnOnce() -> RngResults + Send + 'static>;

fn make_runs(args: Args) -> Vec<(&'static str, RunFn)> {
    let mut runs = Vec::new();

    macro_rules! run {
        ($label:expr, $rng:expr) => {{
            if args.matches_rng($label) {
                let a = args.clone();
                runs.push((
                    $label,
                    Box::new(move || run_one($label, $rng, &a)) as RunFn,
                ));
            }
        }};
    }
    macro_rules! run_nist {
        ($label:expr, $rng:expr) => {{
            if args.matches_rng($label) {
                let a = args.clone();
                runs.push((
                    $label,
                    Box::new(move || run_nist_only($label, $rng, &a)) as RunFn,
                ));
            }
        }};
    }

    run!("OsRng (/dev/urandom)", OsRng::new());
    // MT19937: full 624-word state is recoverable from 624 consecutive outputs.
    // Not for adversarial contexts. Fixed seed is for test reproducibility only.
    run!("MT19937 (seed=19650218)", Mt19937::new(19650218));
    run!("Xorshift64 (seed=1)", Xorshift64::new(1));
    run!("Xorshift32 (seed=1)", Xorshift32::new(1));
    run!(
        "BAD Unix System V rand() (15-bit LCG, seed=1)",
        SystemVRand::new(1)
    );
    run!("BAD Unix System V mrand48() (seed=1)", Rand48::new(1));
    run!("BAD Unix BSD random() TYPE_3 (seed=1)", BsdRandom::new(1));
    run!(
        "BAD Unix Linux glibc rand()/random() (seed=1)",
        LinuxLibcRandom::new(1)
    );
    run!(
        "BAD Unix FreeBSD12 rand_r() compat (seed=1)",
        BsdRandCompat::new(1)
    );
    run!(
        "BAD Windows CRT rand() (MSVC/UCRT lineage, seed=1)",
        WindowsMsvcRand::new(1)
    );
    run!(
        "BAD Windows VB6/VBA Rnd() (project seed=1)",
        WindowsVb6Rnd::new(1)
    );
    run!(
        "BAD Windows .NET Random(seed=1) compat",
        WindowsDotNetRandom::new(1)
    );
    run!(
        "ANSI C sample LCG (1103515245,12345; seed=1)",
        Lcg32::ansi_c()
    );
    run!("LCG MINSTD (seed=1)", Lcg32::minstd());
    run!(
        "BAD Borland C++ rand() LCG (seed=1)",
        Lcg32::new(LcgVariant::Borland, 1)
    );
    run!("AES-128-CTR (NIST key)", AesCtr::with_nist_key());
    // Block-cipher CTR-mode RNGs (NIST SP 800-38A).
    // FOR TESTING ONLY — all use public test-vector keys (K16/K32) and
    // counter=0.  Reusing a (key, counter) starting state produces identical
    // output streams; any production use requires a unique key and a counter
    // that is never rewound.
    run!(
        "Camellia-128-CTR (key=00..0f)",
        BlockCtrRng::new(Camellia128::new(&K16), 0)
    );
    run!(
        "Twofish-128-CTR (key=00..0f)",
        BlockCtrRng::new(Twofish128::new(&K16), 0)
    );
    run!(
        "Serpent-128-CTR (key=00..0f)",
        BlockCtrRng::new(Serpent128::new(&K16), 0)
    );
    run!("SM4-CTR (key=00..0f)", BlockCtrRng::new(Sm4::new(&K16), 0));
    run!(
        "Grasshopper-CTR (key=00..1f)",
        BlockCtrRng::new(Grasshopper::new(&K32), 0)
    );
    run!(
        "CAST-128-CTR (key=00..0f)",
        BlockCtrRng::new(Cast128::new(&K16), 0)
    );
    run!(
        "SEED-CTR (key=00..0f)",
        BlockCtrRng::new(SeedCipher::new(&K16), 0)
    );
    // Stream-cipher RNGs.
    run!(
        "Rabbit (key=00..0f, iv=00..07)",
        StreamRng::new(Rabbit::new(&K16, &IV8))
    );
    run!(
        "Salsa20 (key=00..1f, nonce=00..07)",
        StreamRng::new(Salsa20::new(&K32, &IV8))
    );
    run!(
        "Snow3G (key=00..0f, iv=00..0f)",
        StreamRng::new(Snow3g::new(&K16, &IV16))
    );
    run!(
        "ZUC-128 (key=00..0f, iv=00..0f)",
        StreamRng::new(Zuc128::new(&K16, &IV16))
    );
    run!(
        "SpongeBob (SHA3-512 chain, OsRng seed)",
        SpongeBob::from_os_rng()
    );
    run!(
        "Squidward (SHA-256 chain, OsRng seed)",
        Squidward::from_os_rng()
    );
    run!("PCG32 (OsRng seed)", Pcg32::from_os_rng());
    run!("PCG64 (OsRng seed)", Pcg64::from_os_rng());
    run!("Xoshiro256 (OsRng seed)", Xoshiro256::from_os_rng());
    run!("Xoroshiro128 (OsRng seed)", Xoroshiro128::from_os_rng());
    run!("WyRand (OsRng seed)", WyRand::from_os_rng());
    run!("SFC64 (OsRng seed)", Sfc64::from_os_rng());
    run!("JSF64 (OsRng seed)", Jsf64::from_os_rng());
    run!("ChaCha20 CSPRNG (OsRng key)", ChaCha20Rng::from_os_rng());
    run!("HMAC_DRBG SHA-256 (OsRng seed)", HmacDrbg::from_os_rng());
    run!("Hash_DRBG SHA-256 (OsRng seed)", HashDrbg::from_os_rng());
    run!(
        "cryptography::CtrDrbgAes256 (seed=00..2f)",
        CryptoCtrDrbg::with_test_seed()
    );
    run!("Constant (0xDEAD_DEAD)", ConstantRng::new(0xDEAD_DEAD));
    run!("Counter (0,1,2,…)", CounterRng::new(0));
    // Dual_EC_DRBG: included for reference only.
    // WARNING: This generator is known to be backdoored — the NIST Q point
    // embeds a discrete-log trapdoor (Checkoway et al., 2014; Bernstein,
    // Lange, and Niederhagen, 2016).  It must never be used to produce key
    // material.  Three P-256
    // scalar multiplications per 30-byte block make DIEHARD/DIEHARDER
    // prohibitively slow (~3 M scalar mults); NIST suite only.
    let mut dual_ec_seed = [0u8; 32];
    dual_ec_seed[31] = 1; // seed = 0x00…01 — INSECURE TEST SEED, DO NOT COPY
    run_nist!("Dual_EC_DRBG P-256 (NIST Q, seed=0x00..01)", DualEcDrbg::p256(&dual_ec_seed));

    if runs.is_empty() {
        die("no RNG labels matched --rng filter");
    }

    runs
}

fn run_one<R: Rng>(name: &'static str, mut rng: R, args: &Args) -> RngResults {
    let nist = if args.run_suite(&Suite::Nist) {
        nist::run_all(&mut rng, NIST_N)
    } else {
        vec![]
    };
    let diehard = if args.run_suite(&Suite::Diehard) {
        diehard::run_all(&mut rng, DIEHARD_N, args.quick)
    } else {
        vec![]
    };
    let dieharder = if args.run_suite(&Suite::Dieharder) {
        dieharder::run_all(&mut rng, DIEHARD_N, args.quick)
    } else {
        vec![]
    };
    RngResults {
        name,
        nist_n: NIST_N,
        nist,
        diehard,
        dieharder,
        nist_only: false,
    }
}

/// Run only NIST SP 800-22 — for RNGs too slow for DIEHARD/DIEHARDER.
fn run_nist_only<R: Rng>(name: &'static str, mut rng: R, args: &Args) -> RngResults {
    let nist = if args.run_suite(&Suite::Nist) {
        nist::run_all(&mut rng, NIST_N)
    } else {
        vec![]
    };
    RngResults {
        name,
        nist_n: NIST_N,
        nist,
        diehard: vec![],
        dieharder: vec![],
        nist_only: true,
    }
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    let args = Args::parse();

    let n_cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let runs = make_runs(args.clone());
    let n_rngs = runs.len();

    let worker_count = n_cores.min(n_rngs);
    eprintln!("Running {n_rngs} RNGs across {n_cores} core(s), {worker_count} threads at a time…");

    let banner = "=".repeat(72);
    let work = Arc::new(Mutex::new(
        runs.into_iter()
            .enumerate()
            .map(|(idx, (label, task))| (idx, label, task))
            .collect::<VecDeque<(usize, &'static str, RunFn)>>(),
    ));
    let results = Arc::new(Mutex::new(
        (0..n_rngs)
            .map(|_| None)
            .collect::<Vec<Option<RngResults>>>(),
    ));

    let any_panicked = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let handles: Vec<_> = (0..worker_count)
        .map(|_| {
            let work = Arc::clone(&work);
            let results = Arc::clone(&results);
            let any_panicked = Arc::clone(&any_panicked);
            thread::spawn(move || loop {
                let next = {
                    let mut queue = work.lock().expect("work queue mutex poisoned");
                    queue.pop_front()
                };
                let Some((idx, label, task)) = next else {
                    break;
                };
                // A panic in one RNG's task must not abort the whole run —
                // catch it, report which RNG died, and keep serving the queue
                // so every completed result is still printed at the end.
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(task)) {
                    Ok(result) => {
                        let mut out = results.lock().expect("results mutex poisoned");
                        out[idx] = Some(result);
                    }
                    Err(_) => {
                        any_panicked.store(true, std::sync::atomic::Ordering::Relaxed);
                        eprintln!("error: RNG '{label}' panicked; its results are omitted");
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        if handle.join().is_err() {
            any_panicked.store(true, std::sync::atomic::Ordering::Relaxed);
            eprintln!("error: a worker thread panicked; some results may be missing");
        }
    }

    let all_results = match Arc::try_unwrap(results) {
        Ok(results) => results.into_inner().expect("results mutex poisoned"),
        Err(_) => panic!("results still shared after workers finished"),
    };

    let mut total_fail = 0usize;
    for r in all_results.into_iter().flatten() {
        total_fail += print_rng_results(&r, &banner, &args);
    }
    // A panicked task means the battery did NOT run to completion — exit
    // nonzero regardless of --fail-on-fail, upholding the documented
    // "exit 0 = ran to completion" contract.
    if any_panicked.load(std::sync::atomic::Ordering::Relaxed) {
        eprintln!("error: at least one RNG task panicked; results above are incomplete");
        std::process::exit(2);
    }
    if args.fail_on_fail && total_fail > 0 {
        eprintln!("error: {total_fail} test(s) FAILed and --fail-on-fail was given");
        std::process::exit(1);
    }
}

// ── Output ────────────────────────────────────────────────────────────────────

/// Format an integer with thousands separators (16000000 → "16,000,000").
fn group_thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Print one RNG's block; returns the number of shown tests that FAILed.
fn print_rng_results(r: &RngResults, banner: &str, args: &Args) -> usize {
    // Collect only matching results; skip the entire block if nothing matches.
    let matching: Vec<&TestResult> = r
        .nist
        .iter()
        .chain(&r.diehard)
        .chain(&r.dieharder)
        .filter(|t| args.matches(t.name))
        .collect();
    if matching.is_empty() {
        return 0;
    }

    println!("\n{banner}");
    println!("  {}", r.name);
    println!("{banner}");

    if !r.nist.is_empty() {
        let shown: Vec<&TestResult> = r.nist.iter().filter(|t| args.matches(t.name)).collect();
        if !shown.is_empty() {
            println!("\n  ── NIST SP 800-22 ({} bits) ──", group_thousands(r.nist_n));
            for t in shown {
                println!("  {t}");
            }
        }
    }
    if !r.diehard.is_empty() {
        let shown: Vec<&TestResult> = r.diehard.iter().filter(|t| args.matches(t.name)).collect();
        if !shown.is_empty() {
            println!(
                "\n  ── DIEHARD unique tests ({} words) ──",
                group_thousands(DIEHARD_N)
            );
            for t in shown {
                println!("  {t}");
            }
        }
    }
    if !r.dieharder.is_empty() {
        let shown: Vec<&TestResult> = r
            .dieharder
            .iter()
            .filter(|t| args.matches(t.name))
            .collect();
        if !shown.is_empty() {
            println!(
                "\n  ── DIEHARDER unique tests ({} words) ──",
                group_thousands(DIEHARD_N)
            );
            for t in shown {
                println!("  {t}");
            }
        }
    }

    // Make an intrinsic suite restriction explicit rather than silently
    // omitting the empty blocks (e.g. Dual_EC runs NIST-only for cost).  Gated
    // on the intrinsic `nist_only` flag, not on empty blocks, so it does NOT
    // fire for every generator when the user passes `--suite nist`.
    if r.nist_only && r.nist.iter().any(|t| args.matches(t.name)) {
        println!("\n  (DIEHARD/DIEHARDER not run for this generator — NIST suite only.)");
    }

    let pass = matching.iter().filter(|t| t.passed()).count();
    let fail = matching
        .iter()
        .filter(|t| !t.passed() && !t.skipped())
        .count();
    let skip = matching.iter().filter(|t| t.skipped()).count();
    println!("\n  Summary: {pass} PASS, {fail} FAIL, {skip} SKIP");
    let n_run = matching.iter().filter(|t| !t.skipped()).count();
    if n_run > 0 {
        // Many slots share a name: the 148 non-overlapping templates and the
        // up-to-510 bit-distribution patterns are correlated sub-tests, not
        // independent trials, so the naive n·α count would overstate the
        // expected noise.  Report the family count and frame the estimate as an
        // upper bound read family-wise.
        let families: std::collections::HashSet<&str> = matching
            .iter()
            .filter(|t| !t.skipped())
            .map(|t| t.name)
            .collect();
        println!(
            "  ({n_run} run slots spanning {} distinct test families; \
             `non_overlapping_template` and `bit_distribution` each contribute \
             many correlated slots.)",
            families.len()
        );
        println!(
            "  (At α=0.01 a perfect RNG fails ~{:.0} isolated slots by chance; a \
             whole family failing together is the real signal.)",
            n_run as f64 * 0.01
        );
    }
    fail
}
