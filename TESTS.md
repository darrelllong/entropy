# Full Battery Results

Full `run_tests` battery harvested from `darby.local` on 2026-03-15 from the
local `darby-full-battery.log`.

Command:

```sh
./target/release/run_tests \
  --rng OsRng \
  --rng MT19937 \
  --rng Xorshift64 \
  --rng Xorshift32 \
  --rng "BAD Unix" \
  --rng "BAD Windows" \
  --rng "ANSI C" \
  --rng MINSTD \
  --rng AES-128-CTR \
  --rng cryptography::CtrDrbgAes256 \
  --rng Constant \
  --rng Counter \
  --rng Dual_EC
```

Excluded on purpose:

- `BBS`
- `Blum-Micali`

Notes:

- The active battery is much larger than the old `225`-result runs because
  faithful `rgb_bitdist` now emits its full per-pattern family.
- `738` results means the full active battery plus the always-skipped
  Maurer `L=13..16` rows.
- `714` results means the RNG also missed the NIST excursion-family
  precondition, so those two families collapsed to one skip each.
- `199` results means `Dual_EC_DRBG` ran the NIST and Maurer families only.
- With `714`-`738` tests, a genuinely good generator should still expect about
  `7` low single-test p-values by chance at `α = 0.01`.

## Summary Table

| RNG | Total | PASS | FAIL | SKIP |
|---|---:|---:|---:|---:|
| OsRng (/dev/urandom) | 738 | 724 | 10 | 4 |
| MT19937 (seed=19650218) | 738 | 728 | 6 | 4 |
| Xorshift64 (seed=1) | 714 | 704 | 4 | 6 |
| Xorshift32 (seed=1) | 738 | 721 | 13 | 4 |
| BAD Unix System V rand() (15-bit LCG, seed=1) | 738 | 721 | 13 | 4 |
| BAD Unix System V mrand48() (seed=1) | 738 | 725 | 9 | 4 |
| BAD Unix BSD random() TYPE_3 (seed=1) | 738 | 732 | 2 | 4 |
| BAD Unix Linux glibc rand()/random() (seed=1) | 738 | 732 | 2 | 4 |
| BAD Unix FreeBSD12 rand_r() compat (seed=1) | 738 | 719 | 15 | 4 |
| BAD Windows CRT rand() (MSVC/UCRT lineage, seed=1) | 714 | 705 | 3 | 6 |
| BAD Windows VB6/VBA Rnd() (project seed=1) | 738 | 211 | 523 | 4 |
| BAD Windows .NET Random(seed=1) compat | 714 | 704 | 4 | 6 |
| ANSI C sample LCG (1103515245,12345; seed=1) | 714 | 116 | 592 | 6 |
| LCG MINSTD (seed=1) | 714 | 112 | 596 | 6 |
| AES-128-CTR (NIST key) | 714 | 696 | 12 | 6 |
| cryptography::CtrDrbgAes256 (seed=00..2f) | 714 | 701 | 7 | 6 |
| Constant (0xDEAD_DEAD) | 714 | 0 | 708 | 6 |
| Counter (0,1,2,…) | 714 | 1 | 707 | 6 |
| Dual_EC_DRBG P-256 (NIST Q, seed=0x00..01) | 199 | 195 | 0 | 4 |

## Readout

- `OsRng` came in at `724/738` on this rerun. That is rougher than the prior
  pass, but still looks like battery-tail noise rather than a structural break:
  two universal-family lows, three template lows, and five `rgb_bitdist` lows.
- `MT19937` comes in at `728/738`, which is exactly the “a handful of low
  p-values in a huge battery” zone rather than a red flag.
- `MT19937` is still basically where it should be: `6/738` failures, which is
  below the rough false-positive budget for a battery this large.
- `Xorshift64` no longer gets a fake-clean report. It now takes `4` real
  `rgb_bitdist` failures.
- `BSD random()` and glibc `random()` no longer look spotless either. Each now
  picks up `2` `rgb_bitdist` failures.
