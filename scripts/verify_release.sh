#!/usr/bin/env bash
# Build and test entropy from empty target directories, with and without
# default features, against the sibling checkouts beside it, and print the
# provenance of the run.  Run it on one host of each platform before a
# release.
#
# usage: scripts/verify_release.sh [work directory]
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="${1:-$(mktemp -d "${TMPDIR:-/tmp}/entropy-verify.XXXXXX")}"
mkdir -p "$WORK"
cd "$ROOT" || exit 1

"$ROOT/scripts/provenance.sh" "default and none"
status=0
step() {
    local name=$1; shift
    if "$@" > "$WORK/$name.log" 2>&1; then
        echo "$name: ok $(grep -h 'test result' "$WORK/$name.log" \
            | awk '{p+=$4; f+=$6} END {if (NR) print p" passed, "f" failed"}')"
    else
        echo "$name: FAILED (log $WORK/$name.log)"
        tail -15 "$WORK/$name.log"
        status=1
    fi
}
step default-build env CARGO_TARGET_DIR="$WORK/default" cargo build --locked --release --all-targets
step default-test  env CARGO_TARGET_DIR="$WORK/default" cargo test --locked --release
step no-default-build env CARGO_TARGET_DIR="$WORK/no-default" cargo build --locked --release --all-targets --no-default-features
step no-default-test  env CARGO_TARGET_DIR="$WORK/no-default" cargo test --locked --release --no-default-features
# Each feature alone, so neither carries the other by accident.
for feature in batteries cryptography; do
    step "$feature-build" env CARGO_TARGET_DIR="$WORK/$feature" \
        cargo build --locked --release --all-targets --no-default-features --features "$feature"
    step "$feature-test" env CARGO_TARGET_DIR="$WORK/$feature" \
        cargo test --locked --release --no-default-features --features "$feature"
done
# The minimal build is the consumer promise: no dependencies at all.
dependencies=$(cargo tree --no-default-features --edges normal --prefix none 2>/dev/null | grep -cv '^rng-entropy')
if [ "$dependencies" -eq 0 ]; then
    echo "minimal-dependencies: ok none"
else
    echo "minimal-dependencies: FAILED $dependencies dependencies"
    cargo tree --no-default-features --edges normal
    status=1
fi
exit $status
