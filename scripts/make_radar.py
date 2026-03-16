#!/usr/bin/env python3
"""Generate assets/benchmarks-radar-new.svg from generator throughput data.

Reads measured throughputs from stats/*.bench when present (Markdown table row
format: | name | MW/s | CI | runs |); falls back to the reference values in the
GENERATORS table for entries without bench files.

Usage:
    python scripts/make_radar.py                      # write SVG in place
    python scripts/make_radar.py --out -              # print to stdout
    python scripts/make_radar.py --stats other/dir    # alternate stats dir
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
# Anchors: HMAC_DRBG (~3.2 MW/s) → r=70, WyRand (~3134 MW/s) → r=275.
A, B = 68.5, 35.4

CX, CY = 450.0, 450.0  # canvas centre
CANVAS  = 900
N_SPOKES = 12
SPOKE_R  = 300          # spoke tip radius (labels anchored here)
RINGS    = [105, 150, 195, 240, 285]

TITLE    = "Pilot Throughput — New Generators (log-normalized)"
SUBTITLE = "Dyson · Apple Silicon aarch64 · pilot-bench normal preset"

# (display label, fallback MW/s, bench_file_or_None)
# Spoke k is at k × 30° clockwise from top.
# All fallback values and bench files are Dyson measurements.
GENERATORS = [
    ("JSF64",          1317.0, "jsf64.bench"),
    ("SFC64",          1266.0, "sfc64.bench"),
    ("Xoshiro256**",   1282.0, "xoshiro256ss.bench"),
    ("Xoroshiro128**",  898.1, "xoroshiro128ss.bench"),
    ("PCG32",           934.3, "pcg32.bench"),
    ("WyRand",         3134.0, "wyrand.bench"),
    ("PCG64",           843.4, "pcg64.bench"),
    ("Squidward",       239.9, "squidward.bench"),
    ("ChaCha20",        171.8, "chacha20.bench"),
    ("SpongeBob",        32.09, "spongebob.bench"),
    ("Hash_DRBG",        12.35, "hash_drbg.bench"),
    ("HMAC_DRBG",         3.203, "hmac_drbg.bench"),
]

# Per-spoke label placement: (text-anchor, dx, dy, line-spacing)
# Offsets are from the spoke tip at SPOKE_R.  dy2 = dy + line-spacing for the
# second line (MW/s annotation).
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
    """SVG (x, y) at radius r on the spoke at deg° clockwise from top."""
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
        m = re.search(r'\|\s*[^|]+\|\s*([\d.]+)\s*\|', text)
        return float(m.group(1)) if m else None
    except OSError:
        return None


def fmt_mw(mw_s: float) -> str:
    """Human-readable MW/s string preserving meaningful precision."""
    if mw_s >= 100:
        if abs(mw_s - round(mw_s)) < 0.05:
            return f"{round(mw_s)} MW/s"
        return f"{mw_s:.1f} MW/s"
    return f"{mw_s:.4g} MW/s"

# ---------------------------------------------------------------------------
# Data loading
# ---------------------------------------------------------------------------

def load_data(stats_dir: Path):
    result = []
    for label, fallback, bench_file in GENERATORS:
        mw_s = fallback
        if bench_file:
            measured = parse_bench(stats_dir / bench_file)
            if measured is not None:
                mw_s = measured
        result.append((label, mw_s))
    return result

# ---------------------------------------------------------------------------
# SVG generation
# ---------------------------------------------------------------------------

def generate_svg(data) -> str:
    lines = []
    w = lines.append

    w(f'<svg xmlns="http://www.w3.org/2000/svg" width="{CANVAS}" height="{CANVAS}" viewBox="0 0 {CANVAS} {CANVAS}">')
    w(f'<rect width="100%" height="100%" fill="#f6f1e8"/>')

    # Grid rings
    w(f'<!-- grid rings at r={",".join(str(r) for r in RINGS)} — {N_SPOKES} spokes, {360//N_SPOKES}° apart -->')
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

    # Data polygon
    radii   = [log_r(mw_s) for _, mw_s in data]
    pts_xy  = [spoke_xy(r, k * 360 / N_SPOKES) for k, r in enumerate(radii)]
    pts_str = " ".join(f"{x:.1f},{y:.1f}" for x, y in pts_xy)
    label_str  = " ".join(lbl for lbl, _ in data)
    radii_str  = " ".join(f"{r:.1f}" for r in radii)

    w(f'<!-- data polygon: {label_str} -->')
    w(f'<!-- radii (log-normalized same scale as original): {radii_str} -->')
    w(f'<polygon points="{pts_str}" fill="#3c6bb855" stroke="#1b4b9f" stroke-width="3"/>')

    # Data dots
    w('<g fill="#1b4b9f">')
    for x, y in pts_xy:
        w(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="4.5"/>')
    w('</g>')

    # Labels
    w('<!-- labels -->')
    w('<g font-family="Georgia, serif" font-size="17" fill="#3f3426">')
    for k, ((label, mw_s), (anchor, dx, dy, dy2)) in enumerate(zip(data, LABEL_CONFIG)):
        tx, ty = spoke_xy(SPOKE_R, k * 360 / N_SPOKES)
        lx  = tx + dx
        ly1 = ty + dy
        ly2 = ly1 + dy2
        w(f'<text x="{lx:.1f}" y="{ly1:.1f}" text-anchor="{anchor}">{label}</text>')
        w(f'<text x="{lx:.1f}" y="{ly2:.1f}" text-anchor="{anchor}" font-size="13" fill="#7a6850">{fmt_mw(mw_s)}</text>')
    w('</g>')

    # Title / subtitle
    w(f'<text x="{CX:.0f}" y="52" text-anchor="middle" font-family="Georgia, serif" font-size="24" fill="#2f2418">{TITLE}</text>')
    w(f'<text x="{CX:.0f}" y="78" text-anchor="middle" font-family="Georgia, serif" font-size="15" fill="#6f5d46">{SUBTITLE}</text>')
    w('</svg>')

    return "\n".join(lines) + "\n"

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--stats", default=str(REPO / "stats"),
                    help="directory containing *.bench files (default: stats/)")
    ap.add_argument("--out",
                    default=str(REPO / "assets" / "benchmarks-radar-new.svg"),
                    help="output path, or - for stdout")
    args = ap.parse_args()

    data = load_data(Path(args.stats))
    svg  = generate_svg(data)

    if args.out == "-":
        print(svg, end="")
    else:
        out = Path(args.out)
        out.write_text(svg)
        print(f"Wrote {out}")
        for label, mw_s in data:
            print(f"  {label:18s}  {mw_s:8.3g} MW/s  r={log_r(mw_s):.1f}")


if __name__ == "__main__":
    main()
