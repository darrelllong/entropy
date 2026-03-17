type Case<'a> = (&'a str, Box<dyn Fn() + 'a>);

use entropy::research::{
    practrand_fpf::{fpf_cross_result, fpf_platter_result, fpf_test, FpfConfig},
    testu01_hamming::{
        hamming_corr, hamming_corr_result, hamming_indep, hamming_indep_block_result,
        hamming_indep_main_result,
    },
};
use entropy::rng::{
    AesCtr, BsdRandom, CryptoCtrDrbg, Lcg32, LcgVariant, LinuxLibcRandom, Mt19937, Rand48, Rng,
    SystemVRand, WindowsDotNetRandom, WindowsMsvcRand, WindowsVb6Rnd, Xorshift32, Xorshift64,
};
use entropy::seed::seed_material;

struct Args {
    rng_filters: Vec<String>,
    hc_n: usize,
    hc_r: usize,
    hc_s: usize,
    hc_l: usize,
    hi_n: usize,
    hi_r: usize,
    hi_s: usize,
    hi_l: usize,
    hi_d: usize,
    fpf_bits: usize,
}

impl Args {
    fn parse() -> Self {
        let mut out = Self {
            rng_filters: Vec::new(),
            hc_n: 500_000,
            hc_r: 20,
            hc_s: 10,
            hc_l: 300,
            hi_n: 500_000,
            hi_r: 20,
            hi_s: 10,
            hi_l: 300,
            hi_d: 1,
            fpf_bits: 1 << 27,
        };
        let argv: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0usize;
        while i < argv.len() {
            match argv[i].as_str() {
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--rng" => {
                    i += 1;
                    out.rng_filters.push(
                        argv.get(i)
                            .unwrap_or_else(|| die("--rng requires an argument"))
                            .clone(),
                    );
                }
                "--hc-n" => {
                    i += 1;
                    out.hc_n = parse_usize(argv.get(i), "--hc-n");
                }
                "--hc-r" => {
                    i += 1;
                    out.hc_r = parse_usize(argv.get(i), "--hc-r");
                }
                "--hc-s" => {
                    i += 1;
                    out.hc_s = parse_usize(argv.get(i), "--hc-s");
                }
                "--hc-l" => {
                    i += 1;
                    out.hc_l = parse_usize(argv.get(i), "--hc-l");
                }
                "--hi-n" => {
                    i += 1;
                    out.hi_n = parse_usize(argv.get(i), "--hi-n");
                }
                "--hi-r" => {
                    i += 1;
                    out.hi_r = parse_usize(argv.get(i), "--hi-r");
                }
                "--hi-s" => {
                    i += 1;
                    out.hi_s = parse_usize(argv.get(i), "--hi-s");
                }
                "--hi-l" => {
                    i += 1;
                    out.hi_l = parse_usize(argv.get(i), "--hi-l");
                }
                "--hi-d" => {
                    i += 1;
                    out.hi_d = parse_usize(argv.get(i), "--hi-d");
                }
                "--fpf-bits" => {
                    i += 1;
                    out.fpf_bits = parse_usize(argv.get(i), "--fpf-bits");
                }
                other => die(&format!("unknown option '{other}'")),
            }
            i += 1;
        }
        out
    }

    fn matches_rng(&self, label: &str) -> bool {
        self.rng_filters.is_empty() || self.rng_filters.iter().any(|pat| label.contains(pat))
    }
}

fn parse_usize(v: Option<&String>, flag: &str) -> usize {
    v.unwrap_or_else(|| die(&format!("{flag} requires an argument")))
        .parse()
        .unwrap_or_else(|_| die(&format!("invalid value for {flag}")))
}

fn die(msg: &str) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(1);
}

fn print_usage() {
    eprintln!(
        "Usage: upstream_tests [--rng <label>] [--hc-n N] [--hi-n N] [--fpf-bits N]\n\
         \n\
         Runs one honest TestU01 bit-string slice and one honest PractRand slice:\n\
         - TestU01 sstring_HammingCorr\n\
         - TestU01 sstring_HammingIndep\n\
         - PractRand FPF(4,14,6) core\n\
         \n\
         Defaults are moderate-size runs suitable for development checks.\n\
         Example:\n\
           cargo run --release --bin upstream_tests -- --rng AES"
    );
}

