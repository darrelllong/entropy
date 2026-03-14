//! Test runner: exercises every test against several RNGs available on macOS,
//! running each RNG in its own OS thread.
//!
//! Run with:
//!   cargo run --release

use entropy::rng::{
    BlumBlumShub, BlumMicali, CRand, ConstantRng, CounterRng,
    Lcg32, Mt19937, OsRng, Rand48, Rng, Xorshift32, Xorshift64,
};
use entropy::{diehard, dieharder, nist, result::TestResult};
use std::thread;

// ── Configuration ─────────────────────────────────────────────────────────────

/// Bits consumed by the NIST SP 800-22 battery.
const NIST_N: usize = 1_000_000;

/// 32-bit words consumed by the DIEHARD and DIEHARDER batteries.
const DIEHARD_N: usize = 2_000_000;

// ── RNG descriptors ───────────────────────────────────────────────────────────

/// All test results for one RNG.
struct RngResults {
    name: &'static str,
    nist:      Vec<TestResult>,
    diehard:   Vec<TestResult>,
    dieharder: Vec<TestResult>,
}

/// A factory closure that produces a fresh RNG and runs the full battery,
/// returning an `RngResults`.  Stored as a boxed `Send` closure so it can be
/// shipped to a worker thread.
type RunFn = Box<dyn FnOnce() -> RngResults + Send + 'static>;

fn make_runs(quick: bool) -> Vec<RunFn> {
    // Each entry is a closure that owns its RNG and returns results.
    // All RNGs implement `Rng + Send + 'static` (no shared state).
    vec![
        Box::new(move || run_one("OsRng (/dev/urandom)",        OsRng::new(),                 quick)),
        Box::new(move || run_one("MT19937 (seed=19650218)",     Mt19937::new(19650218),        quick)),
        Box::new(move || run_one("Xorshift64 (seed=1)",         Xorshift64::new(1),            quick)),
        Box::new(move || run_one("Xorshift32 (seed=1)",         Xorshift32::new(1),            quick)),
        Box::new(move || run_one("C rand() (seed=1)",           CRand::new(1),                 quick)),
        Box::new(move || run_one("C mrand48 (seed=1)",          Rand48::new(1),                quick)),
        Box::new(move || run_one("LCG glibc rand (seed=1)",     Lcg32::glibc(),                quick)),
        Box::new(move || run_one("LCG MINSTD (seed=1)",         Lcg32::minstd(),               quick)),
        // BBS: p = 2³¹−1 (Mersenne prime, ≡ 3 mod 4), q = 4294967291 (largest prime < 2³², ≡ 3 mod 4).
        Box::new(move || run_one("BBS (p=2³¹−1, q=4294967291)", BlumBlumShub::new(2_147_483_647, 4_294_967_291, 1_234_567), quick)),
        // Blum-Micali: p = 2³¹−1, g = 7 (large-order element), seed = 42.
        Box::new(move || run_one("Blum-Micali (p=2³¹−1, g=7)", BlumMicali::new(2_147_483_647, 7, 42), quick)),
        Box::new(move || run_one("Constant (0xDEAD_DEAD)",      ConstantRng::new(0xDEAD_DEAD), quick)),
        Box::new(move || run_one("Counter (0,1,2,…)",           CounterRng::new(0),            quick)),
    ]
}

fn run_one<R: Rng>(name: &'static str, mut rng: R, quick: bool) -> RngResults {
    let nist      = nist::run_all(&mut rng, NIST_N);
    let diehard   = diehard::run_all(&mut rng, DIEHARD_N, quick);
    let dieharder = dieharder::run_all(&mut rng, DIEHARD_N, quick);
    RngResults { name, nist, diehard, dieharder }
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    let quick = std::env::args().any(|a| a == "--quick");

    let n_cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let runs = make_runs(quick);
    let n_rngs = runs.len();

    eprintln!(
        "Running {n_rngs} RNGs across {n_cores} core(s), {n_cores} threads at a time…"
    );

    // Process RNGs in batches of `n_cores` so we never have more active
    // threads than cores.  Results are re-ordered to match the original list.
    let banner = "=".repeat(72);
    let mut all_results: Vec<Option<RngResults>> = (0..n_rngs).map(|_| None).collect();

    let mut run_iter = runs.into_iter().enumerate();
    loop {
        // Collect up to n_cores tasks for this batch.
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
        print_rng_results(&r, &banner);
    }
}

fn print_rng_results(r: &RngResults, banner: &str) {
    println!("\n{banner}");
    println!("  {}", r.name);
    println!("{banner}");

    println!("\n  ── NIST SP 800-22 ({NIST_N} bits) ──");
    print_results(&r.nist);

    println!("\n  ── DIEHARD unique tests ({DIEHARD_N} words) ──");
    print_results(&r.diehard);

    println!("\n  ── DIEHARDER unique tests ({DIEHARD_N} words) ──");
    print_results(&r.dieharder);

    let all: Vec<&TestResult> = r.nist.iter().chain(&r.diehard).chain(&r.dieharder).collect();
    let pass = all.iter().filter(|r| r.passed()).count();
    let fail = all.iter().filter(|r| !r.passed() && !r.skipped()).count();
    let skip = all.iter().filter(|r| r.skipped()).count();
    println!("\n  Summary: {pass} PASS, {fail} FAIL, {skip} SKIP");
}

fn print_results(results: &[TestResult]) {
    for r in results {
        println!("  {r}");
    }
}
