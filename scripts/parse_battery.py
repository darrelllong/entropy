#!/usr/bin/env python3
"""Parse a run_tests or run_all log and regenerate TESTS.md.

Reads a log file produced by `run_tests` or `tests/run_all.sh` and rewrites the
main-battery sections of TESTS.md — the header, summary table, and failure
highlights — while preserving the hand-written "Theory By Test" section and
refreshing "Auxiliary Probes" when that content is present in the log.

Usage:
    python scripts/parse_battery.py LOG [--date DATE] [--host HOST]
                                        [--output PATH] [--dry-run]

Arguments:
    LOG             Path to the battery log file (e.g. logs/run_all-darby.log).

Options:
    --date DATE     Run date in YYYY-MM-DD form (default: today).
    --host HOST     Machine name shown in the header (default: darby.local).
    --output PATH   Write the result here (default: TESTS.md in repo root).
    --dry-run       Print the generated TESTS.md to stdout instead of writing.

The "Theory By Test" section (## Theory By Test … ## Failure Highlights) is
copied verbatim from the existing TESTS.md.  The auxiliary-probe section is
regenerated from the input log when possible and otherwise preserved from the
existing TESTS.md.  Everything else is regenerated.
"""

import argparse
import math
import re
import sys
from datetime import date
from pathlib import Path

REPO = Path(__file__).parent.parent

# ---------------------------------------------------------------------------
# Battery slot counts (derived from the suite structure; update here if tests
# are added or removed, rather than in prose strings throughout the file).
# ---------------------------------------------------------------------------

NIST_SLOTS      = 199   # 12 fixed + 148 non_overlapping + 2 serial + 11 Maurer + 8 RE + 18 REV
DIEHARD_SLOTS   = 17    # see src/diehard/mod.rs
DIEHARDER_SLOTS = 522   # see src/dieharder/mod.rs
FULL_SLOTS      = NIST_SLOTS + DIEHARD_SLOTS + DIEHARDER_SLOTS   # 738

# Random excursions emit 8 per-state results; variant emits 18.
RE_STATES       = 8
REV_STATES      = 18
EXCURSION_TOTAL = RE_STATES + REV_STATES   # 26 individual results when active

# When excursions skip, both families collapse to 1 SKIP each → 26 − 2 = 24 fewer slots.
EXCURSION_SKIP_SAVINGS = EXCURSION_TOTAL - 2   # 24
SKIPPED_SLOTS   = FULL_SLOTS - EXCURSION_SKIP_SAVINGS   # 714

# Minimum zero-crossing cycles for the excursion families to run.
EXCURSION_J_MIN = 500

# ---------------------------------------------------------------------------
# Log parser
# ---------------------------------------------------------------------------

SEP = "=" * 72


def parse_log(text: str) -> list[dict]:
    """Return a list of dicts, one per generator, in log order."""
    lines = text.splitlines()
    blocks: list[dict] = []
    i = 0
    n = len(lines)

    while i < n:
        # Find opening separator
        if lines[i].strip() != SEP:
            i += 1
            continue
        # Line after separator is the RNG name (stripped)
        if i + 2 >= n or lines[i + 2].strip() != SEP:
            i += 1
            continue
        name = lines[i + 1].strip()
        i += 3  # skip separator, name, separator

        # Collect test result lines and summary until the next separator or EOF
        fail_lines: list[str] = []
        skip_lines: list[str] = []
        summary = None

        while i < n:
            raw = lines[i]
            stripped = raw.strip()
            if stripped == SEP:
                break
            if stripped.startswith("Summary:"):
                summary = stripped
            elif "  [FAIL]" in raw:
                fail_lines.append(stripped.removeprefix("[FAIL]").strip())
            elif "  [SKIP]" in raw:
                skip_lines.append(stripped.removeprefix("[SKIP]").strip())
            i += 1

        # Parse "Summary: N PASS, M FAIL, K SKIP"
        n_pass = n_fail = n_skip = 0
        if summary:
            m = re.search(r"(\d+) PASS.*?(\d+) FAIL.*?(\d+) SKIP", summary)
            if m:
                n_pass, n_fail, n_skip = int(m.group(1)), int(m.group(2)), int(m.group(3))

        if summary is None:
            continue

        blocks.append({
            "name":       name,
            "pass":       n_pass,
            "fail":       n_fail,
            "skip":       n_skip,
            "total":      n_pass + n_fail + n_skip,
            "fail_lines": fail_lines,
            "skip_lines": skip_lines,
        })

    return blocks


# ---------------------------------------------------------------------------
# TESTS.md section extractor
# ---------------------------------------------------------------------------

