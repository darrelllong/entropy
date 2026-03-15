use entropy::research::testu01_lz::{
    lempel_ziv_ks_result, lempel_ziv_sum_result, lempel_ziv_summary,
};
use entropy::rng::{
    AesCtr, BsdRandom, CryptoCtrDrbg, Lcg32, LcgVariant, LinuxLibcRandom, Mt19937, Rand48, Rng,
    SystemVRand, WindowsDotNetRandom, WindowsMsvcRand, WindowsVb6Rnd, Xorshift32, Xorshift64,
};

struct Args {
    replications: usize,
    k: usize,
    r: usize,
    s: usize,
    rng_filters: Vec<String>,
}

impl Args {
    fn parse() -> Self {
        let mut replications = 10usize;
        let mut k = 25usize;
        let mut r = 0usize;
        let mut s = 30usize;
        let mut rng_filters = Vec::new();
        let argv: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0;
        while i < argv.len() {
            match argv[i].as_str() {
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--replications" => {
                    i += 1;
                    replications = argv
                        .get(i)
                        .unwrap_or_else(|| die("--replications requires an argument"))
                        .parse()
                        .unwrap_or_else(|_| die("invalid --replications value"));
                }
                "--k" => {
                    i += 1;
                    k = argv
                        .get(i)
                        .unwrap_or_else(|| die("--k requires an argument"))
                        .parse()
                        .unwrap_or_else(|_| die("invalid --k value"));
                }
                "--r" => {
                    i += 1;
                    r = argv
                        .get(i)
                        .unwrap_or_else(|| die("--r requires an argument"))
                        .parse()
                        .unwrap_or_else(|_| die("invalid --r value"));
                }
                "--s" => {
                    i += 1;
                    s = argv
                        .get(i)
                        .unwrap_or_else(|| die("--s requires an argument"))
                        .parse()
                        .unwrap_or_else(|_| die("invalid --s value"));
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
        Self { replications, k, r, s, rng_filters }
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
        "Usage: testu01_lz [--rng <label>] [--replications N] [--k K] [--r R] [--s S]\n\
         \n\
         Runs the exact TestU01 1.2.3 Lempel-Ziv core statistic using the\n\
         official empirical calibration tables from scomp.c.\n\
         \n\
         Example:\n\
           cargo run --release --bin testu01_lz -- --rng AES\n\
           cargo run --release --bin testu01_lz -- --rng MT19937 --k 27"
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

fn run_case(label: &str, mut rng: impl Rng, args: &Args) {
    let (reps, summary) = lempel_ziv_summary(&mut rng, args.replications, args.k, args.r, args.s);
    println!("{label}");
    println!("  {}", lempel_ziv_sum_result(&summary));
    println!("  {}", lempel_ziv_ks_result(&summary));
    for (i, rep) in reps.iter().enumerate() {
        println!(
            "  [INFO] testu01::lzw_rep{:02}                    W={} z={:.4}",
            i + 1,
            rep.phrase_count,
            rep.z_score
        );
    }
    println!();
}

fn main() {
    let args = Args::parse();

    let cases: Vec<(&str, Box<dyn Fn()>)> = vec![
        ("MT19937", Box::new(|| run_case("MT19937", Mt19937::new(19650218), &args))),
        ("Xorshift32", Box::new(|| run_case("Xorshift32", Xorshift32::new(1), &args))),
        ("Xorshift64", Box::new(|| run_case("Xorshift64", Xorshift64::new(1), &args))),
        ("BAD Unix System V rand()", Box::new(|| run_case("BAD Unix System V rand()", SystemVRand::new(1), &args))),
        ("BAD Unix System V mrand48()", Box::new(|| run_case("BAD Unix System V mrand48()", Rand48::new(1), &args))),
        ("BAD Unix BSD random()", Box::new(|| run_case("BAD Unix BSD random()", BsdRandom::new(1), &args))),
        ("BAD Unix Linux glibc rand()/random()", Box::new(|| run_case("BAD Unix Linux glibc rand()/random()", LinuxLibcRandom::new(1), &args))),
        ("BAD Windows CRT rand()", Box::new(|| run_case("BAD Windows CRT rand()", WindowsMsvcRand::new(1), &args))),
        ("BAD Windows VB6/VBA Rnd()", Box::new(|| run_case("BAD Windows VB6/VBA Rnd()", WindowsVb6Rnd::new(1), &args))),
        ("BAD Windows .NET Random(seed)", Box::new(|| run_case("BAD Windows .NET Random(seed)", WindowsDotNetRandom::new(1), &args))),
        ("ANSI C sample LCG", Box::new(|| run_case("ANSI C sample LCG", Lcg32::new(LcgVariant::AnsiC, 1), &args))),
        ("LCG MINSTD", Box::new(|| run_case("LCG MINSTD", Lcg32::new(LcgVariant::Minstd, 1), &args))),
        ("AES-128-CTR", Box::new(|| run_case("AES-128-CTR", AesCtr::new(&seed_material::<16>(1), 0), &args))),
        ("cryptography::CtrDrbgAes256", Box::new(|| run_case("cryptography::CtrDrbgAes256", CryptoCtrDrbg::new(&seed_material::<48>(1)), &args))),
    ];

    let mut matched = 0usize;
    for (label, case) in cases {
        if !args.matches_rng(label) {
            continue;
        }
        matched += 1;
        case();
    }
    if matched == 0 {
        die("no RNG labels matched --rng filter");
    }
}
