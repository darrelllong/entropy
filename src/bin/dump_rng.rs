//! Stream raw u32 words from a named RNG to stdout.
//!
//! Usage:
//!     dump_rng <name> <count>
//!
//! Writes `count` little-endian u32 words (4 * count bytes) to stdout. R can
//! read the result with `readBin(con, integer(), n=count, size=4, endian="little")`.
//! Names match `pilot_rng`.

use std::io::{self, BufWriter, Write};

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

fn dump<R: Rng>(mut rng: R, n: u64) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = BufWriter::with_capacity(1 << 20, stdout.lock());
    for _ in 0..n {
        let w = rng.next_u32();
        out.write_all(&w.to_le_bytes())?;
    }
    out.flush()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| {
        eprintln!("usage: dump_rng <name> <count>");
        std::process::exit(1);
    });
    let n: u64 = args
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            eprintln!("usage: dump_rng <name> <count>");
            std::process::exit(1);
        });

    let r: io::Result<()> = match name.to_ascii_lowercase().as_str() {
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
        other => {
            eprintln!("unknown RNG: {other}");
            std::process::exit(1);
        }
    };

    if let Err(e) = r {
        // Broken pipe is normal when the consumer (R) has read enough.
        if e.kind() != io::ErrorKind::BrokenPipe {
            eprintln!("dump_rng: write error: {e}");
            std::process::exit(1);
        }
    }
}
