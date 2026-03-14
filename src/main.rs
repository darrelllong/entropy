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

use entropy::rng::{
    AesCtr, BlumBlumShub, BlumMicali, CRand, ConstantRng, CounterRng,
    DualEcDrbg, Lcg32, Mt19937, OsRng, Rand48, Rng, Xorshift32, Xorshift64,
};
use entropy::{diehard, dieharder, nist, result::TestResult};
use std::thread;

// ── Configuration ─────────────────────────────────────────────────────────────

const NIST_N:     usize = 1_000_000;
const DIEHARD_N:  usize = 16_000_000;

// ── CLI args ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Suite { Nist, Diehard, Dieharder }

#[derive(Clone)]
struct Args {
    quick:       bool,
    suites:      HashSet<Suite>,  // empty = all three
    test_filter: Option<String>,  // substring match on TestResult::name
}

impl Args {
    fn parse() -> Self {
        let mut quick = false;
        let mut explicit_suites: HashSet<Suite> = HashSet::new();
        let mut test_filter: Option<String> = None;
        let mut suite_explicit = false;

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
                        "nist"      => { explicit_suites.insert(Suite::Nist); }
                        "diehard"   => { explicit_suites.insert(Suite::Diehard); }
                        "dieharder" => { explicit_suites.insert(Suite::Dieharder); }
                        other => die(&format!(
                            "unknown suite '{other}' — use: nist, diehard, dieharder"
                        )),
                    }
                    suite_explicit = true;
                }
                "--test" => {
                    i += 1;
                    match argv.get(i) {
                        Some(v) => test_filter = Some(v.clone()),
                        None    => die("--test requires an argument"),
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
        let suites = if suite_explicit {
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
            HashSet::new()  // empty = all three
        };

        Args { quick, suites, test_filter }
    }

    fn run_suite(&self, s: &Suite) -> bool {
        self.suites.is_empty() || self.suites.contains(s)
    }

    fn matches(&self, name: &str) -> bool {
        self.test_filter.as_ref().map_or(true, |pat| name.contains(pat.as_str()))
    }
}