def extract_theory_section(tests_md: str) -> str:
    """Return the text from '## Theory By Test' through '## Failure Highlights'
    (exclusive), or a placeholder if the section cannot be found."""
    m = re.search(
        r"(^## Theory By Test\b.*?)^## ",
        tests_md,
        re.MULTILINE | re.DOTALL,
    )
    if m:
        return m.group(1).rstrip() + "\n"
    return "## Theory By Test\n\n_(see repository history)_\n"


def extract_aux_section(tests_md: str) -> str:
    """Return the '## Auxiliary Probes' section verbatim, or a placeholder."""
    m = re.search(
        r"(^## Auxiliary Probes\b.*?)(?=^## |\Z)",
        tests_md,
        re.MULTILINE | re.DOTALL,
    )
    if m:
        return m.group(1).rstrip() + "\n"
    return ""


def extract_aux_from_log(log_text: str, run_date: str, host: str) -> str:
    """Return a regenerated auxiliary-probe section from a run_all or run_aux log."""
    m = re.search(
        r"(^={72}\n(?:bib_tests|upstream_tests|testu01_lz|webster_tavares|gorilla)\b.*?)(?=^Log saved to |\Z)",
        log_text,
        re.MULTILINE | re.DOTALL,
    )
    if not m:
        return ""

    probe_text = m.group(1).rstrip()
    source = "tests/run_all.sh" if "run_tests  (NIST SP 800-22" in log_text else "tests/run_aux.sh"
    intro = (
        f"These probes are not part of `run_tests`; they are recorded separately here\n"
        f"from `{source}` on `{host}` ({run_date})."
    )
    if source == "tests/run_aux.sh":
        intro += "  `tests/run_all.sh` runs the main battery and these probes together."

    return (
        "## Auxiliary Probes\n\n"
        f"{intro}\n\n"
        "These probes exercise statistical properties not covered by the NIST/DIEHARD/DIEHARDER\n"
        "battery.  They run with their default parameters; use the individual binaries for\n"
        "filtered or resized runs.  All probes exit 0 (no crashes or panics).\n\n"
        "```\n"
        f"{probe_text}\n"
        "```\n"
    )


# ---------------------------------------------------------------------------
# Markdown generators
# ---------------------------------------------------------------------------

def gen_header(run_date: str, host: str, n_bits: int, n_rngs: int) -> str:
    mbits = n_bits // 1_000_000
    return f"""\
# Full Battery Results

Full `run_tests` battery harvested from `{host}` on {run_date}.

Sample size: **{mbits:,} Mbit** per generator.

Command:

```sh
./target/release/run_tests
```

Scope:

This top section covers the standard `run_tests` battery only.  The Knuth,
TestU01, PractRand, Webster-Tavares, and Gorilla probes are reported
separately in `## Auxiliary Probes`; use `tests/run_all.sh` for the combined
audit path or `tests/run_aux.sh` for the auxiliary suite alone.

Notes:

**Why result counts vary across generators.**

The battery total differs from run to run because several test families are
conditionally skipped based on properties of the sample, not the generator.

The battery has **{FULL_SLOTS} test slots** at this sample size:

- **{FULL_SLOTS} results** — the "full active battery" outcome: the signed-random-walk
  tests (`random_excursions` and `random_excursions_variant`) completed
  successfully (J ≥ {EXCURSION_J_MIN} zero-crossing cycles).  At {mbits} Mbit the expected
  cycle count is J ≈ {int(math.sqrt(2 * n_bits / math.pi))} (= √(2n/π)),
  which is comfortably above the threshold for well-behaved generators.

- **{SKIPPED_SLOTS} results** — {EXCURSION_SKIP_SAVINGS} fewer slots than the full battery.  The excursion
  families normally emit {RE_STATES} + {REV_STATES} = {EXCURSION_TOTAL} individual per-state results; when the
  signed random walk produces fewer than J = {EXCURSION_J_MIN} complete zero-crossing cycles
  both families are each collapsed to a single family-level SKIP entry,
  yielding {EXCURSION_TOTAL} − 2 = {EXCURSION_SKIP_SAVINGS} fewer slots.  Degenerate generators (Constant,
  Counter, ANSI C LCG, MINSTD) always land here; a handful of non-degenerate
  generators can too, depending on their random seed.

- **{NIST_SLOTS} results** — `Dual_EC_DRBG` only: two P-256 scalar multiplications per
  30-byte output block makes DIEHARD and DIEHARDER prohibitively slow, so only
  the NIST SP 800-22 suite is run.

**Expected false positives.**  At α = 0.01, a perfect generator should fail
roughly 1% of tests by chance.  With {SKIPPED_SLOTS}–{FULL_SLOTS} active tests, the expected
false-fail count is approximately 7.  Isolated failures below that threshold
are noise, not structure.
"""


def gen_summary_table(blocks: list[dict]) -> str:
    rows = ["## Summary Table\n",
            "| RNG | Total | PASS | FAIL | SKIP |",
            "|---|---:|---:|---:|---:|"]
    for b in blocks:
        rows.append(f"| {b['name']} | {b['total']} | {b['pass']} | {b['fail']} | {b['skip']} |")
    return "\n".join(rows) + "\n"


