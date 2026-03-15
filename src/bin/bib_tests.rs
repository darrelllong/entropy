use entropy::research::{
    knuth::{gap_test, permutation_test, runs_above_below_median_test},
    pincus::approximate_entropy_profile,
};
use entropy::rng::{
    AesCtr, BsdRandom, CryptoCtrDrbg, Lcg32, LcgVariant, LinuxLibcRandom, Mt19937, Rand48, Rng,
    SystemVRand, WindowsDotNetRandom, WindowsMsvcRand, WindowsVb6Rnd, Xorshift32, Xorshift64,
};

struct Args {
    float_samples: usize,
    bit_samples: usize,
    rng_filters: Vec<String>,
}

impl Args {
    fn parse() -> Self {
        let mut float_samples = 200_000usize;
        let mut bit_samples = 1_000_000usize;
        let mut rng_filters = Vec::new();
        let argv: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0;
        while i < argv.len() {
            match argv[i].as_str() {
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--float-samples" => {
                    i += 1;
                    float_samples = argv
                        .get(i)
                        .unwrap_or_else(|| die("--float-samples requires an argument"))
                        .parse()
                        .unwrap_or_else(|_| die("invalid --float-samples value"));
                }
                "--bit-samples" => {
                    i += 1;
                    bit_samples = argv
                        .get(i)
                        .unwrap_or_else(|| die("--bit-samples requires an argument"))
                        .parse()
                        .unwrap_or_else(|_| die("invalid --bit-samples value"));
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
            float_samples,
            bit_samples,
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
        "Usage: bib_tests [--rng <label>] [--float-samples N] [--bit-samples N]\n\
         \n\
         Runs the newer BIB-backed research tests currently implemented in this\n\
         crate: Knuth permutation/gap/runs-median and a Pincus-style ApEn(m)\n\
         profile over m=2..6.\n\
         \n\
         Example:\n\
           cargo run --release --bin bib_tests -- --rng AES"
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

fn collect_case(mut rng: impl Rng, float_samples: usize, bit_samples: usize) -> (Vec<f64>, Vec<u8>) {
    let floats = rng.collect_f64s(float_samples);
    let bits = rng.collect_bits(bit_samples);
    (floats, bits)
}

fn print_case(label: &str, floats: &[f64], bits: &[u8]) {
    let permutation = permutation_test(floats, 5);
    let gap = gap_test(floats, 0.25, 0.5, 15);
    let runs = runs_above_below_median_test(floats);
    let apen = approximate_entropy_profile(bits, &[2, 3, 4, 5, 6]);

    println!("{label}");
    println!("  {permutation}");
    println!("  {gap}");
    println!("  {runs}");
    for point in apen {
        println!(
            "  [INFO] pincus::approximate_entropy_m{:02}          ApEn={:.6} (phi_m={:.6}, phi_m1={:.6})",
            point.m, point.approximate_entropy, point.phi_m, point.phi_m1
        );
    }
    println!();
}

fn main() {
    let args = Args::parse();
    let mut matched = 0usize;

    let cases: Vec<(&str, Box<dyn Fn() -> (Vec<f64>, Vec<u8>)>)> = vec![
        (
            "MT19937",
            Box::new(|| collect_case(Mt19937::new(19650218), args.float_samples, args.bit_samples)),
        ),
        (
            "Xorshift32",
            Box::new(|| collect_case(Xorshift32::new(1), args.float_samples, args.bit_samples)),
        ),
        (
            "Xorshift64",
            Box::new(|| collect_case(Xorshift64::new(1), args.float_samples, args.bit_samples)),
        ),
        (
            "BAD Unix System V rand()",
            Box::new(|| collect_case(SystemVRand::new(1), args.float_samples, args.bit_samples)),
        ),
        (
            "BAD Unix System V mrand48()",
            Box::new(|| collect_case(Rand48::new(1), args.float_samples, args.bit_samples)),
        ),
        (
            "BAD Unix BSD random()",
            Box::new(|| collect_case(BsdRandom::new(1), args.float_samples, args.bit_samples)),
        ),
        (
            "BAD Unix Linux glibc rand()/random()",
            Box::new(|| collect_case(LinuxLibcRandom::new(1), args.float_samples, args.bit_samples)),
        ),
        (
            "BAD Windows CRT rand()",
            Box::new(|| collect_case(WindowsMsvcRand::new(1), args.float_samples, args.bit_samples)),
        ),
        (
            "BAD Windows VB6/VBA Rnd()",
            Box::new(|| collect_case(WindowsVb6Rnd::new(1), args.float_samples, args.bit_samples)),
        ),
        (
            "BAD Windows .NET Random(seed)",
            Box::new(|| collect_case(WindowsDotNetRandom::new(1), args.float_samples, args.bit_samples)),
        ),
        (
            "ANSI C sample LCG",
            Box::new(|| {
                collect_case(
                    Lcg32::new(LcgVariant::AnsiC, 1),
                    args.float_samples,
                    args.bit_samples,
                )
            }),
        ),
        (
            "LCG MINSTD",
            Box::new(|| {
                collect_case(
                    Lcg32::new(LcgVariant::Minstd, 1),
                    args.float_samples,
                    args.bit_samples,
                )
            }),
        ),
        (
            "AES-128-CTR",
            Box::new(|| {
                let key = seed_material::<16>(1);
                collect_case(AesCtr::new(&key, 0), args.float_samples, args.bit_samples)
            }),
        ),
        (
            "cryptography::CtrDrbgAes256",
            Box::new(|| {
                let seed_bytes = seed_material::<48>(1);
                collect_case(CryptoCtrDrbg::new(&seed_bytes), args.float_samples, args.bit_samples)
            }),
        ),
    ];

    for (label, case) in cases {
        if !args.matches_rng(label) {
            continue;
        }
        matched += 1;
        let (floats, bits) = case();
        print_case(label, &floats, &bits);
    }

    if matched == 0 {
        die("no RNG labels matched --rng filter");
    }
}
