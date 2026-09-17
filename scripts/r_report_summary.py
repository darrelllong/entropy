#!/usr/bin/env python3
"""Insert a per-RNG pass/fail/invalid summary just below the front matter of R-REPORT.md.

Exit status: 0 when no result is invalid, 2 when any is (the summary is still
written).

usage: scripts/r_report_summary.py R-REPORT.md
"""
import re
import sys

if len(sys.argv) != 2:
    sys.exit("usage: r_report_summary.py <report.md>")

path = sys.argv[1]
try:
    with open(path) as fh:
        text = fh.read()
except OSError as exc:
    sys.exit(f"r_report_summary: cannot read {path}: {exc}")

# Strip any previous summary block first so re-runs are idempotent.  The
# block ends with a `---\n\n` separator before the first per-RNG `##` heading.
text = re.sub(
    r"## Summary —[^\n]*\n.*?\n---\n\n(?=^## )",
    "", text, flags=re.DOTALL | re.MULTILINE
)

m = re.search(r"^## ", text, flags=re.MULTILINE)
if not m:
    sys.exit("no '## ' section found")
front, body = text[: m.start()], text[m.start() :]

# Each RNG section ends with the analysis's own outcome line.
OUTCOME = re.compile(r"^\*\*Outcome\*\*: (\d+) pass, (\d+) fail, (\d+) invalid(.*)$",
                     re.MULTILINE)
sections = re.split(r"^(## .+)$", body, flags=re.MULTILINE)
rows = []
for i in range(1, len(sections), 2):
    heading = sections[i].lstrip("# ").strip()
    chunk = sections[i + 1]
    if "### Raw moments E[U^k]" not in chunk:
        continue
    outcome = OUTCOME.search(chunk)
    if outcome is None:
        # A section without its outcome line did not finish.
        rows.append((heading, "?", "?", "no outcome"))
        continue
    passes, fails, invalid, rest = outcome.groups()
    detail = invalid if not rest.strip(" ()") else f"{invalid}{rest.split(' (Jarque')[0]}"
    rows.append((heading, passes, fails, detail))

out = ["## Summary — pass, fail and invalid results at α = 0.001\n",
       ("Fail counts exclude `tseries::jarque.bera.test`, a Normality test that "
        "uniform streams are expected to fail.  An invalid result is an error "
        "or a p-value that is not a probability; it is not a pass.\n"),
       "| RNG | Pass | Fail | Invalid |",
       "|-----|------|------|---------|"]
for h, p, f, n in rows:
    out.append(f"| {h} | {p} | {f} | {n} |")
summary = "\n".join(out) + "\n\n---\n\n"

with open(path, "w") as fh:
    fh.write(front + summary + body)

bad = [h for h, p, f, n in rows if not n.startswith("0")]
print(f"[done] {len(rows)} RNG rows summarised in {path}")
if bad:
    print(f"[invalid] {len(bad)} generator(s) with invalid or unfinished results: {', '.join(bad)}",
          file=sys.stderr)
    sys.exit(2)
