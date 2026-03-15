use entropy::research::webster_tavares::evaluate_u64;
use entropy::rng::{
    AesCtr, BsdRandom, CryptoCtrDrbg, Lcg32, LcgVariant, LinuxLibcRandom, Mt19937, Rand48, Rng,
    SystemVRand, WindowsDotNetRandom, WindowsMsvcRand, WindowsVb6Rnd, Xorshift32, Xorshift64,
};

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

        Self {
            input_bits,
            output_bits,
            samples,
            rng_filters,
        }
    }

    fn matches_rng(&self, label: &str) -> bool {
        self.rng_filters.is_empty() || self.rng_filters.iter().any(|pat| label.contains(pat))
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
         on seeded deterministic RNG families.\n\
         \n\
         Examples:\n\
           cargo run --release --bin webster_tavares\n\
           cargo run --release --bin webster_tavares -- --samples 2048 --rng Xorshift\n\
           cargo run --release --bin webster_tavares -- --input-bits 16 --output-bits 32"
    );
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn seed_material<const N: usize>(seed: u64) -> [u8; N] {
    let mut state = seed ^ 0xa076_1d64_78bd_642f;
    let mut out = [0u8; N];
    let mut pos = 0usize;
    while pos < N {
        let word = splitmix64(&mut state).to_be_bytes();
        let take = (N - pos).min(8);
        out[pos..pos + take].copy_from_slice(&word[..take]);
        pos += take;
    }
    out
}

fn nonzero_u32(seed: u64) -> u32 {
    let x = seed as u32;
    if x == 0 { 1 } else { x }
}

fn nonzero_u64(seed: u64) -> u64 {
    if seed == 0 { 1 } else { seed }
}

fn next_u64_of(mut rng: impl Rng) -> u64 {
    rng.next_u64()
}

fn main() {
    let args = Args::parse();
    let cases: Vec<(&str, Box<dyn Fn(u64) -> u64>)> = vec![
        ("MT19937", Box::new(|seed| next_u64_of(Mt19937::new(seed as u32)))),
        ("Xorshift32", Box::new(|seed| next_u64_of(Xorshift32::new(nonzero_u32(seed))))),
        ("Xorshift64", Box::new(|seed| next_u64_of(Xorshift64::new(nonzero_u64(seed))))),
        ("BAD Unix System V rand()", Box::new(|seed| next_u64_of(SystemVRand::new(seed as u32)))),
        ("BAD Unix System V mrand48()", Box::new(|seed| next_u64_of(Rand48::new(seed)))),
        ("BAD Unix BSD random()", Box::new(|seed| next_u64_of(BsdRandom::new(seed as u32)))),
        ("BAD Unix Linux glibc rand()/random()", Box::new(|seed| next_u64_of(LinuxLibcRandom::new(seed as u32)))),
        ("BAD Windows CRT rand()", Box::new(|seed| next_u64_of(WindowsMsvcRand::new(seed as u32)))),
        ("BAD Windows VB6/VBA Rnd()", Box::new(|seed| next_u64_of(WindowsVb6Rnd::new(seed as u32)))),
        ("BAD Windows .NET Random(seed)", Box::new(|seed| next_u64_of(WindowsDotNetRandom::new(seed as i32)))),
        ("ANSI C sample LCG", Box::new(|seed| next_u64_of(Lcg32::new(LcgVariant::AnsiC, seed)))),
        ("LCG MINSTD", Box::new(|seed| next_u64_of(Lcg32::new(LcgVariant::Minstd, seed)))),
        (
            "AES-128-CTR",
            Box::new(|seed| {
                let key = seed_material::<16>(seed);
                next_u64_of(AesCtr::new(&key, 0))
            }),
        ),
        (
            "cryptography::CtrDrbgAes256",
            Box::new(|seed| {
                let seed_bytes = seed_material::<48>(seed);
                next_u64_of(CryptoCtrDrbg::new(&seed_bytes))
            }),
        ),
    ];

    println!(
        "{:<40} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "RNG", "samples", "SACmean", "SACmax", "BICmean", "BICmax"
    );
    println!("{}", "-".repeat(88));

    let mut matched = 0usize;
    for (label, case) in cases {
        if !args.matches_rng(label) {
            continue;
        }
        matched += 1;
        let report = evaluate_u64(args.input_bits, args.output_bits, args.samples, case);
        println!(
            "{:<40} {:>8} {:>8.4} {:>8.4} {:>8.4} {:>8.4}",
            label,
            report.samples,
            report.mean_sac_bias,
            report.max_sac_bias,
            report.mean_bic_abs_corr,
            report.max_bic_abs_corr,
        );
    }

    if matched == 0 {
        die("no RNG labels matched --rng filter");
    }
}