fn print_usage() {
    eprintln!(
        "Usage: run_tests [--quick] [--suite nist|diehard|dieharder] [--test <name>] [--help]\n\
         \n\
         --suite  Run only this battery.  Repeatable: --suite nist --suite diehard.\n\
         --test   Show only tests whose name contains <name>.\n\
                  Prefix nist::/diehard::/dieharder:: also limits which battery runs.\n\
         --quick  Reduced sample counts in DIEHARD/DIEHARDER (faster, less sensitive).\n\
         \n\
         Examples:\n\
           run_tests                              # full battery, all RNGs\n\
           run_tests --suite nist                 # NIST SP 800-22 only\n\
           run_tests --test nist::frequency       # single test (NIST only generated)\n\
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
    name:      &'static str,
    nist:      Vec<TestResult>,
    diehard:   Vec<TestResult>,
    dieharder: Vec<TestResult>,
}

type RunFn = Box<dyn FnOnce() -> RngResults + Send + 'static>;

fn make_runs(args: Args) -> Vec<RunFn> {
    macro_rules! run {
        ($label:expr, $rng:expr) => {{
            let a = args.clone();
            Box::new(move || run_one($label, $rng, &a)) as RunFn
        }};
    }
    macro_rules! run_nist {
        ($label:expr, $rng:expr) => {{
            let a = args.clone();
            Box::new(move || run_nist_only($label, $rng, &a)) as RunFn
        }};
    }

    vec![
        run!("OsRng (/dev/urandom)",         OsRng::new()),
        run!("MT19937 (seed=19650218)",       Mt19937::new(19650218)),
        run!("Xorshift64 (seed=1)",           Xorshift64::new(1)),
        run!("Xorshift32 (seed=1)",           Xorshift32::new(1)),
        run!("C rand() (seed=1)",             CRand::new(1)),
        run!("C mrand48 (seed=1)",            Rand48::new(1)),
        run!("LCG glibc rand (seed=1)",       Lcg32::glibc()),
        run!("LCG MINSTD (seed=1)",           Lcg32::minstd()),
        run!("BBS (p=2³¹−1, q=4294967291)",  BlumBlumShub::new(2_147_483_647, 4_294_967_291, 1_234_567)),
        run!("Blum-Micali (p=2³¹−1, g=7)",   BlumMicali::new(2_147_483_647, 7, 42)),
        run!("AES-128-CTR (NIST key)",        AesCtr::with_nist_key()),
        run!("Constant (0xDEAD_DEAD)",        ConstantRng::new(0xDEAD_DEAD)),
        run!("Counter (0,1,2,…)",             CounterRng::new(0)),
        // Dual_EC_DRBG: two P-256 scalar multiplications per 30-byte block.
        // DIEHARD/DIEHARDER would require ~2 M scalar mults — NIST only.
        run_nist!("Dual_EC_DRBG P-256 (NIST Q, seed=0x00..01)",
            DualEcDrbg::p256(&[0u8; 31].iter().copied().chain([1u8]).collect::<Vec<_>>())),
    ]
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
    RngResults { name, nist, diehard, dieharder }
}

/// Run only NIST SP 800-22 — for RNGs too slow for DIEHARD/DIEHARDER.
fn run_nist_only<R: Rng>(name: &'static str, mut rng: R, args: &Args) -> RngResults {
    let nist = if args.run_suite(&Suite::Nist) {
        nist::run_all(&mut rng, NIST_N)
    } else {
        vec![]
    };
    RngResults { name, nist, diehard: vec![], dieharder: vec![] }
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    let args = Args::parse();

    let n_cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let runs = make_runs(args.clone());
    let n_rngs = runs.len();

    eprintln!(
        "Running {n_rngs} RNGs across {n_cores} core(s), {n_cores} threads at a time…"
    );

    let banner = "=".repeat(72);
    let mut all_results: Vec<Option<RngResults>> = (0..n_rngs).map(|_| None).collect();

    let mut run_iter = runs.into_iter().enumerate();
    loop {
        let batch: Vec<(usize, RunFn)> = run_iter.by_ref().take(n_cores).collect();
        if batch.is_empty() { break; }
        let handles: Vec<(usize, thread::JoinHandle<RngResults>)> = batch
            .into_iter()
            .map(|(idx, f)| (idx, thread::spawn(f)))
            .collect();
        for (idx, handle) in handles {
            all_results[idx] = Some(handle.join().expect("worker thread panicked"));
        }
    }

    for r in all_results.into_iter().flatten() {
        print_rng_results(&r, &banner, &args);
    }
}

// ── Output ────────────────────────────────────────────────────────────────────

fn print_rng_results(r: &RngResults, banner: &str, args: &Args) {
    // Collect only matching results; skip the entire block if nothing matches.
    let matching: Vec<&TestResult> = r.nist.iter()
        .chain(&r.diehard)
        .chain(&r.dieharder)
        .filter(|t| args.matches(t.name))
        .collect();
    if matching.is_empty() { return; }

    println!("\n{banner}");
    println!("  {}", r.name);
    println!("{banner}");

    if !r.nist.is_empty() {
        let shown: Vec<&TestResult> = r.nist.iter().filter(|t| args.matches(t.name)).collect();
        if !shown.is_empty() {
            println!("\n  ── NIST SP 800-22 ({NIST_N} bits) ──");
            for t in shown { println!("  {t}"); }
        }
    }
    if !r.diehard.is_empty() {
        let shown: Vec<&TestResult> = r.diehard.iter().filter(|t| args.matches(t.name)).collect();
        if !shown.is_empty() {
            println!("\n  ── DIEHARD unique tests ({DIEHARD_N} words) ──");
            for t in shown { println!("  {t}"); }
        }
    }
    if !r.dieharder.is_empty() {
        let shown: Vec<&TestResult> = r.dieharder.iter().filter(|t| args.matches(t.name)).collect();
        if !shown.is_empty() {
            println!("\n  ── DIEHARDER unique tests ({DIEHARD_N} words) ──");
            for t in shown { println!("  {t}"); }
        }
    }

    let pass = matching.iter().filter(|t| t.passed()).count();
    let fail = matching.iter().filter(|t| !t.passed() && !t.skipped()).count();
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
