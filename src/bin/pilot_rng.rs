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
//! ```text
//!     pilot_rng <name>       time the named generator, print MW/s
//!     pilot_rng --list       write the supported names, one per line
//!     pilot_rng --help       usage message
//! ```
//!
//! Names match `dump_rng` (`pilot_rng --list` prints the same registry);
//! `tests/registry.rs` locks the two binaries together.
//!
//! `PILOT_RNG_WORDS` must be a positive integer when set; an invalid value is
//! a hard error rather than a silent fall-back, so a typo cannot quietly
//! benchmark the wrong workload size.

use std::hint::black_box;
use std::io::{self, Write};
use std::time::Instant;

use cryptography::{
    Camellia128, Cast128, Grasshopper, Rabbit, Salsa20, Seed as SeedCipher, Serpent128, Sm4,
    Snow3g, Twofish128, Zuc128,
};
use entropy::rng::{
    AesCtr, BlockCtrRng, BsdRandCompat, BsdRandom, ChaCha20Rng, ConstantRng, CounterRng,
    CryptoCtrDrbg, DualEcDrbg, HashDrbg, HmacDrbg, Jsf64, Lcg32, LcgVariant, LinuxLibcRandom,
    Mt19937, OsRng, Pcg32, Pcg64, Rand48, Rng, Sfc64, SpongeBob, Squidward, StreamRng, SystemVRand,
    WindowsDotNetRandom, WindowsMsvcRand, WindowsVb6Rnd, WyRand, Xoroshiro128, Xorshift32,
    Xorshift64, Xoshiro256,
};
use entropy::seed::{IV16, IV8, K16, K32};

/// Supported names, mirroring `dump_rng::NAMES` order.  `--list` prints this;
/// `tests/registry.rs` asserts it stays in lock-step with the dispatch below
/// and with `dump_rng`.
const NAMES: &[&str] = &[
    "osrng",
    "mt19937",
    "xorshift32",
    "xorshift64",
    "sysv_rand",
    "rand48",
    "bsd_random",
    "linux_glibc_random",
    "bsd_rand_compat",
    "windows_msvc_rand",
    "windows_vb6_rnd",
    "windows_dotnet_random",
    "ansi_c_lcg",
    "lcg_minstd",
    "borland_lcg",
    "msvc_lcg",
    "aes_ctr",
    "camellia_ctr",
    "twofish_ctr",
    "serpent_ctr",
    "sm4_ctr",
    "grasshopper_ctr",
    "cast128_ctr",
    "seed_ctr",
    "rabbit",
    "salsa20",
    "snow3g",
    "zuc128",
    "spongebob",
    "squidward",
    "pcg32",
    "pcg64",
    "xoshiro256",
    "xoroshiro128",
    "wyrand",
    "sfc64",
    "jsf64",
    "chacha20",
    "hmac_drbg",
    "hash_drbg",
    "crypto_ctr_drbg",
    "dual_ec_p256",
    "constant",
    "counter",
];

fn die(msg: &str) -> ! {
    eprintln!("pilot_rng: {msg}");
    std::process::exit(1);
}

fn print_usage(stream: &mut dyn Write) {
    let _ = writeln!(
        stream,
        "usage: pilot_rng <name>\n       pilot_rng --list\n       pilot_rng --help",
    );
}

fn workload_words() -> u64 {
    match std::env::var("PILOT_RNG_WORDS") {
        Err(std::env::VarError::NotPresent) => 10_000_000,
        Err(std::env::VarError::NotUnicode(_)) => die("PILOT_RNG_WORDS is not valid UTF-8"),
        Ok(s) => match s.parse::<u64>() {
            Ok(v) if v > 0 => v,
            _ => die(&format!(
                "invalid PILOT_RNG_WORDS {s:?} — must be a positive integer"
            )),
        },
    }
}

