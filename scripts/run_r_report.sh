#!/usr/bin/env bash
# Build R-REPORT.md by running scripts/r_rng_tests.R against every RNG.
# usage: ./scripts/run_r_report.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DUMP="$ROOT/target/release/dump_rng"
R_SCRIPT="$ROOT/scripts/r_rng_tests.R"
OUT="$ROOT/R-REPORT.md"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Fast generators get the full sample size; Dual_EC takes ~10 min/MB so use
# 1/5 of the standard size for it.
N_DEFAULT=5000000
N_DUAL_EC=1000000

# (label, rng_name)  — order kept stable for the report
RNGS=(
  # OS entropy
  "OsRng (/dev/urandom)|osrng"
  # Degenerate
  "ConstantRng|constant"
  "CounterRng|counter"
  # Historical libc / Windows / VBA
  "System V rand()|sysv_rand"
  "rand48 (mrand48)|rand48"
  "BSD random() / glibc random()|bsd_random"
  "FreeBSD rand_r() compat|bsd_rand_compat"
  "Windows MSVC rand()|windows_msvc_rand"
  "Windows VB6/VBA Rnd()|windows_vb6_rnd"
  "Windows .NET Random|windows_dotnet_random"
  # Classic LCGs
  "ANSI C LCG|ansi_c_lcg"
  "MINSTD (Park-Miller)|lcg_minstd"
  "Borland C++ LCG|borland_lcg"
  "MSVC LCG|msvc_lcg"
  # Quality non-cryptographic
  "MT19937|mt19937"
  "Xorshift32|xorshift32"
  "Xorshift64|xorshift64"
  "PCG32|pcg32"
  "PCG64|pcg64"
  "Xoshiro256|xoshiro256"
  "Xoroshiro128|xoroshiro128"
  "WyRand|wyrand"
  "SFC64|sfc64"
  "JSF64|jsf64"
  # Block-CTR cipher CSPRNGs
  "AES-128-CTR|aes_ctr"
  "Camellia-128-CTR|camellia_ctr"
  "Twofish-128-CTR|twofish_ctr"
  "Serpent-128-CTR|serpent_ctr"
  "SM4-CTR|sm4_ctr"
  "Grasshopper-256-CTR|grasshopper_ctr"
  "CAST-128-CTR|cast128_ctr"
  "SEED-CTR|seed_ctr"
  # Stream ciphers
  "Rabbit|rabbit"
  "Salsa20|salsa20"
  "Snow3G|snow3g"
  "ZUC-128|zuc128"
  # Hash- and HMAC-based DRBGs
  "ChaCha20|chacha20"
  "SpongeBob (SHA3-512)|spongebob"
  "Squidward (SHA-256)|squidward"
  "HmacDrbg|hmac_drbg"
  "HashDrbg|hash_drbg"
  "CtrDrbgAes256|crypto_ctr_drbg"
  # Backdoored negative control
  "Dual_EC_DRBG (P-256)|dual_ec_p256"
)

# ----- header ----------------------------------------------------------------
{
cat <<'EOF'
# R-REPORT — RNG tests via R's standard randomness packages

Each generator below was sampled into a binary stream of little-endian u32
words. R then read the stream, normalised it to U[0,1), and ran every test
exposed by the standard R RNG-testing packages (`randtests`, `randtoolbox`,
`tseries`) plus the goodness-of-fit and autocorrelation tests in `stats`.

Sample size: **5 000 000 u32 words** for every generator except
`Dual_EC_DRBG`, which uses **1 000 000** because each block requires two
P-256 scalar multiplications (≈ 10 min/MB). `randtests::rank.test` is O(n²)
Mann-Kendall; it is run on the first 5 000 samples to keep the per-RNG
runtime under a second.

The reject threshold is α = 0.001 (a single test passing/failing is not
proof; a generator that gets a few REJECTs from independent tests at α=0.001
is expected statistical noise across ~16 tests, but a generator that REJECTs
on most tests is broken).

`tseries::jarque.bera.test` tests Normality and is **expected to REJECT**
for a uniform stream; it is included as a sanity check.

The moment table reports the empirical raw moments E[U^k] for k = 1..10 and
the absolute error against the theoretical value 1/(k+1) for U(0,1).

Generated with `scripts/run_r_report.sh`; binary: `target/release/dump_rng`;
analysis: `scripts/r_rng_tests.R`.

R packages used:
EOF

Rscript -e '
pkgs <- c("randtests","randtoolbox","tseries","nortest","moments","stats")
for (p in pkgs) {
  v <- tryCatch(packageVersion(p), error=function(e) NA)
  cat(sprintf("- `%s` %s\n", p, ifelse(is.na(v),"(not installed)",as.character(v))))
}
' 2>/dev/null

cat <<EOF

R version: $(Rscript -e 'cat(paste0(R.version$major,".",R.version$minor))' 2>/dev/null)
Host: $(uname -srm)
Date: $(date "+%Y-%m-%d %H:%M:%S %Z")

---
EOF

# ----- per-RNG ----------------------------------------------------------------
for entry in "${RNGS[@]}"; do
  label="${entry%%|*}"
  name="${entry##*|}"
  if [[ "$name" == "dual_ec_p256" ]]; then
    n="$N_DUAL_EC"
  else
    n="$N_DEFAULT"
  fi
  echo "[run] $label ($name, n=$n)" >&2
  bin="$TMP/$name.bin"
  if ! "$DUMP" "$name" "$n" > "$bin"; then
    echo "[err] dump_rng $name failed" >&2
    continue
  fi
  if ! Rscript "$R_SCRIPT" "$bin" "$label"; then
    echo "[err] R analysis on $label failed" >&2
  fi
  rm -f "$bin"
done
} > "$OUT"

echo "[done] wrote $OUT" >&2
