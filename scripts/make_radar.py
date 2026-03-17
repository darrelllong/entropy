#!/usr/bin/env python3
"""Generate radar SVG charts from generator throughput data.

Reads measured throughputs from stats/<machine>/*.bench (Markdown table row
format: | name | MW/s | CI | runs |); falls back to the reference values in
each chart's GENERATORS list for missing bench files.

One polygon is drawn per machine that has bench files; machines with no data
are skipped.  Two charts are produced:

  assets/benchmarks-radar-fast.svg   — fast / simulation-grade generators
  assets/benchmarks-radar-slow.svg   — slow / cryptographic generators

Usage:
    python scripts/make_radar.py                      # write both SVGs
    python scripts/make_radar.py --stats other/dir    # alternate stats root
"""

import argparse
import math
import re
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Shared geometry
# ---------------------------------------------------------------------------

REPO     = Path(__file__).parent.parent
CX, CY   = 450.0, 450.0
CANVAS   = 900
N_SPOKES = 12
SPOKE_R  = 300
RINGS    = [105, 150, 195, 240, 285]

# Per-machine rendering: (stats subdir, fill, stroke, dot fill, legend label)
MACHINES = [
    ("dyson", "#3c6bb855", "#1b4b9f", "#1b4b9f", "Dyson (Apple M4 aarch64)"),
    ("dmz",   "#b83c3c55", "#9f1b1b", "#9f1b1b", "dmz.lan (Intel i5 x86_64)"),
]

# ---------------------------------------------------------------------------
# Chart definitions
# ---------------------------------------------------------------------------
# Each chart is a dict with:
#   title      — chart title
#   subtitle   — static subtitle text (machines appended at render time)
#   A, B       — log-normalisation: r = A * log10(MW/s) + B
#   out        — output filename under assets/
#   generators — list of (display label, Dyson fallback MW/s, bench file)
#   labels     — list of (text-anchor, dx, dy, line-spacing) per spoke
#
# Scale calibration (Dyson numbers):
#   fast chart: sysv_rand (~441) → r=70, WyRand (~3120) → r=270
#   slow chart: Blum-Micali (~0.462) → r=70, Squidward (~240) → r=270

