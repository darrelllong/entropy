use entropy::research::marsaglia_tsang::{gorilla_all, GorillaBitResult};
use entropy::rng::{
    AesCtr, BsdRandom, CryptoCtrDrbg, Lcg32, LcgVariant, LinuxLibcRandom, Mt19937, Rand48, Rng,
    SystemVRand, WindowsDotNetRandom, WindowsMsvcRand, WindowsVb6Rnd, Xorshift32, Xorshift64,
};

const GORILLA_STREAM_WORDS: usize = (1 << 26) + 25;

struct Args {
    rng_filters: Vec<String>,
}

impl Args {
    fn parse() -> Self {
        let mut rng_filters = Vec::new();
        let argv: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0;
        while i < argv.len() {
            match argv[i].as_str() {
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
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
        Self { rng_filters }
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
        "Usage: gorilla [--rng <label>]\n\
         \n\
         Runs the Marsaglia–Tsang Gorilla test over all 32 bit positions of a\n\
         seeded deterministic RNG family.\n\
         \n\
         Example:\n\
           cargo run --release --bin gorilla -- --rng AES"
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

fn with_rng(mut rng: impl Rng) -> Vec<GorillaBitResult> {
    let words = rng.collect_u32s(GORILLA_STREAM_WORDS);
    gorilla_all(&words)
}

fn summarize(results: &[GorillaBitResult]) -> (f64, f64, usize, f64) {
    let mut min_p = 1.0f64;
    let mut max_p = 0.0f64;
    let mut worst_bit = 0usize;
    let mut worst_abs_z = 0.0f64;
    for result in results {
        if result.p_value < min_p {
            min_p = result.p_value;
        }
        if result.p_value > max_p {
            max_p = result.p_value;
        }
        let abs_z = result.z_score.abs();
        if abs_z > worst_abs_z {
            worst_abs_z = abs_z;
            worst_bit = result.bit_position;
        }
    }
    (min_p, max_p, worst_bit, worst_abs_z)
}

fn main() {
    let args = Args::parse();
    let cases: Vec<(&str, Vec<GorillaBitResult>)> = vec![
        ("MT19937", with_rng(Mt19937::new(19650218))),
        ("Xorshift32", with_rng(Xorshift32::new(1))),
        ("Xorshift64", with_rng(Xorshift64::new(1))),
        ("BAD Unix System V rand()", with_rng(SystemVRand::new(1))),
        ("BAD Unix System V mrand48()", with_rng(Rand48::new(1))),
        ("BAD Unix BSD random()", with_rng(BsdRandom::new(1))),
        ("BAD Unix Linux glibc rand()/random()", with_rng(LinuxLibcRandom::new(1))),
        ("BAD Windows CRT rand()", with_rng(WindowsMsvcRand::new(1))),
        ("BAD Windows VB6/VBA Rnd()", with_rng(WindowsVb6Rnd::new(1))),
        ("BAD Windows .NET Random(seed)", with_rng(WindowsDotNetRandom::new(1))),
        ("ANSI C sample LCG", with_rng(Lcg32::new(LcgVariant::AnsiC, 1))),
        ("LCG MINSTD", with_rng(Lcg32::new(LcgVariant::Minstd, 1))),
        ("AES-128-CTR", with_rng(AesCtr::new(&seed_material::<16>(1), 0))),
        (
            "cryptography::CtrDrbgAes256",
            with_rng(CryptoCtrDrbg::new(&seed_material::<48>(1))),
        ),
    ];

    println!(
        "{:<40} {:>9} {:>9} {:>9} {:>10}",
        "RNG", "min_p", "max_p", "worst_bit", "worst_|z|"
    );
    println!("{}", "-".repeat(84));

    let mut matched = 0usize;
    for (label, results) in cases {
        if !args.matches_rng(label) {
            continue;
        }
        matched += 1;
        let (min_p, max_p, worst_bit, worst_abs_z) = summarize(&results);
        println!(
            "{:<40} {:>9.6} {:>9.6} {:>9} {:>10.3}",
            label, min_p, max_p, worst_bit, worst_abs_z
        );
    }
    if matched == 0 {
        die("no RNG labels matched --rng filter");
    }
}
