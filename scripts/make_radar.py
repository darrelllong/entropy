#!/usr/bin/env python3
"""Generate assets/benchmarks-radar.svg from generator throughput data.

Reads measured throughputs from stats/<machine>/*.bench when present (Markdown
table row format: | name | MW/s | CI | runs |); falls back to the reference
values in the GENERATORS table for entries without bench files.

One polygon is drawn per machine; machines with no bench files are skipped.

Usage:
    python scripts/make_radar.py                      # write SVG in place
    python scripts/make_radar.py --out -              # print to stdout
    python scripts/make_radar.py --stats other/dir    # alternate stats root
"""

import argparse
import math
import re
from pathlib import Path

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

REPO = Path(__file__).parent.parent

# Log-normalisation: r = A * log10(MW/s) + B
# Calibrated for Dyson (Apple Silicon) measurements.
# Anchors: HMAC_DRBG (~3.2 MW/s) → r=70, WyRand (~3120 MW/s) → r=275.
A, B = 68.5, 35.4

CX, CY  = 450.0, 450.0  # canvas centre
CANVAS  = 900
N_SPOKES = 12
SPOKE_R  = 300           # spoke tip radius (labels anchored here)
RINGS    = [105, 150, 195, 240, 285]

TITLE    = "Pilot Throughput — New Generators (log-normalized)"

# (display label, fallback MW/s, bench_file)
# Spoke k is at k × 30° clockwise from top.
# Fallback values are Dyson measurements.
GENERATORS = [
    ("JSF64",          1314.0, "jsf64.bench"),
    ("SFC64",          1262.0, "sfc64.bench"),
    ("Xoshiro256**",   1287.0, "xoshiro256ss.bench"),
    ("Xoroshiro128**",  902.8, "xoroshiro128ss.bench"),
    ("PCG32",           934.1, "pcg32.bench"),
    ("WyRand",         3120.0, "wyrand.bench"),
    ("PCG64",           843.8, "pcg64.bench"),
    ("Squidward",       239.6, "squidward.bench"),
    ("ChaCha20",        170.7, "chacha20.bench"),
    ("SpongeBob",        32.22, "spongebob.bench"),
    ("Hash_DRBG",        12.37, "hash_drbg.bench"),
    ("HMAC_DRBG",         3.218, "hmac_drbg.bench"),
]

# Per-machine rendering config: (subdir, fill, stroke, legend_label)
MACHINES = [
    ("dyson", "#3c6bb855", "#1b4b9f", "#1b4b9f", "Dyson (Apple M4 aarch64)"),
    ("dmz",   "#b83c3c55", "#9f1b1b", "#9f1b1b", "dmz.lan (Intel i5 x86_64)"),
]

# Per-spoke label placement: (text-anchor, dx, dy, line-spacing)
LABEL_CONFIG = [
    ("middle",  0, -42, 18),  # 0°   JSF64
    ("start",  18, -24, 18),  # 30°  SFC64
    ("start",  22, -20, 18),  # 60°  Xoshiro256**
    ("start",   8, -10, 18),  # 90°  Xoroshiro128**
    ("start",  22,  14, 18),  # 120° PCG32
    ("start",  18,  16, 18),  # 150° WyRand
    ("middle",  0,  20, 18),  # 180° PCG64
    ("end",   -18,  16, 18),  # 210° Squidward
    ("end",   -22,   7, 18),  # 240° ChaCha20
    ("end",    -8, -10, 18),  # 270° SpongeBob
    ("end",   -22, -20, 18),  # 300° Hash_DRBG
    ("end",   -18, -28, 18),  # 330° HMAC_DRBG
]

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def spoke_xy(r: float, deg: float):
    rad = math.radians(deg)
    return CX + r * math.sin(rad), CY - r * math.cos(rad)


def ring_polygon(r: float) -> str:
    pts = [spoke_xy(r, k * 360 / N_SPOKES) for k in range(N_SPOKES)]
    return " ".join(f"{x:.1f},{y:.1f}" for x, y in pts)


def log_r(mw_s: float) -> float:
    return A * math.log10(mw_s) + B


def parse_bench(path: Path):
    """Return MW/s from a one-row Markdown bench file, or None on failure."""
    try:
        text = path.read_text()
        m = re.search(r'\|\s*[^|]+\|\s*([\d.e+]+)\s*\|', text)
        return float(m.group(1)) if m else None
    except OSError:
        return None


def fmt_mw(mw_s: float) -> str:
    if mw_s >= 100:
        if abs(mw_s - round(mw_s)) < 0.05:
            return f"{round(mw_s)} MW/s"
        return f"{mw_s:.1f} MW/s"
    return f"{mw_s:.4g} MW/s"

# ---------------------------------------------------------------------------
# Data loading
# ---------------------------------------------------------------------------

def load_machine_data(stats_root: Path, subdir: str):
    """Return list of (label, mw_s) for this machine, or None if no bench files found."""
    machine_dir = stats_root / subdir
    result = []
    found_any = False
    for label, fallback, bench_file in GENERATORS:
        measured = parse_bench(machine_dir / bench_file) if machine_dir.exists() else None
        if measured is not None:
            found_any = True
            result.append((label, measured))
        else:
            result.append((label, fallback))
    return result if found_any else None