CHARTS = [
    {
        "title":    "Throughput — Simulation Generators (log-normalized)",
        "out":      "benchmarks-radar-fast.svg",
        # sysv_rand(441) → r=70, WyRand(3120) → r=270
        "A": 235.3, "B": -552.1,
        # Spokes clockwise from top.
        # mrand48 included as the fastest "BAD" generator — it beats several
        # modern designs, which is the whole point.
        # sysv_rand is the floor reference for the classic 15-bit LCG.
        "generators": [
            ("WyRand",         3120.0, "wyrand.bench"),
            ("JSF64",          1314.0, "jsf64.bench"),
            ("SFC64",          1262.0, "sfc64.bench"),
            ("Xoshiro256",     1287.0, "xoshiro256ss.bench"),
            ("PCG32",           934.1, "pcg32.bench"),
            ("Xoroshiro128",    902.8, "xoroshiro128ss.bench"),
            ("mrand48",         973.0, "rand48.bench"),
            ("PCG64",           843.8, "pcg64.bench"),
            ("MT19937",         641.2, "mt19937.bench"),
            ("Xorshift64",      646.1, "xorshift64.bench"),
            ("Xorshift32",      647.7, "xorshift32.bench"),
            ("sysv_rand",       441.4, "sysv_rand.bench"),
        ],
        "labels": [
            ("middle",  0, -42, 18),  # 0°   WyRand
            ("start",  18, -24, 18),  # 30°  JSF64
            ("start",  22, -20, 18),  # 60°  SFC64
            ("start",   8, -10, 18),  # 90°  Xoshiro256**
            ("start",  22,  14, 18),  # 120° PCG32
            ("start",  18,  16, 18),  # 150° Xoroshiro128**
            ("middle",  0,  28, 18),  # 180° mrand48
            ("end",   -18,  16, 18),  # 210° PCG64
            ("end",   -22,   7, 18),  # 240° MT19937
            ("end",    -8, -10, 18),  # 270° Xorshift64
            ("end",   -22, -20, 18),  # 300° Xorshift32
            ("end",   -18, -28, 18),  # 330° sysv_rand
        ],
    },
    {
        "title":    "Throughput — Slow Generators (log-normalized)",
        "out":      "benchmarks-radar-slow.svg",
        # Blum-Micali(0.462) → r=70, Squidward(240) → r=270
        "A": 73.6, "B": 94.7,
        # FreeBSD rand_r and ANSI C LCG are included because they land near
        # ChaCha20 in throughput — same speed, opposite quality.
        "generators": [
            ("Squidward",       239.6, "squidward.bench"),
            ("FreeBSD rand_r",  189.1, "bsd_rand_compat.bench"),
            ("ChaCha20",        170.7, "chacha20.bench"),
            ("ANSI C LCG",      186.7, "ansi_c_lcg.bench"),
            ("AES-128-CTR",     137.8, "aes_ctr.bench"),
            ("SpongeBob",        32.22, "spongebob.bench"),
            ("BBS",              61.29, "bbs.bench"),
            ("Hash_DRBG",        12.37, "hash_drbg.bench"),
            ("HMAC_DRBG",         3.218, "hmac_drbg.bench"),
            ("CtrDrbgAes256",     1.893, "crypto_ctr_drbg.bench"),
            ("OsRng",             1.191, "osrng.bench"),
            ("Blum-Micali",       0.462, "blum_micali.bench"),
        ],
        "labels": [
            ("middle",  0, -42, 18),  # 0°   Squidward
            ("start",  18, -24, 18),  # 30°  FreeBSD rand_r
            ("start",  22, -20, 18),  # 60°  ChaCha20
            ("start",   8, -10, 18),  # 90°  ANSI C LCG
            ("start",  22,  14, 18),  # 120° AES-128-CTR
            ("start",  18,  16, 18),  # 150° SpongeBob
            ("middle",  0,  28, 18),  # 180° BBS
            ("end",   -18,  16, 18),  # 210° Hash_DRBG
            ("end",   -22,   7, 18),  # 240° HMAC_DRBG
            ("end",    -8, -10, 18),  # 270° CtrDrbgAes256
            ("end",   -22, -20, 18),  # 300° OsRng
            ("end",   -18, -28, 18),  # 330° Blum-Micali
        ],
    },
    {
        "title":    "Throughput — Cipher-Based Generators (log-normalized)",
        "out":      "benchmarks-radar-cipher.svg",
        # Serpent-CTR(2.859) → r=70, Rabbit(352.6) → r=270
        "A": 96.0, "B": 25.6,
        # Spokes clockwise from top: stream ciphers (fast) followed by
        # block-CTR ciphers (slower).  AES-128-CTR included for reference.
        "generators": [
            ("Rabbit",          352.6, "rabbit.bench"),
            ("Salsa20",         201.0, "salsa20.bench"),
            ("Snow3G",          136.4, "snow3g.bench"),
            ("ZUC-128",         142.5, "zuc128.bench"),
            ("AES-128-CTR",     137.8, "aes_ctr.bench"),
            ("Camellia-CTR",     36.21, "camellia_ctr.bench"),
            ("SM4-CTR",          47.2,  "sm4_ctr.bench"),
            ("CAST-128-CTR",     59.5,  "cast128_ctr.bench"),
            ("SEED-CTR",         18.51, "seed_ctr.bench"),
            ("Grasshopper-CTR",   6.723, "grasshopper_ctr.bench"),
            ("Twofish-CTR",       3.512, "twofish_ctr.bench"),
            ("Serpent-CTR",       2.859, "serpent_ctr.bench"),
        ],
        "labels": [
            ("middle",  0, -42, 18),  # 0°   Rabbit
            ("start",  18, -24, 18),  # 30°  Salsa20
            ("start",  22, -20, 18),  # 60°  Snow3G
            ("start",   8, -10, 18),  # 90°  ZUC-128
            ("start",  22,  14, 18),  # 120° AES-128-CTR
            ("start",  18,  16, 18),  # 150° Camellia-CTR
            ("middle",  0,  28, 18),  # 180° SM4-CTR
            ("end",   -18,  16, 18),  # 210° CAST-128-CTR
            ("end",   -22,   7, 18),  # 240° SEED-CTR
            ("end",    -8, -10, 18),  # 270° Grasshopper-CTR
            ("end",   -22, -20, 18),  # 300° Twofish-CTR
            ("end",   -18, -28, 18),  # 330° Serpent-CTR
        ],
    },
]

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def spoke_xy(r, deg):
    rad = math.radians(deg)
    return CX + r * math.sin(rad), CY - r * math.cos(rad)

