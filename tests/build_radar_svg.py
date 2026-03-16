#!/usr/bin/env python3
"""Build an SVG radar chart from stats/*.bench rows."""

from __future__ import annotations

import argparse
import math
import re
from dataclasses import dataclass
from pathlib import Path


ROW_RE = re.compile(
    r"^\|\s*(?P<label>.*?)\s*\|\s*(?P<mean>[0-9]+(?:\.[0-9]+)?)\s*\|\s*(?P<ci>[^\|]+)\|\s*(?P<runs>[^\|]+)\|$"
)


@dataclass
class BenchRow:
    slug: str
    label: str
    mean: float
    ci: str
    runs: str


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description="Build an SVG radar chart from stats/*.bench rows."
    )
    p.add_argument(
        "--stats-dir",
        default="stats",
        help="Directory containing *.bench files (default: stats)",
    )
    p.add_argument(
        "--output",
        required=True,
        help="Output SVG path",
    )
    p.add_argument(
        "--title",
        default="RNG throughput radar",
        help="Chart title",
    )
    p.add_argument(
        "--subtitle",
        default="log10(MW/s) normalization",
        help="Optional subtitle",
    )
    p.add_argument(
        "--include",
        nargs="*",
        default=[],
        help="Optional whitelist of stat slugs (stats/<slug>.bench)",
    )
    p.add_argument(
        "--exclude",
        nargs="*",
        default=[],
        help="Optional blacklist of stat slugs",
    )
    p.add_argument(
        "--rings",
        type=int,
        default=5,
        help="Number of background rings (default: 5)",
    )
    p.add_argument(
        "--width",
        type=int,
        default=900,
        help="SVG width (default: 900)",
    )
    p.add_argument(
        "--height",
        type=int,
        default=900,
        help="SVG height (default: 900)",
    )
    return p.parse_args()


def load_rows(stats_dir: Path) -> list[BenchRow]:
    rows: list[BenchRow] = []
    for path in sorted(stats_dir.glob("*.bench")):
        line = path.read_text(encoding="utf-8").strip()
        if not line:
            continue
        m = ROW_RE.match(line)
        if not m:
            raise SystemExit(f"could not parse bench row in {path}: {line!r}")
        rows.append(
            BenchRow(
                slug=path.stem,
                label=m.group("label").strip(),
                mean=float(m.group("mean")),
                ci=m.group("ci").strip(),
                runs=m.group("runs").strip(),
            )
        )
    if not rows:
        raise SystemExit(f"no .bench rows found under {stats_dir}")
    return rows


def filter_rows(rows: list[BenchRow], include: list[str], exclude: list[str]) -> list[BenchRow]:
    out = rows
    if include:
        wanted = {item.strip() for item in include if item.strip()}
        out = [row for row in out if row.slug in wanted]
        missing = sorted(wanted - {row.slug for row in out})
        if missing:
            raise SystemExit(f"missing stats rows for: {', '.join(missing)}")
        order = {slug: i for i, slug in enumerate(include)}
        out.sort(key=lambda row: order[row.slug])
    if exclude:
        blocked = {item.strip() for item in exclude if item.strip()}
        out = [row for row in out if row.slug not in blocked]
    if not out:
        raise SystemExit("no rows left after include/exclude filtering")
    return out


def polar_point(cx: float, cy: float, radius: float, theta: float) -> tuple[float, float]:
    return (cx + radius * math.cos(theta), cy + radius * math.sin(theta))


def format_points(points: list[tuple[float, float]]) -> str:
    return " ".join(f"{x:.1f},{y:.1f}" for x, y in points)


