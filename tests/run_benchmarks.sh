#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    cat <<'EOF'
Usage: tests/run_benchmarks.sh [bench_rngs options...]

Builds the release pilot probe and then delegates to scripts/bench_rngs.sh.

Examples:
  tests/run_benchmarks.sh
  tests/run_benchmarks.sh --preset normal
  tests/run_benchmarks.sh --preset normal aes_ctr mt19937 spongebob squidward
  PILOT_BENCH_CLI=~/pilot-bench/build/cli/bench tests/run_benchmarks.sh --force

Notes:
  - Results are written to stats/<name>.bench.
  - Existing stats rows are reused unless --force is supplied.
  - All additional arguments are passed through unchanged.
EOF
    exit 0
fi

cargo build --release --bin pilot_rng
exec "$ROOT_DIR/scripts/bench_rngs.sh" "$@"