fn run_case(label: &str, mut rng: impl Rng, args: &Args) {
    println!("{label}");

    let hc = hamming_corr(&mut rng, args.hc_n, args.hc_r, args.hc_s, args.hc_l);
    println!("  {}", hamming_corr_result(&hc));

    let hi = hamming_indep(
        &mut rng, args.hi_n, args.hi_r, args.hi_s, args.hi_l, args.hi_d,
    );
    println!("  {}", hamming_indep_main_result(&hi));
    for k in 1..=args.hi_d {
        println!("  {}", hamming_indep_block_result(&hi, k));
    }

    let fpf = fpf_test(&mut rng, args.fpf_bits, &FpfConfig::default());
    println!("  {}", fpf_cross_result(&fpf));
    for platter in fpf.platter_results.iter().take(8) {
        println!("  {}", fpf_platter_result(platter, &fpf));
    }
    if fpf.platter_results.len() > 8 {
        println!(
            "  [INFO] practrand::fpf_more                   {} additional platter results omitted",
            fpf.platter_results.len() - 8
        );
    }
    println!();
}

fn main() {
    let args = Args::parse();
    let cases: Vec<Case<'_>> = vec![
        (
            "MT19937",
            Box::new(|| run_case("MT19937", Mt19937::new(19650218), &args)),
        ),
        (
            "Xorshift32",
            Box::new(|| run_case("Xorshift32", Xorshift32::new(1), &args)),
        ),
        (
            "Xorshift64",
            Box::new(|| run_case("Xorshift64", Xorshift64::new(1), &args)),
        ),
        (
            "BAD Unix System V rand()",
            Box::new(|| run_case("BAD Unix System V rand()", SystemVRand::new(1), &args)),
        ),
        (
            "BAD Unix System V mrand48()",
            Box::new(|| run_case("BAD Unix System V mrand48()", Rand48::new(1), &args)),
        ),
        (
            "BAD Unix BSD random()",
            Box::new(|| run_case("BAD Unix BSD random()", BsdRandom::new(1), &args)),
        ),
        (
            "BAD Unix Linux glibc rand()/random()",
            Box::new(|| {
                run_case(
                    "BAD Unix Linux glibc rand()/random()",
                    LinuxLibcRandom::new(1),
                    &args,
                )
            }),
        ),
        (
            "BAD Windows CRT rand()",
            Box::new(|| run_case("BAD Windows CRT rand()", WindowsMsvcRand::new(1), &args)),
        ),
        (
            "BAD Windows VB6/VBA Rnd()",
            Box::new(|| run_case("BAD Windows VB6/VBA Rnd()", WindowsVb6Rnd::new(1), &args)),
        ),
        (
            "BAD Windows .NET Random(seed)",
            Box::new(|| {
                run_case(
                    "BAD Windows .NET Random(seed)",
                    WindowsDotNetRandom::new(1),
                    &args,
                )
            }),
        ),
        (
            "ANSI C sample LCG",
            Box::new(|| run_case("ANSI C sample LCG", Lcg32::new(LcgVariant::AnsiC, 1), &args)),
        ),
        (
            "LCG MINSTD",
            Box::new(|| run_case("LCG MINSTD", Lcg32::new(LcgVariant::Minstd, 1), &args)),
        ),
        (
            "AES-128-CTR",
            Box::new(|| {
                run_case(
                    "AES-128-CTR",
                    AesCtr::new(&seed_material::<16>(1), 0),
                    &args,
                )
            }),
        ),
        (
            "cryptography::CtrDrbgAes256",
            Box::new(|| {
                run_case(
                    "cryptography::CtrDrbgAes256",
                    CryptoCtrDrbg::new(&seed_material::<48>(1)),
                    &args,
                )
            }),
        ),
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
