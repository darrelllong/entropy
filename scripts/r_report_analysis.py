#!/usr/bin/env python3
"""Read R-REPORT.md sections and compute summary numbers used by the
hand-written analysis: REJECT counts per RNG, max moment errors, mean
absolute moment error, spectral-flatness deviations, max-periodogram
spikes.  Prints a markdown block.

usage: scripts/r_report_analysis.py R-REPORT.md
"""
import math
import re
import sys

if len(sys.argv) != 2:
    sys.exit("usage: r_report_analysis.py <report.md>")

try:
    with open(sys.argv[1]) as fh:
        text = fh.read()
except OSError as exc:
    sys.exit(f"r_report_analysis: cannot read {sys.argv[1]}: {exc}")

# Strip leading summary block — only the per-RNG sections matter here.
m = re.search(r"^## ", text, flags=re.MULTILINE)
if m is None:
    sys.exit(f"r_report_analysis: no '## ' headings in {sys.argv[1]}")
body = text[m.start():]

# Keep only ## sections that contain a moments table (skip the inserted
# "## Summary" section, which has none).
chunks = re.split(r"^(## .+)$", body, flags=re.MULTILINE)
records = []
for i in range(1, len(chunks), 2):
    label = chunks[i].lstrip("# ").strip()
    chunk = chunks[i + 1]
    if "### Raw moments E[U^k]" not in chunk:
        continue
    # Sample size
    sz = re.search(r"Sample size:\s*([\d,]+)\s*u32", chunk)
    n = int(sz.group(1).replace(",", "")) if sz else None
    # Mean / Var line
    mv = re.search(r"Mean\s*=\s*([-+0-9.eE]+)\s+Var\s*=\s*([-+0-9.eE]+)", chunk)
    mean_obs = float(mv.group(1)) if mv else float("nan")
    var_obs = float(mv.group(2)) if mv else float("nan")
    # REJECT / pass / n/a tally — mirror r_report_summary.py logic.
    rej = passes = nas = 0
    for ln in chunk.splitlines():
        if not ln.startswith("|"):
            continue
        cells = [c.strip() for c in ln.strip("|").split("|")]
        if len(cells) != 4:
            continue
        test, _stat, _p, verdict = cells
        if test == "Test" or set(test) <= {"-"}:
            continue
        if "jarque.bera" in test:
            continue
        if test.startswith(("k ", "Metric")) or "|" in test:
            continue
        if verdict == "REJECT":
            rej += 1
        elif verdict == "pass":
            passes += 1
        elif verdict == "n/a":
            nas += 1
    # Moment errors.  Table has lines like:
    # | 1 | 0.50007 | 0.50000 | 7.12e-05 |
    moments = []
    for ln in chunk.splitlines():
        match = re.match(r"\|\s*(\d+)\s*\|\s*([-+0-9.eE]+)\s*\|\s*([-+0-9.eE]+)\s*\|\s*([-+0-9.eE]+)\s*\|", ln)
        if match:
            k = int(match.group(1))
            if 1 <= k <= 10:
                moments.append((k,
                                float(match.group(2)),
                                float(match.group(3)),
                                float(match.group(4))))
    # Spectral block.
    fl = re.search(r"Spectral flatness \(Wiener entropy\) \| ([-+0-9.eE]+)", chunk)
    flat = float(fl.group(1)) if fl else None
    pmax = re.search(r"max normalized periodogram \(P_max\) \| ([-+0-9.eE]+)", chunk)
    pmax = float(pmax.group(1)) if pmax else None
    pspike = re.search(r"Bonferroni p \(no spike\) \| (\S+)", chunk)
    if pspike:
        v = pspike.group(1).replace("<", "")
        try:
            pspike_v = float(v)
        except ValueError:
            pspike_v = float("nan")
    else:
        pspike_v = None
    p_chi = re.search(r"Periodogram chi\^2 \(.*?\) \| chi2=([-+0-9.eE]+),\s*p=(\S+)", chunk)
    chi2_p = float(p_chi.group(2)) if p_chi else None
    p_ks = re.search(r"Periodogram KS vs Exp\(1\) \| D=([-+0-9.eE]+),\s*p=(\S+)", chunk)
    ks_p = float(p_ks.group(2)) if p_ks else None

    records.append({
        "label": label,
        "n": n,
        "mean": mean_obs,
        "var": var_obs,
        "rej": rej,
        "pass": passes,
        "na": nas,
        "moments": moments,
        "flatness": flat,
        "pmax": pmax,
        "pspike": pspike_v,
        "chi2_p": chi2_p,
        "ks_p": ks_p,
    })

# ---- print derived stats -----------------------------------------------------
print("RNG, REJECTs, passes, n/a, mean, var,",
      "max_moment_err, mean_moment_err, flatness, P_max, p_spike, chi2_p, ks_p")
for r in records:
    abs_errs = [m[3] for m in r["moments"]]
    max_err = max(abs_errs) if abs_errs else float("nan")
    mean_err = sum(abs_errs) / len(abs_errs) if abs_errs else float("nan")
    print(f"{r['label']!r}, {r['rej']}, {r['pass']}, {r['na']}, "
          f"{r['mean']:.6e}, {r['var']:.6e}, "
          f"{max_err:.3e}, {mean_err:.3e}, "
          f"{r['flatness']!r}, {r['pmax']!r}, {r['pspike']!r}, "
          f"{r['chi2_p']!r}, {r['ks_p']!r}")
