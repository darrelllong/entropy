//! Webster–Tavares avalanche metrics (SAC dependence matrix and BIC) for the
//! seeded generators, treating each as a u64 → output mapping.
//!
//! `BICdegen` counts the avalanche-variable pairs whose correlation is
//! undefined because a variable never or always flips; `BICmean` and `BICmax`
//! cover only the other pairs and print NaN when there are none (every pair
//! of a GF(2)-linear generator such as Xorshift).

type Case<'a> = (&'a str, usize, Box<dyn Fn(u64) -> u64 + 'a>);

use entropy::research::webster_tavares::evaluate_u64;
use entropy::rng::{
    AesCtr, BsdRandom, CryptoCtrDrbg, Lcg32, LcgVariant, LinuxLibcRandom, Mt19937, Rand48, Rng,
    SystemVRand, WindowsDotNetRandom, WindowsMsvcRand, WindowsVb6Rnd, Xorshift32, Xorshift64,
};
use entropy::seed::seed_material;

struct Args {
    input_bits: usize,
    output_bits: usize,
    samples: usize,
    rng_filters: Vec<String>,
}

impl Args {
    fn parse() -> Self {
        let mut input_bits = 32usize;
        let mut output_bits = 32usize;
        let mut samples = 4096usize;
        let mut rng_filters = Vec::new();
        let argv: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0;
        while i < argv.len() {
            match argv[i].as_str() {
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--input-bits" => {
                    i += 1;
                    input_bits = argv
                        .get(i)
                        .unwrap_or_else(|| die("--input-bits requires an argument"))
                        .parse()
                        .unwrap_or_else(|_| die("invalid --input-bits value"));
                }
                "--output-bits" => {
                    i += 1;
                    output_bits = argv
                        .get(i)
                        .unwrap_or_else(|| die("--output-bits requires an argument"))
                        .parse()
                        .unwrap_or_else(|_| die("invalid --output-bits value"));
                }
                "--samples" => {
                    i += 1;
                    samples = argv
                        .get(i)
                        .unwrap_or_else(|| die("--samples requires an argument"))
                        .parse()
                        .unwrap_or_else(|_| die("invalid --samples value"));
                }
                "--rng" => {
                    i += 1;
                    rng_filters.push(
                        argv.get(i)
                            .unwrap_or_else(|| die("--rng requires an argument"))
                            .clone(),
                    );
                }
                other => die(&format!("unknown option '{other}'")),
            }
            i += 1;
        }

        // Range checks mirror the asserts in research::webster_tavares::evaluate_u64
        // so a bad flag dies with the flag's name instead of a library panic.
        if !(1..=64).contains(&input_bits) {
            die("--input-bits must be in 1..=64");
        }
        if !(1..=64).contains(&output_bits) {
            die("--output-bits must be in 1..=64");
        }
        if samples == 0 {
            die("--samples must be positive");
        }

        Self {
            input_bits,
            output_bits,
            samples,
            rng_filters,
        }
    }

    fn matches_rng(&self, label: &str) -> bool {
        let label = label.to_lowercase();
        self.rng_filters.is_empty()
            || self
                .rng_filters
                .iter()
                .any(|pat| label.contains(&pat.to_lowercase()))
    }
}

fn die(msg: &str) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(1);
}

fn print_usage() {
    eprintln!(
        "Usage: webster_tavares [--samples N] [--input-bits N] [--output-bits N] [--rng <label>]\n\
         \n\
         Runs Webster–Tavares strict-avalanche / avalanche-correlation sampling\n\
         on seeded deterministic RNG families.  BICdegen counts avalanche-variable\n\
         pairs with an undefined correlation (a variable that never or always\n\
         flips); BICmean/BICmax exclude them and print NaN if no pair is defined.\n\
         \n\
         Examples:\n\
           cargo run --release --bin webster_tavares\n\
           cargo run --release --bin webster_tavares -- --samples 2048 --rng Xorshift\n\
           cargo run --release --bin webster_tavares -- --input-bits 16 --output-bits 32"
    );
}

fn nonzero_u32(seed: u64) -> u32 {
    let x = seed as u32;
    if x == 0 {
        1
    } else {
        x
    }
}

fn nonzero_u64(seed: u64) -> u64 {
    if seed == 0 {
        1
    } else {
        seed
    }
}

fn next_u64_of(mut rng: impl Rng) -> u64 {
    rng.next_u64()
}

