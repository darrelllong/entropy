#!/usr/bin/env bash
# RNG throughput benchmark using pilot-bench.
#
# For each generator, measures throughput (MW/s) with 95% CI using pilot-bench
# and writes a result file to stats/<machine>/<name>.bench.  If the file already
# exists, that RNG is skipped unless --force is given.
#
# Usage:
#   scripts/bench_rngs.sh [--preset quick|normal|strict] [--machine <name>] \
#                         [--force] [name ...]
#
#   --machine <name>  subdirectory under stats/ for this machine (default: dyson)
#   name ...          optional whitelist; if given, only measure those generators.
#
# Environment:
#   PILOT_BENCH_CLI   path to the pilot bench CLI  (default: ~/pilot-bench/build/cli/bench)
#   PILOT_RNG_BIN     path to pilot_rng binary      (default: target/release/pilot_rng)
#   PILOT_PRESET      quick | normal | strict       (default: quick)
#   PILOT_MACHINE     machine subdirectory name     (default: dyson)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BENCH="${PILOT_BENCH_CLI:-$HOME/pilot-bench/build/cli/bench}"
RNG_BIN="${PILOT_RNG_BIN:-$ROOT_DIR/target/release/pilot_rng}"
PRESET="${PILOT_PRESET:-quick}"
MACHINE="${PILOT_MACHINE:-dyson}"
FORCE=0
WHITELIST=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --preset=*) PRESET="${1#--preset=}" ;;
        --preset)   shift; PRESET="$1" ;;
        --machine=*) MACHINE="${1#--machine=}" ;;
        --machine)  shift; MACHINE="$1" ;;
        --force)    FORCE=1 ;;
        *)          WHITELIST+=("$1") ;;
    esac
    shift
done

STATS_DIR="$ROOT_DIR/stats/$MACHINE"
mkdir -p "$STATS_DIR"