fn measure<R: Rng>(mut rng: R, n: u64) -> f64 {
    let t0 = Instant::now();
    let mut acc = 0u32;
    for _ in 0..n {
        acc ^= rng.next_u32();
    }
    black_box(acc);
    let elapsed = t0.elapsed().as_secs_f64();
    n as f64 / elapsed / 1_000_000.0 // MW/s
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    // Flags are only honored as the sole leading argument — `pilot_rng osrng
    // --list`-style trailing flags fall through to normal parsing and error.
    if argv.first().is_some_and(|a| a == "--help" || a == "-h") {
        print_usage(&mut io::stdout());
        return;
    }
    if argv.first().is_some_and(|a| a == "--list") {
        for n in NAMES {
            println!("{n}");
        }
        return;
    }
    let name = match argv.as_slice() {
        [name] => name.to_ascii_lowercase(),
        _ => {
            print_usage(&mut io::stderr());
            std::process::exit(1);
        }
    };

    let n = workload_words();

    let mw_s: f64 = match name.as_str() {
        "osrng" => measure(OsRng::new(), n),
        "mt19937" => measure(Mt19937::new(19650218), n),
        "xorshift64" => measure(Xorshift64::new(1), n),
        "xorshift32" => measure(Xorshift32::new(1), n),
        "sysv_rand" => measure(SystemVRand::new(1), n),
        "rand48" => measure(Rand48::new(1), n),
        "bsd_random" => measure(BsdRandom::new(1), n),
        "linux_glibc_random" => measure(LinuxLibcRandom::new(1), n),
        "bsd_rand_compat" => measure(BsdRandCompat::new(1), n),
        "windows_msvc_rand" => measure(WindowsMsvcRand::new(1), n),
        "windows_vb6_rnd" => measure(WindowsVb6Rnd::new(1), n),
        "windows_dotnet_random" => measure(WindowsDotNetRandom::new(1), n),
        "ansi_c_lcg" => measure(Lcg32::ansi_c(), n),
        "lcg_minstd" => measure(Lcg32::minstd(), n),
        "borland_lcg" => measure(Lcg32::new(LcgVariant::Borland, 1), n),
        // Byte-identical to "windows_msvc_rand" — see the note in dump_rng.rs.
        "msvc_lcg" => measure(Lcg32::new(LcgVariant::Msvc, 1), n),
        "aes_ctr" => measure(AesCtr::with_nist_key(), n),
        "camellia_ctr" => measure(BlockCtrRng::new(Camellia128::new(&K16), 0), n),
        "twofish_ctr" => measure(BlockCtrRng::new(Twofish128::new(&K16), 0), n),
        "serpent_ctr" => measure(BlockCtrRng::new(Serpent128::new(&K16), 0), n),
        "sm4_ctr" => measure(BlockCtrRng::new(Sm4::new(&K16), 0), n),
        "grasshopper_ctr" => measure(BlockCtrRng::new(Grasshopper::new(&K32), 0), n),
        "cast128_ctr" => measure(BlockCtrRng::new(Cast128::new(&K16), 0), n),
        "seed_ctr" => measure(BlockCtrRng::new(SeedCipher::new(&K16), 0), n),
        "rabbit" => measure(StreamRng::new(Rabbit::new(&K16, &IV8)), n),
        "salsa20" => measure(StreamRng::new(Salsa20::new(&K32, &IV8)), n),
        "snow3g" => measure(StreamRng::new(Snow3g::new(&K16, &IV16)), n),
        "zuc128" => measure(StreamRng::new(Zuc128::new(&K16, &IV16)), n),
        "spongebob" => measure(SpongeBob::with_test_seed(), n),
        "squidward" => measure(Squidward::with_test_seed(), n),
        "pcg32" => measure(Pcg32::new(42, 54), n),
        "pcg64" => measure(Pcg64::new(1, 1), n),
        "xoshiro256" => measure(Xoshiro256::new(1, 2, 3, 4), n),
        "xoroshiro128" => measure(Xoroshiro128::new(1, 2), n),
        "wyrand" => measure(WyRand::new(42), n),
        "sfc64" => measure(Sfc64::new(1, 2, 3), n),
        "jsf64" => measure(Jsf64::new(0xdead_beef), n),
        "chacha20" => measure(ChaCha20Rng::from_os_rng(), n),
        "hmac_drbg" => measure(HmacDrbg::from_os_rng(), n),
        "hash_drbg" => measure(HashDrbg::from_os_rng(), n),
        "crypto_ctr_drbg" => measure(CryptoCtrDrbg::with_test_seed(), n),
        "dual_ec_p256" => measure(DualEcDrbg::p256(b"entropy-r-report"), n),
        "constant" => measure(ConstantRng::new(0xDEAD_DEAD), n),
        "counter" => measure(CounterRng::new(0), n),
        other => {
            eprintln!("unknown RNG: {other}");
            eprintln!("hint: pilot_rng --list");
            std::process::exit(1);
        }
    };

    println!("{mw_s:.6}");
}
