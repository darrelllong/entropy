#!/usr/bin/env bash
# Run all auxiliary research probes and print results to stdout.
#
# Probes covered:
#   bib_tests      — Knuth permutation/gap/runs-median + NIST ApEn profile (m=2..6)
#   upstream_tests — TestU01 HammingCorr/HammingIndep, PractRand FPF(4,14,6)
#   testu01_lz     — TestU01 Lempel-Ziv core statistic (k=25, 10 replications)
#   webster_tavares — SAC / BIC avalanche analysis (4096 samples, 32-bit I/O)
#   gorilla        — Marsaglia-Tsang Gorilla (all 32 bit positions + aggregate KS)
#
# All probes run with their default parameters (no --rng filter, no size flags).
# Use the individual binaries directly for filtered or sized runs.
#
# Usage:
#   tests/run_aux.sh
#   tests/run_aux.sh 2>&1 | tee /tmp/aux.log

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    cat <<'EOF'
Usage: tests/run_aux.sh

Builds the five auxiliary research probes and runs all of them with their
default parameters.  Output goes to stdout; redirect or tee to save a log.

Examples:
  tests/run_aux.sh
  tests/run_aux.sh 2>&1 | tee /tmp/aux-$(date +%Y%m%d).log
EOF
    exit 0
fi

SEP=$(printf '=%.0s' {1..72})

section() {
    printf '\n%s\n%s\n%s\n\n' "$SEP" "$1" "$SEP"
}

BIN="$ROOT_DIR/target/release"

cargo build --quiet --release \
    --bin bib_tests \
    --bin upstream_tests \
    --bin testu01_lz \
    --bin webster_tavares \
    --bin gorilla

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