def ring_polygon(r):
    pts = [spoke_xy(r, k * 360 / N_SPOKES) for k in range(N_SPOKES)]
    return " ".join(f"{x:.1f},{y:.1f}" for x, y in pts)

def log_r(mw_s, A, B):
    return A * math.log10(mw_s) + B

def parse_bench(path):
    try:
        text = path.read_text()
        m = re.search(r'\|\s*[^|]+\|\s*([\d.e+]+)\s*\|', text)
        return float(m.group(1)) if m else None
    except OSError:
        return None

def fmt_mw(mw_s):
    if mw_s >= 100:
        return f"{round(mw_s)} MW/s" if abs(mw_s - round(mw_s)) < 0.05 else f"{mw_s:.1f} MW/s"
    return f"{mw_s:.4g} MW/s"

def load_machine_data(chart, stats_root, subdir):
    """Return [(label, mw_s, is_measured), ...] for this machine, or None if no bench files found.

    is_measured is True when the value came from a bench file, False when the
    hard-coded reference fallback was substituted.  Prints a warning to stderr
    for each fallback substitution.
    """
    machine_dir = stats_root / subdir
    result, found_any = [], False
    for label, fallback, bench_file in chart["generators"]:
        measured = parse_bench(machine_dir / bench_file) if machine_dir.exists() else None
        if measured is not None:
            found_any = True
            result.append((label, measured, True))
        else:
            print(f"  warning: {subdir}/{bench_file} missing — using reference fallback "
                  f"({fallback:.4g} MW/s) for {label!r}", file=sys.stderr)
            result.append((label, fallback, False))
    return result if found_any else None

# ---------------------------------------------------------------------------
# SVG generation
# ---------------------------------------------------------------------------

