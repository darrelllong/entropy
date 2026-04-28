#!/usr/bin/env python3
"""Insert a per-RNG REJECT-count summary just below the front-matter of R-REPORT.md.

usage: scripts/r_report_summary.py R-REPORT.md
"""
import re
import sys

if len(sys.argv) != 2:
    sys.exit("usage: r_report_summary.py <report.md>")

path = sys.argv[1]
with open(path) as fh:
    text = fh.read()

# Split into front matter (everything before the first '## ') and body.
m = re.search(r"^## ", text, flags=re.MULTILINE)
if not m:
    sys.exit("no '## ' section found")
front, body = text[: m.start()], text[m.start() :]

# Walk each '## <label>' section.
sections = re.split(r"^(## .+)$", body, flags=re.MULTILINE)
# split() returns ['', heading1, content1, heading2, content2, ...]
rows = []
for i in range(1, len(sections), 2):
    heading = sections[i].lstrip("# ").strip()
    chunk = sections[i + 1]
    # tally REJECT/pass/n/a, ignoring the Jarque-Bera row (always REJECT)
    rejects = 0
    passes = 0
    nas = 0
    jb_skipped = False
    for line in chunk.splitlines():
        if not line.startswith("|"):
            continue
        cols = [c.strip() for c in line.strip("|").split("|")]
        if len(cols) != 4:
            continue
        test, _stat, _p, verdict = cols
        if test in ("Test",) or set(test) <= {"-"}:
            continue
        if test.startswith("k "):
            continue
        if "jarque.bera" in test:
            jb_skipped = True
            continue
        if verdict == "REJECT":
            rejects += 1
        elif verdict == "pass":
            passes += 1
        elif verdict == "n/a":
            nas += 1
    rows.append((heading, rejects, passes, nas, jb_skipped))

# Build summary block.
out = ["## Summary — REJECT counts at α = 0.001\n",
       ("Counts exclude `tseries::jarque.bera.test`, which is a Normality "
        "test and is **expected to REJECT** for any uniform stream.\n"),
       "| RNG | REJECTs | Passes | n/a |",
       "|-----|---------|--------|-----|"]
for h, r, p, n, _ in rows:
    out.append(f"| {h} | {r} | {p} | {n} |")
summary = "\n".join(out) + "\n\n---\n\n"

# Replace any prior summary block, then re-insert.
front = re.sub(r"## Summary —.*?---\n\n", "", front, flags=re.DOTALL)

with open(path, "w") as fh:
    fh.write(front + summary + body)

print(f"[done] {len(rows)} RNG rows summarised in {path}")
