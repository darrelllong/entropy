#!/usr/bin/env python3
"""Regenerate the ## Results throughput table in BENCHMARKS.md from stats/.

Reads the per-machine benchmark files written by ``scripts/bench_rngs.sh``
(``stats/<machine>/<name>.bench``, each a one-row Markdown table) and rebuilds
the ``## Results`` table with one ``MW/s`` + ``±CI`` column pair per machine.
Everything else in BENCHMARKS.md — the intro, provenance, and Generator Notes —
is preserved verbatim.

Generator order and display names come from the ``measure`` calls in
``scripts/bench_rngs.sh`` (the single source of truth), so a generator added
there flows into this table automatically on the next run.

Machine columns are discovered from the ``stats/`` subdirectories.  Known
machines render in a curated order with friendly headers; any additional
machine directory is appended alphabetically using its raw name.

Usage:
    python scripts/make_benchmarks.py [--stats DIR] [--output PATH] [--dry-run]
"""

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).parent.parent

# Curated machine order and column headers.  Directories not listed here are
# appended in alphabetical order under their raw name.
KNOWN_MACHINES = [
    ("dyson", "Dyson MW/s"),
    ("dmz", "dmz MW/s"),
    ("moore", "moore MW/s"),
    ("tolkien", "tolkien MW/s"),
    ("baase", "baase MW/s"),
]


def measure_order(bench_script: Path) -> list[tuple[str, str]]:
    """Return [(short_name, display_name), ...] in bench_rngs.sh measure order."""
    rows = []
    # measure <name> "<display>" <words>   (name may be bare, display is quoted)
    pat = re.compile(r'^\s*measure\s+(\S+)\s+"([^"]*)"')
    for line in bench_script.read_text().splitlines():
        m = pat.match(line)
        if m:
            rows.append((m.group(1), m.group(2)))
    return rows


def parse_bench(path: Path) -> tuple[str, str] | None:
    """Return (mean, ci) strings from a .bench row, or None if unreadable."""
    try:
        text = path.read_text()
    except OSError:
        return None
    # | display | mean | ±ci | runs | — parse from the RIGHT so a pipe in the
    # display cell (cell 0) cannot shift the mean/ci columns.  mean, ci, runs
    # are always the last three cells.
    cells = [c.strip() for c in text.strip().strip("|").split("|")]
    if len(cells) < 4:
        return None
    return cells[-3], cells[-2]  # (mean, ci) — passed through verbatim


def discover_machines(stats_root: Path) -> list[tuple[str, str]]:
    if not stats_root.is_dir():
        return []
    # Only count a subdirectory as a machine if it actually holds bench data;
    # an empty or stray dir (stats/archive/, stats/.git/) must not become a
    # column of em-dashes.
    dirs = {
        p.name
        for p in stats_root.iterdir()
        if p.is_dir() and not p.name.startswith(".") and any(p.glob("*.bench"))
    }
    ordered = [(name, hdr) for name, hdr in KNOWN_MACHINES if name in dirs]
    listed = {name for name, _ in ordered}
    extra = sorted(dirs - listed)
    ordered.extend((name, f"{name} MW/s") for name in extra)
    return ordered


def gen_table(stats_root: Path, generators: list[tuple[str, str]],
              machines: list[tuple[str, str]]) -> str:
    header = "| Generator | " + " | ".join(f"{h} | ±CI" for _, h in machines) + " |"
    rule = "|---|" + "|".join(["---:"] * (2 * len(machines))) + "|"
    lines = [header, rule]
    for short, display in generators:
        cells = [f"`{display}`"]
        for machine, _ in machines:
            data = parse_bench(stats_root / machine / f"{short}.bench")
            if data is None:
                cells.extend(["—", "—"])
            else:
                # Pass mean and CI through verbatim: the .bench file is the
                # source of record, so the table must not silently reround it.
                mean, ci = data
                cells.extend([mean, ci])
        lines.append("| " + " | ".join(cells) + " |")
    return "\n".join(lines) + "\n"


def splice(benchmarks_md: str, table: str) -> str:
    """Replace the body between the '## Results' heading and the next section.

    Deterministic and idempotent: the heading capture is exactly the heading
    line (no trailing blank lines), and the body is always rebuilt as one blank
    line, the table, one blank line — regardless of the current spacing, so
    repeated runs are byte-identical.
    """
    m = re.search(r"(^## Results[^\n]*\n)(.*?)(?=^## |\Z)", benchmarks_md,
                  re.MULTILINE | re.DOTALL)
    if not m:
        raise SystemExit("error: '## Results' section not found in BENCHMARKS.md")
    return benchmarks_md[:m.start()] + m.group(1) + "\n" + table + "\n" + benchmarks_md[m.end():]


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--stats", default=str(REPO / "stats"),
                    help="stats root directory (default: stats/)")
    ap.add_argument("--output", default=str(REPO / "BENCHMARKS.md"),
                    help="BENCHMARKS.md path (default: BENCHMARKS.md)")
    ap.add_argument("--dry-run", action="store_true",
                    help="print the table to stdout instead of writing")
    args = ap.parse_args()

    stats_root = Path(args.stats)
    generators = measure_order(REPO / "scripts" / "bench_rngs.sh")
    if not generators:
        raise SystemExit("error: no `measure` lines found in bench_rngs.sh")
    machines = discover_machines(stats_root)
    if not machines:
        raise SystemExit(f"error: no machine directories under {stats_root}")

    table = gen_table(stats_root, generators, machines)

    if args.dry_run:
        sys.stdout.write(table)
        return

    out = Path(args.output)
    out.write_text(splice(out.read_text(), table))
    cols = ", ".join(name for name, _ in machines)
    print(f"Wrote {out}  ({len(generators)} generators, machines: {cols})")


if __name__ == "__main__":
    main()
