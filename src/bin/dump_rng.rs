//! Stream raw u32 words from a named RNG to stdout.
//!
//! Usage:
//!     dump_rng <name> <count>     write `count` LE u32 words to stdout
//!     dump_rng --list             write the supported names, one per line
//!     dump_rng --help             usage message
//!
//! Each output u32 is little-endian; R reads it with
//!   `readBin(con, integer(), n=count, size=4, endian="little")`.
//! `count == 0` is legal and produces empty output.  Names match `pilot_rng`.

use std::io::{self, BufWriter, ErrorKind, Write};

use cryptography::{
    Camellia128, Cast128, Grasshopper, Rabbit, Salsa20, Seed as SeedCipher, Serpent128, Sm4,
    Snow3g, Twofish128, Zuc128,
};
use entropy::rng::{
    AesCtr, BlockCtrRng, BsdRandCompat, BsdRandom, ChaCha20Rng, ConstantRng, CounterRng,
    CryptoCtrDrbg, DualEcDrbg, HashDrbg, HmacDrbg, Jsf64, Lcg32, LcgVariant, LinuxLibcRandom,
    Mt19937, OsRng, Pcg32, Pcg64, Rand48, Rng, Sfc64, SpongeBob, Squidward, StreamRng,
    SystemVRand, WindowsDotNetRandom, WindowsMsvcRand, WindowsVb6Rnd, WyRand, Xoroshiro128,
    Xorshift32, Xorshift64, Xoshiro256,
};
use entropy::seed::{IV16, IV8, K16, K32};

/// Canonical list of supported names.  Tests and `--list` use this directly.
pub const NAMES: &[&str] = &[
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

fn dump<R: Rng>(mut rng: R, n: u64) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = BufWriter::with_capacity(1 << 20, stdout.lock());
    for _ in 0..n {
        out.write_all(&rng.next_u32().to_le_bytes())?;
    }
    out.flush()
}

fn dispatch(name: &str, n: u64) -> Result<io::Result<()>, ()> {
    let r = match name {
        "osrng" => dump(OsRng::new(), n),
        "mt19937" => dump(Mt19937::new(19650218), n),
        "xorshift64" => dump(Xorshift64::new(1), n),
        "xorshift32" => dump(Xorshift32::new(1), n),
        "sysv_rand" => dump(SystemVRand::new(1), n),
        "rand48" => dump(Rand48::new(1), n),
        "bsd_random" => dump(BsdRandom::new(1), n),
        "linux_glibc_random" => dump(LinuxLibcRandom::new(1), n),
        "bsd_rand_compat" => dump(BsdRandCompat::new(1), n),
        "windows_msvc_rand" => dump(WindowsMsvcRand::new(1), n),
        "windows_vb6_rnd" => dump(WindowsVb6Rnd::new(1), n),
        "windows_dotnet_random" => dump(WindowsDotNetRandom::new(1), n),
        "ansi_c_lcg" => dump(Lcg32::ansi_c(), n),
        "lcg_minstd" => dump(Lcg32::minstd(), n),
        "borland_lcg" => dump(Lcg32::new(LcgVariant::Borland, 1), n),
        "msvc_lcg" => dump(Lcg32::new(LcgVariant::Msvc, 1), n),
        "aes_ctr" => dump(AesCtr::with_nist_key(), n),
        "camellia_ctr" => dump(BlockCtrRng::new(Camellia128::new(&K16), 0), n),
        "twofish_ctr" => dump(BlockCtrRng::new(Twofish128::new(&K16), 0), n),
        "serpent_ctr" => dump(BlockCtrRng::new(Serpent128::new(&K16), 0), n),
        "sm4_ctr" => dump(BlockCtrRng::new(Sm4::new(&K16), 0), n),
        "grasshopper_ctr" => dump(BlockCtrRng::new(Grasshopper::new(&K32), 0), n),
        "cast128_ctr" => dump(BlockCtrRng::new(Cast128::new(&K16), 0), n),
        "seed_ctr" => dump(BlockCtrRng::new(SeedCipher::new(&K16), 0), n),
        "rabbit" => dump(StreamRng::new(Rabbit::new(&K16, &IV8)), n),
        "salsa20" => dump(StreamRng::new(Salsa20::new(&K32, &IV8)), n),
        "snow3g" => dump(StreamRng::new(Snow3g::new(&K16, &IV16)), n),
        "zuc128" => dump(StreamRng::new(Zuc128::new(&K16, &IV16)), n),
        "spongebob" => dump(SpongeBob::with_test_seed(), n),
        "squidward" => dump(Squidward::with_test_seed(), n),
        "pcg32" => dump(Pcg32::new(42, 54), n),
        "pcg64" => dump(Pcg64::new(1, 1), n),
        "xoshiro256" => dump(Xoshiro256::new(1, 2, 3, 4), n),
        "xoroshiro128" => dump(Xoroshiro128::new(1, 2), n),
        "wyrand" => dump(WyRand::new(42), n),
        "sfc64" => dump(Sfc64::new(1, 2, 3), n),
        "jsf64" => dump(Jsf64::new(0xdead_beef), n),
        "chacha20" => dump(ChaCha20Rng::from_os_rng(), n),
        "hmac_drbg" => dump(HmacDrbg::from_os_rng(), n),
        "hash_drbg" => dump(HashDrbg::from_os_rng(), n),
        "crypto_ctr_drbg" => dump(CryptoCtrDrbg::with_test_seed(), n),
        "dual_ec_p256" => dump(DualEcDrbg::p256(b"entropy-r-report"), n),
        "constant" => dump(ConstantRng::new(0xDEAD_DEAD), n),
        "counter" => dump(CounterRng::new(0), n),
        _ => return Err(()),
    };
    Ok(r)
}

