//! RNG throughput benchmark with adaptive 95% confidence intervals.
//!
//! For each RNG, generates batches of 10 M u32 words and records throughput
//! (MW/s) per batch.  Sampling continues until the 95% CI half-width is
//! ≤5% of the mean (or 200 batches are collected).  Reports mean, 95% CI,
//! and the number of batches needed.
//!
//! Run with:
//!   cargo run --release --bin bench_rngs

use entropy::rng::{
    AesCtr, BlumBlumShub, BlumMicali, BsdRandCompat, BsdRandom, ConstantRng,
    CounterRng, Lcg32, LinuxLibcRandom, Mt19937, OsRng, Rand48, Rng,
    SpongeBob, SystemVRand, WindowsDotNetRandom, WindowsMsvcRand,
    WindowsVb6Rnd, Xorshift32, Xorshift64,
};
use std::time::Instant;

const BATCH:      u64 = 10_000_000;   // words per timing sample
const WARMUP:     usize = 3;           // discarded warm-up batches
const MIN_SAMP:   usize = 5;           // minimum samples before CI check
const MAX_SAMP:   usize = 200;         // hard cap on samples
const TARGET_RCI: f64 = 0.05;         // target relative CI (half-width / mean)

// ── Two-tailed t critical value at 95% (α=0.025 per tail) ───────────────────
// Lookup for df = 1..30; fall back to normal approximation for df > 30.
fn t_crit(df: usize) -> f64 {
    const TABLE: [f64; 31] = [
        f64::NAN, // df=0 unused
        12.706, 4.303, 3.182, 2.776, 2.571, // 1-5
         2.447, 2.365, 2.306, 2.262, 2.228, // 6-10
         2.201, 2.179, 2.160, 2.145, 2.131, // 11-15
         2.120, 2.110, 2.101, 2.093, 2.086, // 16-20
         2.080, 2.074, 2.069, 2.064, 2.060, // 21-25
         2.056, 2.052, 2.048, 2.045, 2.042, // 26-30
    ];
    if df <= 30 { TABLE[df] } else { 1.960 }
}

// ── Bench one RNG ─────────────────────────────────────────────────────────────

fn bench<R: Rng>(name: &str, mut rng: R) {
    // Warm-up: discard these samples.
    for _ in 0..WARMUP {
        let mut acc = 0u32;
        for _ in 0..BATCH { acc ^= rng.next_u32(); }
        std::hint::black_box(acc);
    }

    let mut samples: Vec<f64> = Vec::with_capacity(32);

    loop {
        let t0 = Instant::now();
        let mut acc = 0u32;
        for _ in 0..BATCH { acc ^= rng.next_u32(); }
        std::hint::black_box(acc);
        let elapsed = t0.elapsed().as_secs_f64();
        let mw_s = BATCH as f64 / elapsed / 1_000_000.0;
        samples.push(mw_s);

        let n = samples.len();
        if n < MIN_SAMP { continue; }

        let mean = samples.iter().sum::<f64>() / n as f64;
        let var  = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
        let se   = (var / n as f64).sqrt();
        let half = t_crit(n - 1) * se;

        if half / mean <= TARGET_RCI || n >= MAX_SAMP {
            let pct  = 100.0 * half / mean;
            println!(
                "  {name:<36}  {mean:>7.1} ± {pct:.1}%  MW/s  [{:>7.1}..{:>7.1}]  MB/s  n={n}",
                (mean - half) * 4.0,
                (mean + half) * 4.0,
            );
            return;
        }

        if n >= MAX_SAMP { break; }
    }
}

fn main() {
    println!("\n  RNG throughput (10 M words/batch, 95% CI ≤5% or n=200)\n");
    println!(
        "  {:<36}  {:>16}  {:>24}  {}",
        "Generator", "mean ± rCI", "95% CI [lo..hi] MB/s", "n"
    );
    println!("  {}", "-".repeat(90));

    bench("OsRng (/dev/urandom)",         OsRng::new());
    bench("MT19937 (seed=19650218)",       Mt19937::new(19650218));
    bench("Xorshift64 (seed=1)",           Xorshift64::new(1));
    bench("Xorshift32 (seed=1)",           Xorshift32::new(1));
    bench("BAD Unix System V rand() (seed=1)", SystemVRand::new(1));
    bench("BAD Unix System V mrand48() (seed=1)", Rand48::new(1));
    bench("BAD Unix BSD random() TYPE_3 (seed=1)", BsdRandom::new(1));
    bench("BAD Unix Linux glibc rand()/random() (seed=1)", LinuxLibcRandom::new(1));
    bench("BAD Unix FreeBSD12 rand_r() compat (seed=1)", BsdRandCompat::new(1));
    bench("BAD Windows CRT rand() (seed=1)", WindowsMsvcRand::new(1));
    bench("BAD Windows VB6/VBA Rnd() (seed=1)", WindowsVb6Rnd::new(1));
    bench("BAD Windows .NET Random(seed=1) compat", WindowsDotNetRandom::new(1));
    bench("ANSI C sample LCG (seed=1)",    Lcg32::ansi_c());
    bench("LCG MINSTD (seed=1)",           Lcg32::minstd());
    bench("BBS (p=2³¹−1, q=4294967291)",  BlumBlumShub::new(2_147_483_647, 4_294_967_291, 1_234_567));
    bench("Blum-Micali (p=2³¹−1, g=7)",   BlumMicali::new(2_147_483_647, 7, 42));
    bench("AES-128-CTR (NIST key)",        AesCtr::with_nist_key());
    bench("SpongeBob (SHA3-512 chain, seed=00..3f)", SpongeBob::with_test_seed());
    bench("Constant (0xDEAD_DEAD)",        ConstantRng::new(0xDEAD_DEAD));
    bench("Counter (0,1,2,…)",             CounterRng::new(0));

    println!();
}