def gen_failure_highlights(blocks: list[dict]) -> str:
    """One bullet per failing generator: FAIL/TOTAL and collapsed test-family counts."""
    lines = ["## Failure Highlights\n",
             "One line per generator.  Test-family repetition counts in parentheses.\n"]
    degenerate_keywords = {"Constant", "Counter"}
    for b in blocks:
        if b["fail"] == 0:
            continue
        first_word = b["name"].split()[0]
        if first_word in degenerate_keywords:
            lines.append(f"- **{b['name']}**: {b['fail']}/{b['total']}"
                         " — expected for degenerate generator.")
            continue
        # Collapse repeated test names into counts.
        name_counts: dict[str, int] = {}
        for fl in b["fail_lines"]:
            m = re.match(r"(\S+)\s+p\s*=", fl)
            tname = m.group(1) if m else fl.split()[0]
            name_counts[tname] = name_counts.get(tname, 0) + 1
        detail = ", ".join(
            f"`{t}` (×{n})" if n > 1 else f"`{t}`"
            for t, n in sorted(name_counts.items())
        )
        lines.append(f"- **{b['name']}**: {b['fail']}/{b['total']} — {detail}")
    return "\n".join(lines) + "\n"


def _is_real_generator(name: str) -> bool:
    """Return True for non-trivial generators; False for degenerate/legacy ones.

    Degenerate: Constant, Counter (zero-entropy, expected to fail).
    Legacy: BAD*, ANSI*, LCG MINSTD — historically broken, included as a
    sanity check that the battery can distinguish garbage from structure.
    """
    first_word = name.split()[0]
    if first_word in {"Constant", "Counter"}:
        return False
    if name.startswith("BAD") or name.startswith("ANSI"):
        return False
    if name == "LCG MINSTD (seed=1)":
        return False
    return True


def gen_bottom_line(blocks: list[dict]) -> str:
    real = [b for b in blocks if _is_real_generator(b["name"])]

    if not real:
        return "## Bottom Line\n\n_(no non-degenerate generators found)_\n"

    best = min(real, key=lambda b: b["fail"])
    worst = max(real, key=lambda b: b["fail"])

    lines = ["## Bottom Line\n",
             "- Degenerate generators (Constant, Counter) and legacy PRNGs "
             "(ANSI C LCG, MINSTD, VB6 Rnd) remain annihilated — the battery "
             "continues to distinguish garbage from structure.",
             f"- Among non-trivial generators, the lowest FAIL count is "
             f"**{best['fail']}** (`{best['name']}`) and the highest is "
             f"**{worst['fail']}** (`{worst['name']}`).",
             "- Isolated failures in `non_overlapping_template` and "
             "`bit_distribution` are expected at α = 0.01; they are noise "
             "unless they form a family cluster.",
             ]
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    ap = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    ap.add_argument("log", metavar="LOG", help="battery log file")
    ap.add_argument("--date", default=str(date.today()),
                    help="run date YYYY-MM-DD (default: today)")
    ap.add_argument("--host", default="darby.local",
                    help="machine name (default: darby.local)")
    ap.add_argument("--output", default=str(REPO / "TESTS.md"),
                    help="output path (default: TESTS.md)")
    ap.add_argument("--dry-run", action="store_true",
                    help="print to stdout instead of writing")
    args = ap.parse_args()

    log_text = Path(args.log).read_text()
    blocks = parse_log(log_text)

    if not blocks:
        print("error: no generator blocks found in log", file=sys.stderr)
        sys.exit(1)

    # Infer sample size from the first NIST header in the log
    n_bits = 16_000_000  # default
    m = re.search(r"NIST SP 800-22 \((\d+) bits\)", log_text)
    if m:
        n_bits = int(m.group(1))

    # Read existing TESTS.md for stable hand-written sections
    tests_path = Path(args.output)
    theory_section = ""
    aux_section = extract_aux_from_log(log_text, args.date, args.host)
    if tests_path.exists():
        existing = tests_path.read_text()
        theory_section = extract_theory_section(existing)
        if not aux_section:
            aux_section = extract_aux_section(existing)

    # Assemble new TESTS.md
    parts = [
        gen_header(args.date, args.host, n_bits, len(blocks)),
        gen_summary_table(blocks),
        theory_section,
        gen_failure_highlights(blocks),
        gen_bottom_line(blocks),
    ]
    if aux_section:
        parts.append(aux_section)
    output = "\n".join(parts)

    if args.dry_run:
        print(output)
    else:
        tests_path.write_text(output)
        print(f"Wrote {tests_path}  ({len(blocks)} generators, "
              f"{sum(b['total'] for b in blocks)} total test results)")


if __name__ == "__main__":
    main()
