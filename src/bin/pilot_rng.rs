//! Single-shot RNG throughput probe for pilot-bench.
//!
//! Generates `PILOT_RNG_WORDS` (default 10 000 000) 32-bit words from the
//! named generator, times the loop, and prints a single line:
//!
//!     <MW/s>
//!
//! to stdout.  pilot-bench calls this binary repeatedly until it has collected
//! enough readings to produce a 95% confidence interval.
//!
//! Usage:
//!   pilot_rng <name>
//!
//! Names: osrng  mt19937  xorshift64  xorshift32  sysv_rand  rand48
//!        bsd_random  linux_glibc_random  bsd_rand_compat
//!        windows_msvc_rand  windows_vb6_rnd  windows_dotnet_random
//!        ansi_c_lcg  lcg_minstd  bbs  blum_micali  aes_ctr
//!        crypto_ctr_drbg  constant  counter

use std::hint::black_box;
use std::time::Instant;

use entropy::rng::{
    AesCtr, BlumBlumShub, BlumMicali, BsdRandCompat, BsdRandom, ConstantRng,
    CounterRng, CryptoCtrDrbg, Lcg32, LinuxLibcRandom, Mt19937, OsRng, Rand48,
    Rng, SystemVRand, WindowsDotNetRandom, WindowsMsvcRand, WindowsVb6Rnd,
    Xorshift32, Xorshift64,
};

fn workload_words() -> u64 {
    std::env::var("PILOT_RNG_WORDS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|&v| v > 0)
        .unwrap_or(10_000_000)
}

fn measure<R: Rng>(mut rng: R, n: u64) -> f64 {
    let t0 = Instant::now();
    let mut acc = 0u32;
    for _ in 0..n {
        acc ^= rng.next_u32();
    }
    black_box(acc);
    let elapsed = t0.elapsed().as_secs_f64();
    n as f64 / elapsed / 1_000_000.0   // MW/s
}

fn main() {
    let name = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: pilot_rng <name>");
        std::process::exit(1);
    });

    let n = workload_words();

    let mw_s: f64 = match name.to_ascii_lowercase().as_str() {
        "osrng" =>
            measure(OsRng::new(), n),
        "mt19937" =>
            measure(Mt19937::new(19650218), n),
        "xorshift64" =>
            measure(Xorshift64::new(1), n),
        "xorshift32" =>
            measure(Xorshift32::new(1), n),
        "sysv_rand" =>
            measure(SystemVRand::new(1), n),
        "rand48" =>
            measure(Rand48::new(1), n),
        "bsd_random" =>
            measure(BsdRandom::new(1), n),
        "linux_glibc_random" =>
            measure(LinuxLibcRandom::new(1), n),
        "bsd_rand_compat" =>
            measure(BsdRandCompat::new(1), n),
        "windows_msvc_rand" =>
            measure(WindowsMsvcRand::new(1), n),
        "windows_vb6_rnd" =>
            measure(WindowsVb6Rnd::new(1), n),
        "windows_dotnet_random" =>
            measure(WindowsDotNetRandom::new(1), n),
        "ansi_c_lcg" =>
            measure(Lcg32::ansi_c(), n),
        "lcg_minstd" =>
            measure(Lcg32::minstd(), n),
        "bbs" =>
            measure(BlumBlumShub::new(2_147_483_647, 4_294_967_291, 1_234_567), n),
        "blum_micali" =>
            measure(BlumMicali::new(2_147_483_647, 7, 42), n),
        "aes_ctr" =>
            measure(AesCtr::with_nist_key(), n),
        "crypto_ctr_drbg" =>
            measure(CryptoCtrDrbg::with_test_seed(), n),
        "constant" =>
            measure(ConstantRng::new(0xDEAD_DEAD), n),
        "counter" =>
            measure(CounterRng::new(0), n),
        other => {
            eprintln!("unknown RNG: {other}");
            std::process::exit(1);
        }
    };

    println!("{mw_s:.6}");
}
