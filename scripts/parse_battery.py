#!/usr/bin/env python3
"""Parse a run_tests battery log and regenerate TESTS.md.

Reads a log file produced by `run_tests` and rewrites the variable sections of
TESTS.md — the header, summary table, and failure highlights — while preserving
the hand-written "Theory By Test" section from the existing TESTS.md.

Usage:
    python scripts/parse_battery.py LOG [--date DATE] [--host HOST]
                                        [--output PATH] [--dry-run]

Arguments:
    LOG             Path to the battery log file (e.g. /tmp/battery-16m.log).

Options:
    --date DATE     Run date in YYYY-MM-DD form (default: today).
    --host HOST     Machine name shown in the header (default: darby.local).
    --output PATH   Write the result here (default: TESTS.md in repo root).
    --dry-run       Print the generated TESTS.md to stdout instead of writing.

The "Theory By Test" section (## Theory By Test … ## Failure Highlights) is
copied verbatim from the existing TESTS.md.  Everything else is regenerated.
"""

import argparse
import math
import re
import sys
from datetime import date
from pathlib import Path

REPO = Path(__file__).parent.parent

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

Notes:

**Why result counts vary across generators.**

The battery total differs from run to run because several test families are
conditionally skipped based on properties of the sample, not the generator.

The battery has **738 test slots** at this sample size:

- **738 results** — the "full active battery" outcome: the signed-random-walk
  tests (`random_excursions` and `random_excursions_variant`) completed
  successfully (J ≥ 500 zero-crossing cycles).  At {mbits} Mbit the expected
  cycle count is J ≈ {int(math.sqrt(2 * n_bits / math.pi))} (= √(2n/π)),
  comfortably above the threshold for well-behaved generators.

- **714 results** — 24 fewer slots than the full battery.  The excursion
  families normally emit 8 + 18 = 26 individual per-state results; when the
  signed random walk produces fewer than J = 500 complete zero-crossing cycles
  both families are each collapsed to a single family-level SKIP entry,
  yielding 26 − 2 = 24 fewer slots.  Degenerate generators (Constant,
  Counter, ANSI C LCG, MINSTD) always land here; a handful of non-degenerate
  generators can too, depending on their random seed.

- **199 results** — `Dual_EC_DRBG` only: two P-256 scalar multiplications per
  30-byte output block makes DIEHARD and DIEHARDER prohibitively slow, so only
  the NIST SP 800-22 suite is run.

**Expected false positives.**  At α = 0.01, a perfect generator should fail
roughly 1% of tests by chance.  With 714–738 active tests, the expected
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


def _fail_section(b: dict) -> str:
    name = b["name"]
    n_fail = b["fail"]
    n_total = b["total"]
    lines = [f"### {name}\n",
             f"- `{n_fail}` failure{'s' if n_fail != 1 else ''} out of `{n_total}` tests:"]
    for fl in b["fail_lines"]:
        # Extract test name and p-value for a tidy bullet
        m = re.match(r"(\S+)\s+p\s*=\s*([\d.e+-]+)\s+\((.*)\)", fl)
        if m:
            tname, pval, detail = m.group(1), m.group(2), m.group(3)
            lines.append(f"  - `{tname}`: p = {pval}  ({detail})")
        else:
            lines.append(f"  - {fl}")
    return "\n".join(lines) + "\n"


def gen_failure_highlights(blocks: list[dict]) -> str:
    parts = ["## Failure Highlights\n"]
    # Degenerate generators (Constant, Counter) — note briefly
    degenerate_keywords = {"Constant", "Counter"}
    for b in blocks:
        if b["fail"] == 0:
            continue
        first_word = b["name"].split()[0]
        if first_word in degenerate_keywords:
            parts.append(f"### {b['name']}\n\n"
                         f"- `{b['fail']}/{b['total']}` failures — expected for a degenerate generator.\n")
        else:
            parts.append(_fail_section(b))
    return "\n".join(parts)


def gen_bottom_line(blocks: list[dict]) -> str:
    # Separate degenerate from real generators
    degenerate_starts = {"Constant", "Counter"}
    bad_starts = {"BAD", "ANSI", "LCG", "MINSTD"}

    real = [b for b in blocks
            if b["name"].split()[0] not in degenerate_starts
            and not b["name"].startswith("BAD")
            and not b["name"].startswith("ANSI")
            and b["name"] != "LCG MINSTD (seed=1)"]

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

    # Read existing TESTS.md for the stable Theory section
    tests_path = Path(args.output)
    theory_section = ""
    if tests_path.exists():
        theory_section = extract_theory_section(tests_path.read_text())

    # Assemble new TESTS.md
    parts = [
        gen_header(args.date, args.host, n_bits, len(blocks)),
        gen_summary_table(blocks),
        theory_section,
        gen_failure_highlights(blocks),
        gen_bottom_line(blocks),
    ]
    output = "\n".join(parts)

    if args.dry_run:
        print(output)
    else:
        tests_path.write_text(output)
        print(f"Wrote {tests_path}  ({len(blocks)} generators, "
              f"{sum(b['total'] for b in blocks)} total test results)")


if __name__ == "__main__":
    main()