- `AES-128-CTR` lands at `12/714` failures and `cryptography::CtrDrbgAes256`
  lands at `7/714`. Most of those are `rgb_bitdist` family lows; this is the
  right place to be cautious rather than melodramatic.
- `VB6 Rnd()`, ANSI C `rand()`, `MINSTD`, `Constant`, and `Counter` are
  destroyed, which is exactly the sanity check the suite needed to keep.
- The newly faithful `dab_monobit2` is now part of these results. It does not
  by itself annihilate every weak generator, which is consistent with the
  reference code; the culling still comes from the whole battery, not one test.

## Failure Highlights

### OsRng (/dev/urandom)

- `10` failures total:
  - `nist::universal`: `p = 0.006308`
  - `maurer::universal_l07`: `p = 0.006308`
  - `nist::non_overlapping_template` at `B=000111111`: `p = 0.004105`
  - `nist::non_overlapping_template` at `B=001110111`: `p = 0.000065`
  - `nist::non_overlapping_template` at `B=110110100`: `p = 0.002936`
  - five `dieharder::bit_distribution` lows:
    - width `6`, pattern `12`: `p = 0.000041`
    - width `6`, pattern `60`: `p = 0.002539`
    - width `7`, pattern `104`: `p = 0.007486`
    - width `8`, pattern `33`: `p = 0.002378`
    - width `8`, pattern `135`: `p = 0.008520`

### MT19937 (seed=19650218)

- `nist::non_overlapping_template` at `B=100110000`: `p = 0.007998`
- `nist::non_overlapping_template` at `B=101101000`: `p = 0.006862`
- four `dieharder::bit_distribution` lows across widths `5` and `8`

### Xorshift64 (seed=1)

- no NIST or classic DIEHARD failures in this run
- `4` `dieharder::bit_distribution` failures:
  - width `5`, pattern `3`: `p = 0.007169`
  - width `5`, pattern `12`: `p = 0.005938`
  - width `6`, pattern `0`: `p = 0.001047`
  - width `8`, pattern `95`: `p = 0.006322`

### Xorshift32 (seed=1)

- `nist::matrix_rank`: `p = 0.000000`
- `diehard::binary_rank_32x32`: `p = 0.000000`
- `diehard::binary_rank_31x31`: `p = 0.000000`
- `diehard::count_ones_stream`: `p = 0.000020`
- plus `9` more failures, mostly `rgb_bitdist`

### BAD Unix / Windows Historical Generators

- `System V rand()`: `13` failures, including `runs_up`, `lagged_sums`, and
  several `rgb_bitdist` rows
- `System V mrand48()`: `9` failures, including a very low
  `non_overlapping_template` and `dieharder::dct`
- `FreeBSD12 rand_r() compat`: `15` failures
- `Windows CRT rand()`: `3` failures
- `.NET Random(seed)` compat: `4` failures
- `VB6/VBA Rnd()`: `523` failures; it is catastrophically bad here

### AES-128-CTR (NIST key)

- the same mirrored `nist::non_overlapping_template` failures remain:
  - `B=000000001`: `p = 0.000483`
  - `B=100000000`: `p = 0.000483`
- the other `10` failures are all `dieharder::bit_distribution`

### cryptography::CtrDrbgAes256 (seed=00..2f)

- `7` failures total, all in `dieharder::bit_distribution`
- this is roughly the count you would expect from chance alone in a
  `714`-result battery

### Constant / Counter

- `Constant`: `708/714` failures
- `Counter`: `707/714` failures

## Bottom Line

This run is much more believable than the old one.

- weak generators are now getting caught by the enlarged, less-fake battery
- `Xorshift64`, `BSD random()`, and glibc `random()` are no longer coming back
  artificially spotless
- the obviously awful generators are annihilated
- `AES-CTR`, `MT19937`, and `cryptography::CtrDrbgAes256` are in the
  “watch the clustering, but don’t panic” zone rather than the old
  overconfident greenwash
