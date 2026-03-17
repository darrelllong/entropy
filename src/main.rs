//! Test runner: exercises every test against several RNGs available on macOS,
//! running each RNG in its own OS thread.
//!
//! # Usage
//!
//! ```text
//! cargo run --release [-- [OPTIONS]]
//!
//! Options:
//!   --suite nist|diehard|dieharder   Run only this battery (repeatable).
//!   --test  <name>                   Run only tests whose name contains <name>.
//!                                    If <name> starts with a known suite prefix
//!                                    (nist::, diehard::, dieharder::) only that
//!                                    battery is generated, saving time.
//!   --quick                          Use reduced sample counts in DIEHARD/DIEHARDER.
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

// 16 M bits: enough for all Maurer L=6..16 parametric slots and for the
// signed-random-walk in random_excursions to complete ~3 191 zero-crossing
// cycles (J = √(2n/π) >> 500 minimum) for any non-degenerate generator.
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
}

impl Args {
    fn parse() -> Self {
        let mut quick = false;
        let mut explicit_suites: HashSet<Suite> = HashSet::new();
        let mut test_filter: Option<String> = None;
        let mut rng_filters: Vec<String> = Vec::new();

        let argv: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0;
        while i < argv.len() {
            match argv[i].as_str() {
                "--quick" => quick = true,
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--suite" => {
                    i += 1;
                    let v = argv.get(i).map(String::as_str).unwrap_or("");
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
            if pat.starts_with("nist::") {
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
    eprintln!(
        "Usage: run_tests [--quick] [--suite nist|diehard|dieharder] [--test <name>] [--rng <label>] [--help]\n\
         \n\
         --suite  Run only this battery.  Repeatable: --suite nist --suite diehard.\n\
         --test   Show only tests whose name contains <name>.\n\
                  Prefix nist::/diehard::/dieharder:: also limits which battery runs.\n\
         --rng    Run only RNGs whose label contains <label>. Repeatable.\n\
         --quick  Reduced sample counts in DIEHARD/DIEHARDER (faster, less sensitive).\n\
         \n\
         Examples:\n\
          run_tests                              # full battery, all RNGs\n\
          run_tests --suite nist                 # NIST SP 800-22 only\n\
          run_tests --test nist::frequency       # single test (NIST only generated)\n\
          run_tests --rng Windows                # only the Windows CRT generator\n\
          run_tests --test frequency             # all tests containing \"frequency\"\n\
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
}

type RunFn = Box<dyn FnOnce() -> RngResults + Send + 'static>;

fn make_runs(args: Args) -> Vec<RunFn> {
    let mut runs = Vec::new();

    macro_rules! run {
        ($label:expr, $rng:expr) => {{
            if args.matches_rng($label) {
                let a = args.clone();
                runs.push(Box::new(move || run_one($label, $rng, &a)) as RunFn);
            }
        }};
    }
    macro_rules! run_nist {
        ($label:expr, $rng:expr) => {{
            if args.matches_rng($label) {
                let a = args.clone();
                runs.push(Box::new(move || run_nist_only($label, $rng, &a)) as RunFn);
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
    // embeds a discrete-log trapdoor (Bernstein et al., 2014; Checkoway et
    // al., 2014).  It must never be used to produce key material.  Two P-256
    // scalar multiplications per 30-byte block make DIEHARD/DIEHARDER
    // prohibitively slow (~2 M scalar mults); NIST suite only.
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

    eprintln!("Running {n_rngs} RNGs across {n_cores} core(s), {n_cores} threads at a time…");

    let banner = "=".repeat(72);
    let work = Arc::new(Mutex::new(
        runs.into_iter()
            .enumerate()
            .collect::<VecDeque<(usize, RunFn)>>(),
    ));
    let results = Arc::new(Mutex::new(
        (0..n_rngs)
            .map(|_| None)
            .collect::<Vec<Option<RngResults>>>(),
    ));

    let worker_count = n_cores.min(n_rngs);
    let handles: Vec<_> = (0..worker_count)
        .map(|_| {
            let work = Arc::clone(&work);
            let results = Arc::clone(&results);
            thread::spawn(move || loop {
                let next = {
                    let mut queue = work.lock().expect("work queue mutex poisoned");
                    queue.pop_front()
                };
                let Some((idx, task)) = next else {
                    break;
                };
                let result = task();
                let mut out = results.lock().expect("results mutex poisoned");
                out[idx] = Some(result);
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("worker thread panicked");
    }

    let all_results = match Arc::try_unwrap(results) {
        Ok(results) => results.into_inner().expect("results mutex poisoned"),
        Err(_) => panic!("results still shared after workers finished"),
    };

    for r in all_results.into_iter().flatten() {
        print_rng_results(&r, &banner, &args);
    }
}

// ── Output ────────────────────────────────────────────────────────────────────

fn print_rng_results(r: &RngResults, banner: &str, args: &Args) {
    // Collect only matching results; skip the entire block if nothing matches.
    let matching: Vec<&TestResult> = r
        .nist
        .iter()
        .chain(&r.diehard)
        .chain(&r.dieharder)
        .filter(|t| args.matches(t.name))
        .collect();
    if matching.is_empty() {
        return;
    }

    println!("\n{banner}");
    println!("  {}", r.name);
    println!("{banner}");

    if !r.nist.is_empty() {
        let shown: Vec<&TestResult> = r.nist.iter().filter(|t| args.matches(t.name)).collect();
        if !shown.is_empty() {
            println!("\n  ── NIST SP 800-22 ({} bits) ──", r.nist_n);
            for t in shown {
                println!("  {t}");
            }
        }
    }
    if !r.diehard.is_empty() {
        let shown: Vec<&TestResult> = r.diehard.iter().filter(|t| args.matches(t.name)).collect();
        if !shown.is_empty() {
            println!("\n  ── DIEHARD unique tests ({DIEHARD_N} words) ──");
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
            println!("\n  ── DIEHARDER unique tests ({DIEHARD_N} words) ──");
            for t in shown {
                println!("  {t}");
            }
        }
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
        println!(
            "  (At α=0.01, expect ~{:.0} false FAILs by chance for a perfect RNG with {n_run} tests)",
            n_run as f64 * 0.01
        );
    }
}
