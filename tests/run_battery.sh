#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    cat <<'EOF'
Usage: tests/run_battery.sh [run_tests options...]

Builds the release test runner and then executes it.

Examples:
  tests/run_battery.sh
  tests/run_battery.sh --quick
  tests/run_battery.sh --suite nist
  tests/run_battery.sh --test nist::frequency
  tests/run_battery.sh --rng "SpongeBob"
  tests/run_battery.sh --suite diehard --quick --rng "Windows"

All arguments are passed directly to the run_tests binary.
EOF
    exit 0
fi

cargo build --release --bin run_tests
exec "$ROOT_DIR/target/release/run_tests" "$@"