# measure <rng_name> <display_name> <words_per_probe>
# Writes result to stats/<machine>/<rng_name>.bench and prints a Markdown table row.
# Skips if stats file exists and --force not given.
# NOTE: no numeric underscores in <words_per_probe> — bash passes them literally.
measure() {
    local rng_name=$1 display=$2 words=$3
    local stat_file="$STATS_DIR/${rng_name}.bench"

    # Apply whitelist filter.
    if [[ ${#WHITELIST[@]} -gt 0 ]]; then
        local found=0
        for w in "${WHITELIST[@]}"; do [[ "$w" == "$rng_name" ]] && found=1; done
        [[ $found -eq 0 ]] && return
    fi

    # Skip if already measured.
    if [[ $FORCE -eq 0 && -f "$stat_file" ]]; then
        # Re-emit the cached row.
        cat "$stat_file"
        return
    fi

    local out mean ci rounds
    out=$("$BENCH" run_program \
          --preset "$PRESET" \
          --pi "${rng_name},MW/s,0,1,1" \
          --env "PILOT_RNG_WORDS=${words}" \
          -- "$RNG_BIN" "$rng_name" 2>&1)
    mean=$(  echo "$out" | awk '/Reading mean/{print $5}')
    ci=$(    echo "$out" | awk '/Reading CI/{print $5}')
    rounds=$(echo "$out" | awk '/^Rounds:/{print $2}')

    local row
    row=$(printf "| %-36s | %8s | %-8s | %5s |" \
                 "$display" "$mean" "±$ci" "$rounds")

    # Cache for future runs.
    echo "$row" > "$stat_file"
    echo "$row"
}

echo ""
echo "## RNG throughput benchmark (pilot-bench, preset=$PRESET, machine=$MACHINE)"
echo ""
echo "Throughput in MW/s (10⁶ u32 words/s).  CI is 95%."
echo ""
echo "| Generator                            |   MW/s   | ±CI 95%  | Runs  |"
echo "|--------------------------------------|----------|----------|-------|"

# Keep each probe comfortably above timer noise. The ultra-fast synthetic
# generators need much larger batches than the cryptographic or libc RNGs.
#
#              name             display                                      words/probe
measure osrng         "OsRng (/dev/urandom)"                           100000
measure mt19937       "MT19937 (seed=19650218)"                       25000000
measure xorshift64    "Xorshift64 (seed=1)"                           25000000
measure xorshift32    "Xorshift32 (seed=1)"                           25000000
measure sysv_rand     "BAD Unix System V rand() (seed=1)"             10000000
measure rand48        "BAD Unix System V mrand48() (seed=1)"          25000000
measure bsd_random    "BAD Unix BSD random() TYPE_3 (seed=1)"         10000000
measure linux_glibc_random "BAD Unix Linux glibc rand()/random() (seed=1)" 10000000
measure bsd_rand_compat "BAD Unix FreeBSD12 rand_r() compat (seed=1)" 10000000
measure windows_msvc_rand "BAD Windows CRT rand() (seed=1)"           10000000
measure windows_vb6_rnd "BAD Windows VB6/VBA Rnd() (seed=1)"          10000000
measure windows_dotnet_random "BAD Windows .NET Random(seed=1) compat" 10000000
measure ansi_c_lcg    "ANSI C sample LCG (seed=1)"                    10000000
measure lcg_minstd    "LCG MINSTD (seed=1)"                           10000000
measure bbs           "BBS (p=2^31-1, q=4294967291)"                  10000000
measure blum_micali   "Blum-Micali (p=2^31-1, g=7)"                       50000
measure aes_ctr       "AES-128-CTR (NIST key)"                        10000000
measure camellia_ctr  "Camellia-128-CTR (key=00..0f)"                 10000000
measure twofish_ctr   "Twofish-128-CTR (key=00..0f)"                  10000000
measure serpent_ctr   "Serpent-128-CTR (key=00..0f)"                  10000000
measure sm4_ctr       "SM4-CTR (key=00..0f)"                          10000000
measure grasshopper_ctr "Grasshopper-CTR (key=00..1f)"                10000000
measure cast128_ctr   "CAST-128-CTR (key=00..0f)"                     10000000
measure seed_ctr      "SEED-CTR (key=00..0f)"                         10000000
measure rabbit        "Rabbit (key=00..0f, iv=00..07)"                10000000
measure salsa20       "Salsa20 (key=00..1f, nonce=00..07)"            50000000
measure snow3g        "Snow3G (key=00..0f, iv=00..0f)"                10000000
measure zuc128        "ZUC-128 (key=00..0f, iv=00..0f)"               10000000
measure spongebob     "SpongeBob (SHA3-512 chain, seed=00..3f)"        5000000
measure squidward     "Squidward (SHA-256 chain, seed=00..1f)"        10000000
measure pcg32         "PCG32 (seed=42, seq=54)"                       50000000
measure pcg64         "PCG64 (state=1, seq=1)"                        25000000
measure xoshiro256ss  "Xoshiro256** (seeds=1,2,3,4)"                  50000000
measure xoroshiro128ss "Xoroshiro128** (seeds=1,2)"                   50000000
measure wyrand        "WyRand (seed=42)"                               50000000
measure sfc64         "SFC64 (seeds=1,2,3)"                           50000000
measure jsf64         "JSF64 (seed=0xdeadbeef)"                       50000000
measure chacha20      "ChaCha20 CSPRNG (OsRng key)"                   50000000
measure hmac_drbg     "HMAC_DRBG SHA-256 (OsRng seed)"                 1000000
measure hash_drbg     "Hash_DRBG SHA-256 (OsRng seed)"                 5000000
measure crypto_ctr_drbg "cryptography::CtrDrbgAes256 (seed=00..2f)"    1000000
measure constant      "Constant (0xDEAD_DEAD)"                      1000000000
measure counter       "Counter (0,1,2,...)"                         1000000000

echo ""