# ---------------------------------------------------------------------------
# SVG generation
# ---------------------------------------------------------------------------

def generate_svg(stats_root: Path) -> str:
    lines = []
    w = lines.append

    w(f'<svg xmlns="http://www.w3.org/2000/svg" width="{CANVAS}" height="{CANVAS}" viewBox="0 0 {CANVAS} {CANVAS}">')
    w(f'<rect width="100%" height="100%" fill="#f6f1e8"/>')

    # Grid rings
    w(f'<!-- grid rings -->')
    w('<g stroke="#d8cdb8" fill="none">')
    for r in RINGS:
        w(f'<polygon points="{ring_polygon(r)}" stroke-width="1"/>')
    w('</g>')

    # Spokes
    w('<!-- spokes -->')
    w('<g stroke="#b8aa90" stroke-width="1">')
    for k in range(N_SPOKES):
        x2, y2 = spoke_xy(SPOKE_R, k * 360 / N_SPOKES)
        w(f'<line x1="{CX:.0f}" y1="{CY:.0f}" x2="{x2:.1f}" y2="{y2:.1f}"/>')
    w('</g>')

    # One polygon + dots per machine
    legend_entries = []
    label_data = None  # use first machine's data for labels
    for subdir, fill, stroke, dot_fill, legend_label in MACHINES:
        data = load_machine_data(stats_root, subdir)
        if data is None:
            continue
        if label_data is None:
            label_data = data

        radii  = [log_r(mw_s) for _, mw_s in data]
        pts_xy = [spoke_xy(r, k * 360 / N_SPOKES) for k, r in enumerate(radii)]
        pts_str = " ".join(f"{x:.1f},{y:.1f}" for x, y in pts_xy)

        w(f'<!-- {subdir} polygon -->')
        w(f'<polygon points="{pts_str}" fill="{fill}" stroke="{stroke}" stroke-width="3"/>')
        w(f'<g fill="{dot_fill}">')
        for x, y in pts_xy:
            w(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="4.5"/>')
        w('</g>')
        legend_entries.append((fill, stroke, legend_label))

    # Labels (from first machine with data)
    if label_data:
        w('<!-- labels -->')
        w('<g font-family="Georgia, serif" font-size="17" fill="#3f3426">')
        for k, ((label, mw_s), (anchor, dx, dy, dy2)) in enumerate(zip(label_data, LABEL_CONFIG)):
            tx, ty = spoke_xy(SPOKE_R, k * 360 / N_SPOKES)
            lx  = tx + dx
            ly1 = ty + dy
            ly2 = ly1 + dy2
            w(f'<text x="{lx:.1f}" y="{ly1:.1f}" text-anchor="{anchor}">{label}</text>')
            w(f'<text x="{lx:.1f}" y="{ly2:.1f}" text-anchor="{anchor}" font-size="13" fill="#7a6850">{fmt_mw(mw_s)}</text>')
        w('</g>')

    # Legend
    if len(legend_entries) > 1:
        lx, ly = 30, CANVAS - 30 - len(legend_entries) * 22
        w('<!-- legend -->')
        w('<g font-family="Georgia, serif" font-size="14" fill="#3f3426">')
        for i, (fill, stroke, lbl) in enumerate(legend_entries):
            y = ly + i * 22
            w(f'<rect x="{lx}" y="{y - 12}" width="18" height="14" fill="{fill}" stroke="{stroke}" stroke-width="2"/>')
            w(f'<text x="{lx + 24}" y="{y}">{lbl}</text>')
        w('</g>')

    # Title
    machines_with_data = [m[0] for m in MACHINES
                          if load_machine_data(stats_root, m[0]) is not None]
    subtitle = " · ".join(machines_with_data) + " · pilot-bench normal preset"
    w(f'<text x="{CX:.0f}" y="52" text-anchor="middle" font-family="Georgia, serif" font-size="24" fill="#2f2418">{TITLE}</text>')
    w(f'<text x="{CX:.0f}" y="78" text-anchor="middle" font-family="Georgia, serif" font-size="15" fill="#6f5d46">{subtitle}</text>')
    w('</svg>')

    return "\n".join(lines) + "\n"

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--stats", default=str(REPO / "stats"),
                    help="root stats directory containing machine subdirs (default: stats/)")
    ap.add_argument("--out",
                    default=str(REPO / "assets" / "benchmarks-radar.svg"),
                    help="output path, or - for stdout")
    args = ap.parse_args()

    stats_root = Path(args.stats)
    svg = generate_svg(stats_root)

    if args.out == "-":
        print(svg, end="")
    else:
        out = Path(args.out)
        out.write_text(svg)
        print(f"Wrote {out}")
        for subdir, *_ in MACHINES:
            data = load_machine_data(stats_root, subdir)
            if data is None:
                print(f"  {subdir}: no bench files, skipped")
                continue
            print(f"  {subdir}:")
            for label, mw_s in data:
                print(f"    {label:18s}  {mw_s:8.3g} MW/s  r={log_r(mw_s):.1f}")


if __name__ == "__main__":
    main()
