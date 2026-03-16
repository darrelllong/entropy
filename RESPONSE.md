# Response to Peer Review (2026-03-16)

All findings from both the original review and the follow-up are now closed.

---

## P2 — Stale battery counts and excursion math

**Fixed.**

`scripts/parse_battery.py` corrected:

- 742 → **738** total slots (199 NIST + 17 DIEHARD + 522 DIEHARDER)
- `random_excursions_variant` state count: 16 → **18** states `{-9..-1, 1..9}`
- Skip description: "24 per-state results skipped" → "families emit 8 + 18 = 26 individual results; both replaced by 1 family-level SKIP each → **26 − 2 = 24 fewer slots**"
- J formula: `0.564√n` (one-sided) → `√(2n/π)` (correct symmetric-walk two-sided asymptotic); at n = 16,000,000 this gives J ≈ **3,191**, not 2,256

`src/main.rs` comment updated from `~2 256` to `~3 191 (= √(2n/π))`.

`TESTS.md` regenerated from the corrected script.

---

## P2 — Radar charts silently mix measured and fallback data

**Fixed.**

`scripts/make_radar.py` now annotates fallback (unmeasured) data visibly in the
generated SVG:

- **Dashed polygon stroke** for any machine whose data contains at least one fallback value
- **Hollow circle markers** at fallback data points (filled circles remain for measured points)
- **`†` suffix** on the throughput label for each fallback generator
- **Legend suffix** `(partial data†)` when a machine has any fallback values
- **Bottom footnote** in the chart: `† reference value — not measured on this machine`

A `stderr` warning is also printed for each substitution at generation time.

---

## P2 — Gorilla missing aggregate KS check

**Fixed.**

`src/research/marsaglia_tsang.rs` now exports `gorilla_aggregate_ks()`, which
runs a KS uniformity test over the 32 per-bit p-values as specified in
Marsaglia & Tsang (2002). `src/bin/gorilla.rs` prints an `agg_ks_p` column.

---

## P3 — Mixed-width buffered reads discard bytes without documentation

**Fixed (documentation).**

`src/rng/squidward.rs`, `chacha20_rng.rs`, `hmac_drbg.rs`, and `hash_drbg.rs`
now carry the same caveat that `SpongeBob` already had: mixing `next_u32` and
`next_u64` calls at a refill boundary silently discards up to 7 trailing bytes.

---

## P3 — `webster_tavares` input-bits vs. seed-width mismatch

**Fixed.**

Each case now carries an explicit `seed_bits` annotation. When `--input-bits`
exceeds that value the binary warns on `stderr`. The default `--input-bits 32`
is safe for all 32-bit RNGs.

---

## P3 — Pincus reference file is not a PDF

**Fixed — implementation replaced.**

The bogus `pubs/pincus-1991-approximate-entropy.pdf` (a Cloudflare HTML page)
was deleted. The multi-scale ApEn sweep was re-implemented as
`src/research/approx_entropy.rs`, citing NIST SP 800-22 §2.12 — the actual
specification the code follows, present in the bundled `pubs/NIST-SP-800-22r1a.pdf`.
`BIB.md`, `README.md`, and `bib_tests.rs` updated accordingly.

---

## P3 — `BIB.md` cites missing doganaksoy local file

**Fixed.**

The `[pubs/doganaksoy-gologlu-2006-bent-functions.pdf]` local-file reference
was removed from the BIB entry. The DOI and bibliographic metadata are retained.

---

## Workflow Gap — Extra probes not in the standard battery

**Fixed.**

Two new scripts integrate all five auxiliary probes into the standard audit path:

**`tests/run_aux.sh`** — builds and runs the five probes with their default
parameters, printing section-delimited output to stdout.

**`tests/run_all.sh`** — the new canonical full-audit entry point.  Builds all
binaries, runs the main NIST/DIEHARD/DIEHARDER battery, then runs all five
auxiliary probes.  Tees everything to a timestamped `/tmp/run_all-<date>.log`.

`README.md` updated to document `tests/run_all.sh` as the primary audit command.

`TESTS.md` now contains an `## Auxiliary Probes` section with the results of
the most recent `run_aux.sh` run (darby.local, 2026-03-15).  This section is
preserved by `scripts/parse_battery.py` across future battery regenerations.
