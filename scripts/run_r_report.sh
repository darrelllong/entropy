#!/usr/bin/env bash
# Build R-REPORT.md by running scripts/r_rng_tests.R against every RNG.
# usage: ./scripts/run_r_report.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
DUMP="$TARGET/release/dump_rng"
R_SCRIPT="$ROOT/scripts/r_rng_tests.R"
OUT="$ROOT/R-REPORT.md"

# Always build: a binary left from other sources or features would be
# reported as this checkout's.
(cd "$ROOT" && cargo build --quiet --release --bin dump_rng)
if [[ ! -x "$DUMP" ]]; then
  echo "error: $DUMP was not built" >&2
  exit 1
fi
if [[ ! -r "$R_SCRIPT" ]]; then
  echo "error: $R_SCRIPT missing or unreadable" >&2
  exit 1
fi
if ! command -v Rscript >/dev/null 2>&1; then
  echo "error: Rscript not on PATH" >&2
  exit 1
fi
# Verify every R package upfront — one clear error beats 44 per-RNG failures.
MISSING_PKGS=$(Rscript -e '
  pkgs <- c("moments", "randtests", "randtoolbox", "tseries")
  missing <- pkgs[!vapply(pkgs, requireNamespace, logical(1), quietly = TRUE)]
  cat(missing, sep = " ")' 2>/dev/null)
if [[ -n "$MISSING_PKGS" ]]; then
  echo "error: missing R package(s): $MISSING_PKGS" >&2
  echo "  install.packages(c($(echo "$MISSING_PKGS" | sed 's/[^ ]*/"&"/g; s/ /, /g')))" >&2
  exit 1
fi

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
  "BSD random()|bsd_random"
  "Linux glibc rand()/random()|linux_glibc_random"
  "FreeBSD rand_r() compat|bsd_rand_compat"
  "Windows MSVC rand()|windows_msvc_rand"
  "Windows VB6/VBA Rnd()|windows_vb6_rnd"
  "Windows .NET Random|windows_dotnet_random"
  # Classic LCGs
  "ANSI C LCG|ansi_c_lcg"
  "MINSTD (Park-Miller)|lcg_minstd"
  "Borland C++ LCG|borland_lcg"
  # NOTE: msvc_lcg is byte-identical to windows_msvc_rand (same stream under
  # two registry names); both rows are kept for registry completeness, at the
  # cost of one redundant battery slot.
  "MSVC LCG|msvc_lcg"
  # Quality non-cryptographic
  "MT19937|mt19937"
  "Xorshift32|xorshift32"
  "Xorshift64|xorshift64"
  "PCG32|pcg32"
  "PCG64|pcg64"
  "Xoshiro256|xoshiro256"
  "Xoroshiro128|xoroshiro128"
  "SFC64|sfc64"
  "JSF64|jsf64"
  # Block-CTR cipher CSPRNGs
  "AES-128-CTR|aes_ctr"
  "Camellia-128-CTR|camellia_ctr"
  "Twofish-128-CTR|twofish_ctr"
  "Serpent-128-CTR|serpent_ctr"
  "SM4-CTR|sm4_ctr"
  "Grasshopper-CTR|grasshopper_ctr"
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
words. R then read the stream, normalised it to U[0,1), and ran the
randomness tests exposed by the standard R RNG-testing packages that apply
to a value-level uniform stream — `randtests` (runs, Bartels rank,
Cox-Stuart, difference-sign, turning-point, Mann-Kendall rank),
`randtoolbox` (freq, gap, serial, poker, order) — plus `tseries`
(runs, Jarque-Bera), the goodness-of-fit and autocorrelation tests in
`stats` (KS, χ²(256), Ljung-Box), and `moments` for the raw-moment
diagnostics.  (`randtoolbox::coll.test` was not run.)

Sample size: **5 000 000 u32 words** for every generator except
`Dual_EC_DRBG`, which uses **1 000 000** because each block requires three
P-256 scalar multiplications (≈ 10 min/MB). `randtests::rank.test` is O(n²)
Mann-Kendall; it is run on the first 5 000 samples to keep the per-RNG
runtime under a second.

Each result is **pass** (p ≥ 0.001), **fail** (p < 0.001) or **invalid**,
with its reason, when the test raised an error or gave no single finite
p-value in [0, 1].  A few failures among ~20 tests are expected noise; a
generator failing most of them is broken.  An invalid result says nothing
about the generator either way.

`tseries::jarque.bera.test` tests Normality and is **expected to fail** for a
uniform stream; it is included as a sanity check.

The moment table reports the empirical raw moments E[U^k] for k = 1..10 and
the absolute error against the theoretical value 1/(k+1) for U(0,1).

Generated with `scripts/run_r_report.sh`; binary: `dump_rng`;
analysis: `scripts/r_rng_tests.R`.

R packages used:
EOF

Rscript -e '
pkgs <- c("randtests","randtoolbox","tseries","moments","stats")
for (p in pkgs) {
  v <- tryCatch(packageVersion(p), error=function(e) NA)
  cat(sprintf("- `%s` %s\n", p, ifelse(is.na(v),"(not installed)",as.character(v))))
}
' 2>/dev/null

cat <<EOF

R version: $(Rscript -e 'cat(paste0(R.version$major,".",R.version$minor))' 2>/dev/null)

\`\`\`
$("$ROOT/scripts/provenance.sh" default "$DUMP")
\`\`\`

---
EOF

# ----- per-RNG ----------------------------------------------------------------
FAILED=()
INVALID=()
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
    FAILED+=("$name")
    continue
  fi
  expected=$(( n * 4 ))
  # Use stat directly (no pipeline) so set -eo pipefail can't abort here
  # on a transient error.  Linux/BSD stat have different flags.
  if actual=$(stat -f%z "$bin" 2>/dev/null) || actual=$(stat -c%s "$bin" 2>/dev/null); then
    :
  else
    echo "[err] cannot stat $bin" >&2
    FAILED+=("$name")
    continue
  fi
  if [[ "$actual" != "$expected" ]]; then
    echo "[err] $name: expected $expected bytes, got $actual" >&2
    FAILED+=("$name")
    continue
  fi
  # Exit 0: every result passes; 1: some fail; 2: some invalid.  Anything
  # else means the analysis did not finish.
  status=0
  Rscript "$R_SCRIPT" "$bin" "$label" || status=$?
  case $status in
    0|1) ;;
    2) echo "[invalid] $label has invalid results" >&2; INVALID+=("$name") ;;
    *) echo "[err] R analysis on $label failed (exit $status)" >&2; FAILED+=("$name") ;;
  esac
  rm -f "$bin"
done
} > "$OUT.tmp"

# A report missing a generator is not a report: leave R-REPORT.md alone.
if (( ${#FAILED[@]} > 0 )); then
  mv -f "$OUT.tmp" "$OUT.incomplete"
  echo "[fail] ${#FAILED[@]} generator(s) failed: ${FAILED[*]}; partial output in $OUT.incomplete" >&2
  exit 1
fi
mv -f "$OUT.tmp" "$OUT"
echo "[done] wrote $OUT" >&2
# Invalid results are part of a complete report, but the run is not clean.
if (( ${#INVALID[@]} > 0 )); then
  echo "[invalid] ${#INVALID[@]} generator(s) have invalid results: ${INVALID[*]}" >&2
  exit 2
fi