fn main() {
    let args = Args::parse();
    // (label, seed_bits, closure)
    // seed_bits: the number of low seed bits that can influence the
    // generator state — not the width of the constructor's parameter type.
    // A bit is dead when the cast or the constructor's masking discards it
    // outright; a many-to-one map that still lets every bit reach the state
    // (MINSTD's modular reduction, .NET's `abs`) does not count as dead,
    // because flipping such a bit still changes the state and the avalanche
    // measurement stays meaningful.  If args.input_bits > seed_bits the dead
    // input bits show up as non-avalanching, so a warning is printed.
    let cases: Vec<Case<'_>> = vec![
        (
            "MT19937",
            32,
            Box::new(|seed| next_u64_of(Mt19937::new(seed as u32))),
        ),
        (
            "Xorshift32",
            32,
            Box::new(|seed| next_u64_of(Xorshift32::new(nonzero_u32(seed)))),
        ),
        (
            "Xorshift64",
            64,
            Box::new(|seed| next_u64_of(Xorshift64::new(nonzero_u64(seed)))),
        ),
        (
            "BAD Unix System V rand()",
            32,
            Box::new(|seed| next_u64_of(SystemVRand::new(seed as u32))),
        ),
        (
            // srand48 semantics: the seed fills the high 32 bits of the
            // 48-bit state; seed bits above 32 never reach the state.
            "BAD Unix System V mrand48()",
            32,
            Box::new(|seed| next_u64_of(Rand48::new(seed))),
        ),
        (
            "BAD Unix BSD random()",
            32,
            Box::new(|seed| next_u64_of(BsdRandom::new(seed as u32))),
        ),
        (
            "BAD Unix Linux glibc rand()/random()",
            32,
            Box::new(|seed| next_u64_of(LinuxLibcRandom::new(seed as u32))),
        ),
        (
            "BAD Windows CRT rand()",
            32,
            Box::new(|seed| next_u64_of(WindowsMsvcRand::new(seed as u32))),
        ),
        (
            // `WindowsVb6Rnd::new` keeps only the low 24 bits of the seed.
            "BAD Windows VB6/VBA Rnd()",
            24,
            Box::new(|seed| next_u64_of(WindowsVb6Rnd::new(seed as u32))),
        ),
        (
            "BAD Windows .NET Random(seed)",
            32,
            Box::new(|seed| next_u64_of(WindowsDotNetRandom::new(seed as i32))),
        ),
        (
            // `Lcg32::new` masks the ANSI C seed to 31 bits (m = 2³¹).
            "ANSI C sample LCG",
            31,
            Box::new(|seed| next_u64_of(Lcg32::new(LcgVariant::AnsiC, seed))),
        ),
        (
            // MINSTD reduces the full 64-bit seed mod 2³¹ − 1: no bit is
            // discarded (2³¹ ≡ 1, so bit 31 folds onto bit 0 and so on).
            "LCG MINSTD",
            64,
            Box::new(|seed| next_u64_of(Lcg32::new(LcgVariant::Minstd, seed))),
        ),
        // The two cases below stretch a u64 through seed_material to fill the
        // cipher key, so the avalanche input is still only 64 bits wide —
        // seed_bits is 64, not the 128/384-bit key size.
        (
            "AES-128-CTR",
            64,
            Box::new(|seed| {
                let key = seed_material::<16>(seed);
                next_u64_of(AesCtr::new(&key, 0))
            }),
        ),
        (
            "cryptography::CtrDrbgAes256",
            64,
            Box::new(|seed| {
                let seed_bytes = seed_material::<48>(seed);
                next_u64_of(CryptoCtrDrbg::new(&seed_bytes))
            }),
        ),
    ];

    println!(
        "{:<40} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "RNG", "samples", "SACmean", "SACmax", "BICmean", "BICmax", "BICdegen"
    );
    println!("{}", "-".repeat(97));

    let mut matched = 0usize;
    for (label, seed_bits, case) in cases {
        if !args.matches_rng(label) {
            continue;
        }
        if args.input_bits > seed_bits {
            eprintln!(
                "warning: {label}: --input-bits {input} exceeds RNG seed width ({seed_bits} bits); \
                 bits {seed_bits}..{input} are silently truncated — results are misleading",
                input = args.input_bits,
            );
        }
        matched += 1;
        let report = evaluate_u64(args.input_bits, args.output_bits, args.samples, case);
        println!(
            "{:<40} {:>8} {:>8.4} {:>8.4} {:>8.4} {:>8.4} {:>8}",
            label,
            report.samples,
            report.mean_sac_bias,
            report.max_sac_bias,
            report.mean_bic_abs_corr,
            report.max_bic_abs_corr,
            report.bic_degenerate_pairs,
        );
    }

    if matched == 0 {
        die("no RNG labels matched --rng filter");
    }
}