fn print_usage(stream: &mut dyn Write) {
    let _ = writeln!(
        stream,
        "usage: dump_rng <name> <count>\n       dump_rng --list\n       dump_rng --help",
    );
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.iter().any(|a| a == "--help" || a == "-h") {
        print_usage(&mut io::stdout());
        return;
    }
    if argv.iter().any(|a| a == "--list") {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        for n in NAMES {
            let _ = writeln!(out, "{n}");
        }
        return;
    }
    if argv.len() != 2 {
        print_usage(&mut io::stderr());
        std::process::exit(2);
    }
    let name = argv[0].to_ascii_lowercase();
    let count: u64 = match argv[1].parse() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("dump_rng: count must be a non-negative integer, got {:?}", argv[1]);
            std::process::exit(2);
        }
    };

    match dispatch(&name, count) {
        Ok(Ok(())) => {}
        Ok(Err(e)) if e.kind() == ErrorKind::BrokenPipe => {
            // Consumer closed stdin early — normal for `head`/R early-stop.
        }
        Ok(Err(e)) => {
            eprintln!("dump_rng: write error: {e}");
            std::process::exit(1);
        }
        Err(()) => {
            eprintln!("dump_rng: unknown RNG: {name}");
            eprintln!("hint: dump_rng --list");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_unique_and_sorted_meaningfully() {
        let mut sorted = NAMES.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), NAMES.len(), "duplicate name in NAMES");
    }

    #[test]
    fn every_name_is_dispatchable_with_zero_words() {
        // count=0 exercises the dispatch & constructor for each generator
        // without paying any RNG output cost.  Validates that adding a name
        // to NAMES without a matching match-arm trips the test, and that
        // all constructors run without panicking.
        for &n in NAMES {
            match dispatch(n, 0) {
                Ok(Ok(())) => {}
                Ok(Err(e)) => panic!("dispatch({n}, 0) failed I/O: {e}"),
                Err(()) => panic!("dispatch({n}, 0) returned UnknownName"),
            }
        }
    }

    #[test]
    fn dispatch_rejects_unknown_name() {
        assert!(dispatch("not_a_real_rng", 0).is_err());
    }
}
