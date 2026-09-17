#!/usr/bin/env python3
"""Summarise an `examples/battery_null` snapshot.

Prints, for every result slot: how many streams scored it, its rejection rate
at 0.05, 0.01 and 0.001, and the standard-normal deviation of the 0.01 rate
from the nominal 1%.  Then the family and whole-battery decisions, and the
slots whose deviation is largest.

usage: scripts/battery_null_report.py <snapshot> [--slots N]
"""
import math
import sys

# Levels the driver counts, in the order it writes them.
LEVELS = [0.05, 0.01, 0.001]

# The level whose deviation the slots are ranked by.
RANK_LEVEL = 1

# Slots listed unless --slots says otherwise.
DEFAULT_SLOTS = 20


def deviation(rejected, trials, level):
    """The rejection rate's deviation from `level`, in standard errors."""
    if trials == 0:
        return 0.0
    error = math.sqrt(level * (1 - level) / trials)
    return (rejected / trials - level) / error


def main():
    argv = sys.argv[1:]
    if not argv:
        sys.exit("usage: battery_null_report.py <snapshot> [--slots N]")
    path = argv[0]
    slots_shown = DEFAULT_SLOTS
    if "--slots" in argv:
        slots_shown = int(argv[argv.index("--slots") + 1])

    streams = 0
    slots, families, battery = [], [], None
    with open(path) as fh:
        for line in fh:
            if line.startswith("# streams"):
                streams = int(line.split()[-1])
            elif line.startswith("slot "):
                fields = line.split()
                name = fields[1]
                scored, insufficient, unsupported, error = map(int, fields[2:6])
                below = list(map(int, fields[6:9]))
                slots.append((name, scored, insufficient, unsupported, error, below))
            elif line.startswith("family "):
                fields = line.split()
                families.append((fields[1], list(map(int, fields[2:5]))))
            elif line.startswith("battery "):
                battery = list(map(int, line.split()[1:4]))

    print(f"streams {streams}, slots {len(slots)}")
    unscored = [s for s in slots if s[1] == 0]
    if unscored:
        print(f"never scored: {', '.join(s[0] for s in unscored)}")
    ranked = sorted(
        (s for s in slots if s[1] > 0),
        key=lambda s: abs(deviation(s[5][RANK_LEVEL], s[1], LEVELS[RANK_LEVEL])),
        reverse=True,
    )
    print(f"\n| Slot | Scored | 0.05 | 0.01 | 0.001 | z at 0.01 |")
    print("|---|--:|--:|--:|--:|--:|")
    for name, scored, _, _, _, below in ranked[:slots_shown]:
        rates = [count / scored for count in below]
        z = deviation(below[RANK_LEVEL], scored, LEVELS[RANK_LEVEL])
        print(
            f"| {name} | {scored} | {rates[0]:.4f} | {rates[1]:.4f} | "
            f"{rates[2]:.5f} | {z:+.1f} |"
        )

    print("\nfamily decisions (Bonferroni over the family's scored results)")
    print("| Family | 0.05 | 0.01 | 0.001 |")
    print("|---|--:|--:|--:|")
    for name, counts in sorted(families, key=lambda f: -f[1][RANK_LEVEL]):
        rates = [count / streams for count in counts]
        print(f"| {name} | {rates[0]:.4f} | {rates[1]:.4f} | {rates[2]:.5f} |")
    if battery:
        rates = [count / streams for count in battery]
        print(
            f"\nany family rejects: {rates[0]:.4f} at 0.05, {rates[1]:.4f} at "
            f"0.01, {rates[2]:.5f} at 0.001"
        )


if __name__ == "__main__":
    main()
