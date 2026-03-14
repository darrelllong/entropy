# Full Battery Results

Full `run_tests` battery harvested from `darby.local` on 2026-03-14 from [darby-full-battery.log](/Users/darrell/entropy/darby-full-battery.log).

Command:

```sh
cargo run --release --bin run_tests
```

Notes:

- Most RNGs ran the full active battery and therefore produced `225` results.
- Some RNGs produced `201` results because `nist::random_excursions` and `nist::random_excursions_variant` skipped when the walk-cycle precondition failed.
- `Dual_EC_DRBG P-256` is intentionally NIST-only in this runner, so it produced `188` results.
- These are single-sample battery results, not repeated meta-analysis. Low single p-values can happen by chance, but large clusters of failures are still informative.

## Summary Table

| RNG | Total | PASS | FAIL | SKIP |
|---|---:|---:|---:|---:|
| OsRng (/dev/urandom) | 225 | 225 | 0 | 0 |
| MT19937 (seed=19650218) | 225 | 223 | 2 | 0 |
| Xorshift64 (seed=1) | 201 | 199 | 0 | 2 |
| Xorshift32 (seed=1) | 225 | 221 | 4 | 0 |
| BAD Unix System V rand() (15-bit LCG, seed=1) | 225 | 218 | 7 | 0 |
| BAD Unix System V mrand48() (seed=1) | 225 | 221 | 4 | 0 |
| BAD Unix BSD random() TYPE_3 (seed=1) | 225 | 225 | 0 | 0 |
| BAD Unix Linux glibc rand()/random() (seed=1) | 225 | 225 | 0 | 0 |
| BAD Unix FreeBSD12 rand_r() compat (seed=1) | 225 | 222 | 3 | 0 |
| BAD Windows CRT rand() (MSVC/UCRT lineage, seed=1) | 201 | 198 | 1 | 2 |
| BAD Windows VB6/VBA Rnd() (project seed=1) | 225 | 198 | 27 | 0 |
| BAD Windows .NET Random(seed=1) compat | 201 | 199 | 0 | 2 |
| ANSI C sample LCG (1103515245,12345; seed=1) | 201 | 113 | 86 | 2 |
| LCG MINSTD (seed=1) | 201 | 108 | 91 | 2 |
| BBS (p=2³¹−1, q=4294967291) | 225 | 224 | 1 | 0 |
| Blum-Micali (p=2³¹−1, g=7) | 225 | 221 | 4 | 0 |
| AES-128-CTR (NIST key) | 201 | 197 | 2 | 2 |
| cryptography::CtrDrbgAes256 (seed=00..2f) | 201 | 199 | 0 | 2 |
| Constant (0xDEAD_DEAD) | 201 | 0 | 199 | 2 |
| Counter (0,1,2,…) | 201 | 1 | 198 | 2 |
| Dual_EC_DRBG P-256 (NIST Q, seed=0x00..01) | 188 | 188 | 0 | 0 |

## Notable Results

- `OsRng`: clean sweep, `225/225`.
- `MT19937`: only two low `nist::non_overlapping_template` p-values.
- `Xorshift32`: caught hard by rank tests and `diehard::count_ones_stream`.
- `System V rand()` and `mrand48()`: visibly weak, but not annihilated the way the tiny LCGs are.
- `BSD random()` and `glibc random()`: this single run did not catch them. That does not make them good RNGs; it means this one sample was not enough to embarrass them statistically.
- `Windows VB6/VBA Rnd()`: badly exposed, with `27` failures.
- `ANSI C sample LCG` and `MINSTD`: destroyed by the battery, as expected.
- `AES-128-CTR`: only the mirrored `nist::non_overlapping_template` probes failed.
- `cryptography::CtrDrbgAes256`: no failures; only the excursion-family precondition skips.
- `Constant` and `Counter`: obliterated, which is a useful sanity check on the harness.

## Failure Highlights

### MT19937 (seed=19650218)

- `nist::non_overlapping_template` at `B=100110000`: `p = 0.007998`
- `nist::non_overlapping_template` at `B=101101000`: `p = 0.006862`

### Xorshift32 (seed=1)

- `nist::matrix_rank`: `p = 0.000000`
- `diehard::binary_rank_32x32`: `p = 0.000000`
- `diehard::binary_rank_31x31`: `p = 0.000000`
- `diehard::count_ones_stream`: `p = 0.000020`

### BAD Unix System V rand() (15-bit LCG, seed=1)

- `diehard::runs_up`: `p = 0.000217`
- `dieharder::lagged_sums`: `p = 0.000116`
- plus five more low-p failures, mostly template-style probes

### BAD Unix System V mrand48() (seed=1)

- `nist::non_overlapping_template` at `B=110000010`: `p = 0.000026`
- `dieharder::dct`: `p = 0.003623`
- plus two more low-p `nist::non_overlapping_template` results

### BAD Windows CRT rand() (MSVC/UCRT lineage, seed=1)

- `dieharder::ks_uniform`: `p = 0.000018`
- skipped both excursion families (`J = 358 < 500`)

### BAD Windows VB6/VBA Rnd() (project seed=1)

- `nist::spectral`: `p = 0.000000`
- `diehard::binary_rank_6x8`: `p = 0.000000`
- `diehard::bitstream`: `p = 0.000000`
- `diehard::birthday_spacings`: `p = 0.007099`
- plus `23` more failures

### ANSI C sample LCG (1103515245,12345; seed=1)

- `86` failures total
- immediately fails `nist::frequency`, `block_frequency`, `runs`, `longest_run`, and `matrix_rank`
- skipped both excursion families (`J = 36 < 500`)

### LCG MINSTD (seed=1)

- `91` failures total
- immediately fails `nist::frequency`, `block_frequency`, `runs`, `matrix_rank`, and `spectral`
- skipped both excursion families (`J = 2 < 500`)

### BBS (p=2³¹−1, q=4294967291)

- one failure: `nist::random_excursions` at `x = -1`: `p = 0.007816`

### Blum-Micali (p=2³¹−1, g=7)

- three low `nist::non_overlapping_template` results
- `dieharder::bit_distribution`: `p = 0.009815`

### AES-128-CTR (NIST key)

- `nist::non_overlapping_template` at `B=000000001`: `p = 0.000483`
- `nist::non_overlapping_template` at `B=100000000`: `p = 0.000483`
- skipped both excursion families (`J = 434 < 500`)

### cryptography::CtrDrbgAes256 (seed=00..2f)

- no failures
- skipped both excursion families (`J = 24 < 500`)

### Constant (0xDEAD_DEAD)

- `199` failures out of `201`
- only the excursion-family precondition checks skipped

### Counter (0,1,2,…)

- `198` failures out of `201`
- only one test survived; both excursion families skipped

## Bottom Line

The full Darby run behaves directionally the way we want:

- obviously bad generators get destroyed
- tiny historical LCGs get culled hard
- VB6 `Rnd()` looks awful
- AES-CTR and `cryptography::CtrDrbgAes256` look healthy in a single full-battery pass
- `BSD` and `glibc random()` still deserve the “bad historical Unix RNG” label even though this one run did not catch them
