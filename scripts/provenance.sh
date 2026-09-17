#!/usr/bin/env bash
# Print what identifies a measurement run, one "key: value" line each: host,
# CPU, OS, date, toolchain, features, lockfile digest, and the revision of
# entropy and of each sibling checkout it builds against ("+modified" when
# the working tree differs from that revision).
#
# usage: scripts/provenance.sh [features] [executable ...]
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
sha256() { (sha256sum "$1" 2>/dev/null || shasum -a 256 "$1") | cut -d' ' -f1; }

cpu() {
    local ncpu model
    if [[ "$(uname -s)" == Darwin ]]; then
        model=$(sysctl -n machdep.cpu.brand_string 2>/dev/null)
        local perf eff
        perf=$(sysctl -n hw.perflevel0.physicalcpu 2>/dev/null)
        eff=$(sysctl -n hw.perflevel1.physicalcpu 2>/dev/null)
        if [[ -n "$perf" && -n "$eff" ]]; then
            echo "${model:-unknown CPU}, ${perf}P+${eff}E cores"
            return
        fi
        ncpu=$(sysctl -n hw.ncpu 2>/dev/null)
    else
        ncpu=$(nproc 2>/dev/null || echo "?")
        model=$(awk -F': *' '/^model name/{print $2; exit}' /proc/cpuinfo 2>/dev/null)
        if [[ -z "$model" ]] && command -v lscpu >/dev/null 2>&1; then
            model=$(lscpu | awk -F': *' '/^Model name/{print $2; exit}')
        fi
    fi
    echo "${model:-$(uname -m)}, ${ncpu} cores"
}

revision() {
    local dir=$1
    if git -C "$dir" rev-parse --verify -q HEAD >/dev/null 2>&1; then
        local rev
        rev=$(git -C "$dir" rev-parse HEAD)
        if git -C "$dir" diff --quiet HEAD -- 2>/dev/null; then
            echo "$rev"
        else
            echo "$rev+modified"
        fi
    else
        echo "absent"
    fi
}

echo "host: $(hostname -s 2>/dev/null || hostname)"
echo "cpu: $(cpu)"
echo "os: $(uname -srm)"
echo "date: $(date '+%Y-%m-%d %H:%M:%S %Z')"
echo "rustc: $(rustc --version 2>/dev/null || echo absent)"
echo "features: ${1:-default}"
echo "Cargo.lock sha256: $(sha256 "$ROOT/Cargo.lock")"
echo "entropy: $(revision "$ROOT")"
for sibling in rump cryptography; do
    echo "$sibling: $(revision "$ROOT/../$sibling")"
done
shift || true
for exe in "$@"; do
    echo "$(basename "$exe") sha256: $(sha256 "$exe")"
done