def escape_xml(text: str) -> str:
    return (
        text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


def build_svg(rows: list[BenchRow], title: str, subtitle: str, width: int, height: int, rings: int) -> str:
    cx = width / 2.0
    cy = height / 2.0
    margin = 165.0
    radius = min(width, height) / 2.0 - margin
    label_radius = radius + 70.0
    n = len(rows)
    if n < 3:
        raise SystemExit("need at least 3 stats rows to build a radar chart")

    values = [row.mean for row in rows]
    logs = [math.log10(v) for v in values]
    lo = min(logs)
    hi = max(logs)
    if math.isclose(lo, hi):
        lo -= 1.0
        hi += 1.0
    pad = 0.05 * (hi - lo)
    lo -= pad
    hi += pad

    def scaled(v: float) -> float:
        return (math.log10(v) - lo) / (hi - lo)

    angles = [(-math.pi / 2.0) + 2.0 * math.pi * i / n for i in range(n)]

    background = []
    for ring_idx in range(1, rings + 1):
        frac = ring_idx / rings
        ring_r = radius * frac
        pts = [polar_point(cx, cy, ring_r, theta) for theta in angles]
        background.append(
            f'<polygon points="{format_points(pts)}" fill="none" stroke="#d8ccb8" stroke-width="1"/>'
        )

    axes = []
    for theta in angles:
        x, y = polar_point(cx, cy, radius, theta)
        axes.append(f'<line x1="{cx:.1f}" y1="{cy:.1f}" x2="{x:.1f}" y2="{y:.1f}" stroke="#c7baa5" stroke-width="1"/>')

    data_pts = [
        polar_point(cx, cy, radius * scaled(row.mean), theta)
        for row, theta in zip(rows, angles)
    ]
    polygon = (
        f'<polygon points="{format_points(data_pts)}" fill="#c86b3c55" '
        f'stroke="#9f4b1b" stroke-width="3"/>'
    )

    labels = []
    for row, theta in zip(rows, angles):
        x, y = polar_point(cx, cy, label_radius, theta)
        anchor = "middle"
        if math.cos(theta) > 0.25:
            anchor = "start"
        elif math.cos(theta) < -0.25:
            anchor = "end"
        labels.append(
            f'<text x="{x:.1f}" y="{y:.1f}" text-anchor="{anchor}" font-size="14" fill="#3f3428">{escape_xml(row.label)}</text>'
        )

    value_labels = []
    for row, theta in zip(rows, angles):
        x, y = polar_point(cx, cy, label_radius - 24.0, theta)
        anchor = "middle"
        if math.cos(theta) > 0.25:
            anchor = "start"
        elif math.cos(theta) < -0.25:
            anchor = "end"
        value_labels.append(
            f'<text x="{x:.1f}" y="{y:.1f}" text-anchor="{anchor}" font-size="13" fill="#7a6850">{row.mean:g} MW/s</text>'
        )

    legend = [
        '<rect x="44" y="58" width="18" height="18" fill="#c86b3c55" stroke="#9f4b1b" stroke-width="2"/>',
        '<text x="72" y="72" font-size="14" fill="#3f3428">throughput profile</text>',
        f'<text x="44" y="98" font-size="14" fill="#7a6850">scale: log10(MW/s), range {10**lo:.3g} to {10**hi:.3g}</text>',
    ]

    svg = f'''<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
<rect width="{width}" height="{height}" fill="#f5efe6"/>
<text x="{cx:.1f}" y="42" text-anchor="middle" font-size="26" font-weight="700" fill="#2f261d">{escape_xml(title)}</text>
<text x="{cx:.1f}" y="70" text-anchor="middle" font-size="15" fill="#7a6850">{escape_xml(subtitle)}</text>
{chr(10).join(background)}
{chr(10).join(axes)}
{polygon}
{chr(10).join(labels)}
{chr(10).join(value_labels)}
{chr(10).join(legend)}
</svg>
'''
    return svg


def main() -> int:
    args = parse_args()
    stats_dir = Path(args.stats_dir)
    rows = filter_rows(load_rows(stats_dir), args.include, args.exclude)
    svg = build_svg(rows, args.title, args.subtitle, args.width, args.height, args.rings)
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(svg, encoding="utf-8")
    print(f"wrote {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
