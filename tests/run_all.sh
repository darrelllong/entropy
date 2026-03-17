#!/usr/bin/env bash
# Full repository audit: main NIST/DIEHARD/DIEHARDER battery + all auxiliary probes.
#
# Output is written to both stdout and a timestamped log file under logs/.
# The main-battery portion of the log can be fed to scripts/parse_battery.py
# to regenerate TESTS.md.
#
# Usage:
#   tests/run_all.sh                  # full audit, default parameters
#   tests/run_all.sh --quick          # quick mode for the main battery only
#
# The log file path is printed at the end of the run.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    cat <<'EOF'
Usage: tests/run_all.sh [run_tests options...]

Runs the full repository audit:
  1. NIST SP 800-22 / DIEHARD / DIEHARDER battery  (run_tests)
  2. Knuth + ApEn profile                           (bib_tests)
  3. TestU01 Hamming + PractRand FPF                (upstream_tests)
  4. TestU01 Lempel-Ziv                             (testu01_lz)
  5. SAC / BIC avalanche                            (webster_tavares)
  6. Marsaglia-Tsang Gorilla                        (gorilla)

All run_tests flags (--quick, --suite, --rng, etc.) are forwarded to step 1.
Steps 2-6 always run with their default parameters.

Output is tee'd to logs/run_all-<host>-<YYYYMMDD-HHMMSS>.log.
Feed that file to scripts/parse_battery.py to regenerate TESTS.md.

Examples:
  tests/run_all.sh
  tests/run_all.sh --quick
  tests/run_all.sh --suite nist
EOF
    exit 0
fi

LOG_DIR="${LOG_DIR:-$ROOT_DIR/logs}"
mkdir -p "$LOG_DIR"
HOST_TAG="$(hostname -s 2>/dev/null || hostname)"
LOG="$LOG_DIR/run_all-${HOST_TAG}-$(date +%Y%m%d-%H%M%S).log"

# Tee everything (stdout + stderr) to the log.
exec > >(tee "$LOG") 2>&1

cargo build --quiet --release \
    --bin run_tests \
    --bin bib_tests \
    --bin upstream_tests \
    --bin testu01_lz \
    --bin webster_tavares \
    --bin gorilla

BIN="$ROOT_DIR/target/release"
SEP=$(printf '=%.0s' {1..72})

section() {
    printf '\n%s\n%s\n%s\n\n' "$SEP" "$1" "$SEP"
}

section "run_tests  (NIST SP 800-22 · DIEHARD · DIEHARDER)"
"$BIN/run_tests" "$@"

section "bib_tests  (Knuth + NIST ApEn profile m=2..6)"
"$BIN/bib_tests"

section "upstream_tests  (TestU01 HammingCorr/HammingIndep · PractRand FPF)"
"$BIN/upstream_tests"

section "testu01_lz  (TestU01 Lempel-Ziv  k=25  replications=10)"
"$BIN/testu01_lz"

section "webster_tavares  (SAC / BIC avalanche  samples=4096  bits=32)"
"$BIN/webster_tavares"

section "gorilla  (Marsaglia-Tsang Gorilla  all 32 bit positions)"
"$BIN/gorilla"

printf '\nLog saved to %s\n' "$LOG"
printf 'To regenerate TESTS.md:\n'
printf '  python3 scripts/parse_battery.py %s\n' "$LOG"
