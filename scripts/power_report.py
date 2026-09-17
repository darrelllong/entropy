#!/usr/bin/env python3
"""Rebuild the rejection table in POWER.md from `examples/power_curves` output.

The table replaces everything between the BEGIN and END markers; the prose
around it is kept.  Each cell is the number of streams, out of the run's
count, whose family rejected at the family level.

usage: scripts/power_report.py <power_curves output> POWER.md
"""
import collections
import sys

BEGIN = "<!-- power table: begin -->"
END = "<!-- power table: end -->"

# Column order and short headings for the NIST families.
NIST_FAMILIES = [
    ("frequency", "Freq"),
    ("block_frequency", "Block"),
    ("cumulative_sums_forward", "Cusum+"),
    ("cumulative_sums_backward", "Cusum−"),
    ("runs", "Runs"),
    ("longest_run", "Longest"),
    ("matrix_rank", "Rank"),
    ("spectral", "DFT"),
    ("non_overlapping_template", "NonOvl"),
    ("overlapping_template", "Ovl"),
    ("universal", "Univ"),
    ("approximate_entropy", "ApEn"),
    ("serial_delta1", "Serial1"),
    ("serial_delta2", "Serial2"),
    ("linear_complexity", "LinComp"),
    ("random_excursions", "Exc"),
    ("random_excursions_variant", "ExcVar"),
]
# Maurer's universal test at several block lengths is one column: the largest
# count over the lengths run at that size.
MAURER_PREFIX = "maurer::universal_"
MAURER_HEADING = "Maurer (max L)"

DEFECT_ORDER = ["none", "bias", "stuck_low_bits", "repeated_blocks", "lagged_msb", "short_period"]
DEFECT_TEXT = {
    "none": "none",
    "bias": "bits biased to ½ + {}",
    "stuck_low_bits": "{} low bit(s) stuck at 0",
    "repeated_blocks": "{}-word blocks repeated",
    "lagged_msb": "top bit copies bit 30 of previous word",
    "short_period": "period {} words",
}


def strength_key(strength):
    """Order strengths numerically, with 2^-k as its magnitude."""
    if strength == "-":
        return 0.0
    if strength.startswith("2^"):
        return 2.0 ** int(strength[2:])
    return float(strength)


def main():
    if len(sys.argv) != 3:
        sys.exit("usage: power_report.py <power output> <POWER.md>")
    source, report = sys.argv[1], sys.argv[2]
    counts = collections.defaultdict(dict)
    streams = None
    with open(source) as fh:
        for line in fh:
            defect, strength, bits, family, fraction = line.split()
            rejected, total = map(int, fraction.split("/"))
            if streams not in (None, total):
                sys.exit(f"power_report: mixed stream counts {streams} and {total}")
            streams = total
            counts[(defect, strength, int(bits))][family] = rejected

    headings = [h for _, h in NIST_FAMILIES] + [MAURER_HEADING]
    lines = [
        f"Streams per cell: {streams}.  `-`: the family scored no result at that size.",
        "",
        "| Defect | Bits | " + " | ".join(headings) + " |",
        "|---|--:|" + "--:|" * len(headings),
    ]
    keys = sorted(counts, key=lambda k: (DEFECT_ORDER.index(k[0]), -strength_key(k[1]), k[2]))
    for defect, strength, bits in keys:
        row = counts[(defect, strength, bits)]
        cells = [str(row.get("nist::" + f, "-")) for f, _ in NIST_FAMILIES]
        maurer = [v for f, v in row.items() if f.startswith(MAURER_PREFIX)]
        cells.append(str(max(maurer)) if maurer else "-")
        label = DEFECT_TEXT[defect].format(strength)
        lines.append(f"| {label} | 2^{bits.bit_length() - 1} | " + " | ".join(cells) + " |")
    table = "\n".join(lines)

    with open(report) as fh:
        text = fh.read()
    start, end = text.find(BEGIN), text.find(END)
    if start < 0 or end < start:
        sys.exit(f"power_report: {report} lacks the table markers")
    text = text[: start + len(BEGIN)] + "\n" + table + "\n" + text[end:]
    with open(report, "w") as fh:
        fh.write(text)


if __name__ == "__main__":
    main()