def generate_svg(chart, stats_root, machine_cache):
    """Render one radar chart SVG.

    machine_cache maps subdir → load_machine_data() result (or None).
    The caller populates it once per chart so that the polygon loop, the
    title/subtitle block, and any external reporting loop all share the same
    loaded data without re-reading bench files or re-emitting warnings.
    """
    A, B = chart["A"], chart["B"]
    lines = []
    w = lines.append

    w(f'<svg xmlns="http://www.w3.org/2000/svg" width="{CANVAS}" height="{CANVAS}" viewBox="0 0 {CANVAS} {CANVAS}">')
    w(f'<rect width="100%" height="100%" fill="#f6f1e8"/>')

    # Grid rings
    w('<g stroke="#d8cdb8" fill="none">')
    for r in RINGS:
        w(f'<polygon points="{ring_polygon(r)}" stroke-width="1"/>')
    w('</g>')

    # Spokes
    w('<g stroke="#b8aa90" stroke-width="1">')
    for k in range(N_SPOKES):
        x2, y2 = spoke_xy(SPOKE_R, k * 360 / N_SPOKES)
        w(f'<line x1="{CX:.0f}" y1="{CY:.0f}" x2="{x2:.1f}" y2="{y2:.1f}"/>')
    w('</g>')

    # One polygon + dots per machine.
    # Fallback (unmeasured) points are rendered with a dashed polygon stroke
    # and hollow circles so the viewer can distinguish them from measured data.
    legend_entries = []
    label_data = None
    for subdir, fill, stroke, dot_fill, legend_label in MACHINES:
        data = machine_cache.get(subdir)
        if data is None:
            continue
        if label_data is None:
            label_data = data

        radii  = [log_r(mw_s, A, B) for _, mw_s, _ in data]
        pts_xy = [spoke_xy(r, k * 360 / N_SPOKES) for k, r in enumerate(radii)]
        pts_str = " ".join(f"{x:.1f},{y:.1f}" for x, y in pts_xy)
        has_fallback = any(not m for _, _, m in data)
        dash_attr = ' stroke-dasharray="8 4"' if has_fallback else ''

        w(f'<polygon points="{pts_str}" fill="{fill}" stroke="{stroke}" stroke-width="3"{dash_attr}/>')
        w('<g>')
        for (_, _, is_measured), (x, y) in zip(data, pts_xy):
            if is_measured:
                w(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="4.5" fill="{dot_fill}"/>')
            else:
                # Hollow circle marks a fallback (reference) value.
                w(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="4.5" fill="#f6f1e8" stroke="{dot_fill}" stroke-width="2"/>')
        w('</g>')
        legend_entries.append((fill, stroke, legend_label, has_fallback))

    # Labels (from first machine with data).
    # Fallback throughput values are annotated with † to indicate they are
    # reference constants, not measurements from this machine.
    if label_data:
        w('<g font-family="Georgia, serif" font-size="17" fill="#3f3426">')
        for k, ((label, mw_s, is_measured), (anchor, dx, dy, dy2)) in enumerate(
                zip(label_data, chart["labels"])):
            tx, ty = spoke_xy(SPOKE_R, k * 360 / N_SPOKES)
            lx, ly1 = tx + dx, ty + dy
            w(f'<text x="{lx:.1f}" y="{ly1:.1f}" text-anchor="{anchor}">{label}</text>')
            annotation = fmt_mw(mw_s) + ("†" if not is_measured else "")
            w(f'<text x="{lx:.1f}" y="{ly1 + dy2:.1f}" text-anchor="{anchor}" font-size="13" fill="#7a6850">{annotation}</text>')
        w('</g>')

    # Legend (only when more than one machine is plotted)
    if len(legend_entries) > 1:
        lx, ly = 30, CANVAS - 30 - len(legend_entries) * 22
        w('<g font-family="Georgia, serif" font-size="14" fill="#3f3426">')
        for i, (fill, stroke, lbl, has_fallback) in enumerate(legend_entries):
            y = ly + i * 22
            dash_attr = ' stroke-dasharray="4 2"' if has_fallback else ''
            w(f'<rect x="{lx}" y="{y - 12}" width="18" height="14" fill="{fill}" stroke="{stroke}" stroke-width="2"{dash_attr}/>')
            suffix = " (partial data†)" if has_fallback else ""
            w(f'<text x="{lx + 24}" y="{y}">{lbl}{suffix}</text>')
        w('</g>')

    # Title / subtitle
    machines_present = [m[0] for m in MACHINES if machine_cache.get(m[0]) is not None]
    subtitle = " · ".join(machines_present) + " · pilot-bench normal preset"
    any_fallback = any(has_fb for *_, has_fb in legend_entries)
    w(f'<text x="{CX:.0f}" y="52" text-anchor="middle" font-family="Georgia, serif" font-size="22" fill="#2f2418">{chart["title"]}</text>')
    w(f'<text x="{CX:.0f}" y="76" text-anchor="middle" font-family="Georgia, serif" font-size="14" fill="#6f5d46">{subtitle}</text>')
    if any_fallback:
        w(f'<text x="{CX:.0f}" y="{CANVAS - 12}" text-anchor="middle" font-family="Georgia, serif" font-size="12" fill="#9a7a5a">† reference value — not measured on this machine</text>')
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
    args = ap.parse_args()

    stats_root = Path(args.stats)
    for chart in CHARTS:
        # Load bench data once per (chart, machine) pair; reuse for SVG
        # rendering, title/subtitle, and the CLI summary below.
        machine_cache = {
            subdir: load_machine_data(chart, stats_root, subdir)
            for subdir, *_ in MACHINES
        }

        svg  = generate_svg(chart, stats_root, machine_cache)
        out  = REPO / "assets" / chart["out"]
        out.write_text(svg)
        print(f"Wrote {out}")
        for subdir, *_ in MACHINES:
            data = machine_cache.get(subdir)
            if data is None:
                print(f"  {subdir}: no bench files, skipped")
                continue
            print(f"  {subdir}:")
            for label, mw_s, is_measured in data:
                flag = "" if is_measured else "†"
                print(f"    {label:18s}  {mw_s:8.3g} MW/s  r={log_r(mw_s, chart['A'], chart['B']):.1f}{flag}")

if __name__ == "__main__":
    main()
