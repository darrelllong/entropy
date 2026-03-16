# Response to Peer Review (2026-03-16)

All six issues identified in the peer review have been addressed in commit `7300f8b`.

---

## P2 — Stale battery counts and excursion math

**Fixed.**

`scripts/parse_battery.py` corrected:

- 742 → **738** total slots (199 NIST + 17 DIEHARD + 522 DIEHARDER)
- `random_excursions_variant` state count: 16 → **18** states `{-9..-1, 1..9}`
- Skip description: "24 per-state results skipped" → "families emit 8 + 18 = 26 individual results; both replaced by 1 family-level SKIP each → **26 − 2 = 24 fewer slots**"
- J formula: `0.564√n` (one-sided) → `√(2n/π)` (correct symmetric-walk two-sided asymptotic); at n = 16,000,000 this gives J ≈ **3,192**, not 2,256

`TESTS.md` regenerated from the corrected script.

---

## P2 — Radar charts silently mix measured and fallback data

**Mitigated.**

`scripts/make_radar.py` now emits a `stderr` warning for every generator whose
bench file is absent and a reference fallback value is substituted:

```
warning: dyson/jsf64.bench missing — using reference fallback (1314 MW/s) for 'JSF64'
```

The fall-back behaviour is by design (it allows the chart to render when a machine
has only partial data), but it is no longer silent. Any chart regeneration with
missing bench files will produce visible warnings in the shell.

---

## P2 — Gorilla missing aggregate KS check

**Fixed.**

`src/research/marsaglia_tsang.rs` now exports:

```rust
pub fn gorilla_aggregate_ks(results: &[GorillaBitResult]) -> f64
```

which runs a KS uniformity test over the 32 per-bit p-values, as specified in
Marsaglia & Tsang (2002). The module doc comment has been updated to describe
both stages of the test.

`src/bin/gorilla.rs` now prints an `agg_ks_p` column alongside the existing
`min_p`, `max_p`, `worst_bit`, and `worst_|z|` columns.

---

## P3 — Mixed-width buffered reads discard bytes without documentation

**Fixed (documentation).**

`src/rng/squidward.rs`, `chacha20_rng.rs`, `hmac_drbg.rs`, and `hash_drbg.rs`
now carry the same caveat that `SpongeBob` already carried:

> For uniform-width access (all `next_u32` or all `next_u64`) all N bits per
> block are used; mixing widths at a refill boundary silently discards up to 7
> trailing bytes before refilling.

The behaviour itself is unchanged — the battery and benchmarks use `next_u32()`
uniformly and are unaffected.

---

## P3 — `webster_tavares` input-bits vs. seed-width mismatch

**Fixed.**

Each case in `src/bin/webster_tavares.rs` now carries an explicit `seed_bits`
annotation (32, 48, 64, 128, or 384). When `--input-bits` exceeds that value
the binary now warns:

```
warning: MT19937: --input-bits 48 exceeds RNG seed width (32 bits);
         bits 32..48 are silently truncated — results are misleading
```

The default `--input-bits 32` is safe for all 32-bit RNGs and produces no
warning.

---

## P3 — Pincus reference file is not a PDF

**Fixed — implementation removed.**

`pubs/pincus-1991-approximate-entropy.pdf` was a Cloudflare HTML challenge page
saved with a `.pdf` extension. Since the paper is unavailable locally and cannot
be audited, the entire Pincus multi-scale ApEn implementation has been removed:

- `src/research/pincus.rs` deleted
- `src/bin/bib_tests.rs` stripped of the Pincus import and the ApEn(m) output loop
- `BIB.md` `pincus1991approximate` entry removed
- `README.md` Pincus rows removed

The NIST SP 800-22 §2.12 `approximate_entropy` test is unaffected — it is a
distinct implementation drawn from the bundled NIST SP 800-22 Rev. 1a PDF.

---

## Workflow Gap — Extra tests not integrated into the main battery

**Acknowledged, not yet addressed.**

The extra probes (`bib_tests`, `upstream_tests`, `testu01_lz`, `webster_tavares`,
`gorilla`) remain standalone binaries. Integrating them into `run_tests` /
`TESTS.md` is a design task deferred to a future iteration. They provide useful
diagnostic information on demand but are not part of the standard published
battery.
