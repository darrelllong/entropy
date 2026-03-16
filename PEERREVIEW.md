# Peer Review
*2026-03-16 follow-up to `RESPONSE.md`*

This follow-up reviewed the claims in `RESPONSE.md` against the tree after
commit `7300f8b`.

**All findings are now closed.**  See `RESPONSE.md` for the full resolution log.

---

## Resolved Findings

### P2 — The stale excursion-count comment in `src/main.rs` ✓

**Closed.**  `src/main.rs` now says `~3 191 (= √(2n/π))` as documented in
`RESPONSE.md`.

---

### P2 — Extra probes not integrated into the standard battery ✓

**Closed.**  `tests/run_aux.sh` and `tests/run_all.sh` now build and run all
five auxiliary probes (Knuth + ApEn, TestU01 Hamming, TestU01 Lempel-Ziv,
Webster-Tavares, Gorilla) as part of the canonical full-audit path.
`TESTS.md` carries an `## Auxiliary Probes` section preserved across
regenerations.  `README.md` documents `tests/run_all.sh` as the primary
audit command.

---

### P3 — Radar charts mixed measured and fallback values silently ✓

**Closed.**  `scripts/make_radar.py` now renders fallback data visibly:
dashed polygon stroke, hollow circle markers, `†` throughput labels, legend
`(partial data†)` suffix, and a bottom footnote.  A `stderr` warning is also
emitted at generation time.

---

### P3 — `BIB.md` cited a missing local shelf file ✓

**Closed.**  The `[pubs/doganaksoy-gologlu-2006-bent-functions.pdf]` local-file
reference was removed from the `BIB.md` entry.  DOI and bibliographic metadata
are retained.
