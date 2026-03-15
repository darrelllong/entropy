# Peer Review — Fifth Pass
*Generated 2026-03-14. Every .rs file read line-by-line against primary references.*
*Updated 2026-03-15: BUG 1 and BUG 2 applied.*

---

## Bugs Confirmed — Must Fix

### BUG 1 — `nist::runs` — wrong erfc convention

**File:** `src/nist/runs.rs:42`

```rust
// WRONG (current)
let denom = 2.0 * (2.0 * n as f64).sqrt() * pi * (1.0 - pi);
let p_value = erfc(numer / (denom * SQRT_2));

// CORRECT — match NIST STS sts-2.1.2/src/runs.c
// erfc_arg = |V_n - 2nπ(1-π)| / (2π(1-π)√(2n))
// p_value = erfc(erfc_arg)   ← no SQRT_2
let p_value = erfc(numer / denom);
```

**Impact:** The argument to erfc is √2 times too small. A generator that NIST STS
rejects at p=0.001 shows p≈0.024 in the current code — above the α=0.01 threshold.
The test passes RNGs it should fail.

**Reference:** NIST STS `sts-2.1.2/src/runs.c`:
```c
erfc_arg = fabs(V_n_obs - 2.0*n*pi*(1-pi))/(2.0*pi*(1-pi)*sqrt(2.0*n));
p_value  = erfc(erfc_arg);
```

NIST uses `erfc(|obs−μ|/σ)` directly — NOT `erfc(|obs−μ|/(σ√2))`. This
differs from the monobit test (which correctly uses erfc(s_obs/√2)) because
NIST's runs σ is already defined as σ = 2π(1-π)√(2n) and the reference calls
`erfc(arg)` with that exact denominator, not σ·√2.

---

### BUG 2 — `nist::random_excursions_variant` — wrong erfc convention

**File:** `src/nist/random_excursions_variant.rs:54-55`

```rust
// WRONG (current)
let denom = (2.0 * j as f64 * (4.0 * x.unsigned_abs() as f64 - 2.0)).sqrt();
let p_value = erfc(numer / (denom * SQRT_2));

// CORRECT — match NIST STS sts-2.1.2/src/randomexcursionsvariant.c
let p_value = erfc(numer / denom);
```

**Impact:** Same √2 factor error as BUG 1. All 18 sub-test p-values are inflated,
making every state test too lenient.

**Reference:** NIST STS `sts-2.1.2/src/randomexcursionsvariant.c`:
```c
p_value = erfc(fabs((double)count-(double)J)/
               sqrt(2.0*(double)J*(4.0*abs(x)-2)));
```

---

## Recently Fixed — Confirmed Correct

| Fix | Status |
|-----|--------|
| `nist::linear_complexity` μ formula: `(9+r)/36` → `(9+(−1)^M)/36` | FIXED ✓ |
| `dieharder::lagged_sums` one-sided p → two-sided `erfc(z/√2)` | FIXED ✓ |
| `diehard::dna` boffset step 1 → step 2, groups div_ceil(16) | FIXED ✓ |
| `nist::runs` erfc argument: `erfc(numer/(denom*√2))` → `erfc(numer/denom)` | FIXED ✓ |
| `nist::random_excursions_variant` erfc argument: `erfc(numer/(denom*√2))` → `erfc(numer/denom)` | FIXED ✓ |
| `dieharder::monobit2` simplified fixed-block χ² → faithful multi-scale `dab_monobit2` extreme-p path | FIXED ✓ |
| `diehard::runs_float` final closeout now matches `diehard_runs.c` binning behavior | FIXED ✓ |

---

## Full Function Status Table

### `src/nist/` — NIST SP 800-22

| Function | File | Verdict | Notes |
|----------|------|---------|-------|
| `frequency` | frequency.rs | **CORRECT** | erfc(s_obs/√2) matches NIST STS ✓ |
| `block_frequency` | block_frequency.rs | **CORRECT** | χ²=4M·Σ(π_j−0.5)², igamc(N/2,χ²/2) ✓ |
| `runs` | runs.rs | **CORRECT** (just fixed) | Extra √2 in erfc denominator removed; `erfc(numer/denom)` ✓ |
| `longest_run` | longest_run.rs | **CORRECT** | Table 1 parameters, pi values, bins ✓ |
| `matrix_rank` | matrix_rank.rs | **CORRECT** | P32=0.2888, P31=0.5776, P≤30=0.1336 ✓ |
| `spectral` | spectral.rs | **CORRECT** | T=√(n·ln(20)), d=(N1−N0)/√var, erfc(|d|/√2) ✓ |
| `non_overlapping_template` | non_overlapping_template.rs | **CORRECT** | N=8 blocks, σ² formula ✓, 148 templates ✓ |
| `overlapping_template` | overlapping_template.rs | **CORRECT** | m=9 restriction ✓, pi table ✓, M=1032 ✓ |
| `universal` | universal.rs | **CORRECT** | Maurer table ✓, correction factor c ✓, erfc(z/√2) ✓ |
| `linear_complexity` | linear_complexity.rs | **CORRECT** (just fixed) | μ uses (9+(−1)^M)/36 ✓ |
| `serial` | serial.rs | **CORRECT** | ψ² formula ✓, del1/del2 ✓, dof ✓ |
| `approximate_entropy` | approximate_entropy.rs | **CORRECT** | φ(m)−φ(m+1) ✓, χ²=2n(ln2−ApEn) ✓, dof=2^{m-1} ✓ |
| `cumulative_sums` | cumulative_sums.rs | **CORRECT** | SP 800-22 §2.13.4 series formula ✓ |
| `random_excursions` | random_excursions.rs | **CORRECT** | π_k formula ✓, df=5 → igamc(2.5,…) ✓ |
| `random_excursions_variant` | random_excursions_variant.rs | **CORRECT** (just fixed) | Extra √2 in erfc denominator removed; `erfc(numer/denom)` ✓ |

### `src/diehard/` — Marsaglia DIEHARD

| Function | File | Verdict | Notes |
|----------|------|---------|-------|
| `birthday_spacings` | birthday_spacings.rs | **CORRECT** | λ=2 ✓, interval counting ✓, 9 offsets → KS ✓ |
| `binary_rank_32x32` | binary_rank.rs | **CORRECT** | Reference probs ✓, GF(2) Gaussian elimination ✓ |
| `binary_rank_31x31` | binary_rank.rs | **CORRECT** | Exact gf2_rank_probability() ✓ |
| `binary_rank_6x8` | binary_rank.rs | **CORRECT** | Exact probabilities computed ✓ |
| `bitstream` | bitstream.rs | **CORRECT** | 20-bit words, 2^21 windows, mean=141909, σ=428 ✓ |
| `count_ones_stream` | count_ones.rs | **CORRECT** | Q5−Q4 diff statistic ✓, mean=2500, σ=√5000 ✓ |
| `craps` | craps.rs | **CLOSE** | Stats correct; single-result `craps()` takes min(p_wins, p_throws) which inflates false-positive rate — use `craps_both()` |
| `minimum_distance` (2D) | minimum_distance.rs | **WARNED** | Known-buggy formula explicitly flagged in code ✓ |
| `monkey::opso` | monkey.rs | **CORRECT** | Paired 10-bit fields ✓, constants ✓ |
| `monkey::oqso` | monkey.rs | **CORRECT** | 4-word groups, 5-bit fields ✓ |
| `monkey::dna` | monkey.rs | **CORRECT** (just fixed) | boffset step=2, groups=div_ceil(16) ✓ |
| `parking_lot` | parking_lot.rs | **CORRECT** | L∞ cars ✓, mean=3523, σ=21.9 ✓, z→CDF→KS ✓ |
| `runs_float` | runs_float.rs | **CORRECT** | Grafton 1981 A/B matrices ✓; final closeout matches `diehard_runs.c` (`next > first`) ✓ |
| `spheres_3d` | spheres_3d.rs | **CORRECT** | r³~Exp(30) ✓, 1−exp(−r³/30) ✓, KS ✓ |
| `squeeze` | squeeze.rs | **CORRECT** | SDATA table from reference ✓, j-clamping logic ✓ |

### `src/dieharder/` — Brown Dieharder

| Function | File | Verdict | Notes |
|----------|------|---------|-------|
| `bit_distribution` | bit_distribution.rs | **CORRECT** | MSB-first ✓, binomial PMF ✓, Vtest bundling ✓ |
| `byte_distribution` | byte_distribution.rs | **CORRECT** | Exact `dab_bytedistrib.c` shifting/layout ✓, including the 9-stream flat table indexing ✓ |
| `dct` | dct.rs | **CORRECT** | DCT-II formula ✓, DC adjustment ✓, rotation schedule ✓ |
| `fill_tree` | fill_tree.rs | **CORRECT** | TARGET_DATA table ✓, `position_counts[fail_pos/2]` matches `dab_filltree.c` ✓ |
| `gcd` | gcd.rs | **CORRECT** | P(gcd=k)=6/(π²k²) ✓, KPROB table ✓, Euclidean step count ✓ |
| `ks_uniform` | ks_uniform.rs | **CORRECT** | Single KS on all samples ✓ |
| `lagged_sums` | lagged_sums.rs | **CORRECT** (just fixed) | Two-sided erfc(z/√2) ✓ |
| `minimum_distance_nd` | minimum_distance_nd.rs | **CORRECT** | Fischler formula ✓, Q_CORRECTION table ✓, ball_volume ✓ |
| `monobit2` | monobit2.rs | **CORRECT** | Faithful `dab_monobit2.c` port: auto-`ntuple`, per-scale `chisq_binomial`, `evalMostExtreme` correction ✓ |
| `permutations` | permutations.rs | **CORRECT** | Lehmer code ✓, factorial ✓, chi-square ✓ |

### `src/research/` — Research Tests

| Function | File | Verdict | Notes |
|----------|------|---------|-------|
| `gorilla` | marsaglia_tsang.rs | **CORRECT** | 26-bit words ✓, mean=24687971, σ=4170 ✓, upper-tail p intentional per paper ✓ |
| `permutation_test` | knuth.rs | **CORRECT** | Factorial number system rank ✓, chi-square ✓ |
| `gap_test` | knuth.rs | **CORRECT** | Geometric probabilities ✓, df=max_gap ✓ |
| `runs_above_below_median_test` | knuth.rs | **CORRECT** | Wald-Wolfowitz moments ✓ |
| `pincus::approximate_entropy_profile` | pincus.rs | **CORRECT** | Same φ computation as nist::approximate_entropy ✓ |
| `fpf_test` | practrand_fpf.rs | **APPROX** | G-test ✓; uses asymptotic chi-square, not PractRand's empirical tables — documented |
| `hamming_corr` | testu01_hamming.rs | **CORRECT** | ρ̂=4·Σ/((n−1)L) ✓, z=ρ̂·√(n−1) ✓, erfc(z/√2) ✓ |
| `hamming_indep` | testu01_hamming.rs | **CORRECT** | Main chi-square ✓, lumping rule ✓, block dof special case matches `sstring.c` ✓ |
| `lempel_ziv` | testu01_lz.rs | **APPROX** | LZ78 trie ✓; μ/σ tables empirical — documented |
| `webster_tavares` | webster_tavares.rs | **CORRECT** | Avalanche/SAC/BIC ✓ |

### `src/rng/` — Reference RNGs

| Module | Verdict | Notes |
|--------|---------|-------|
| `c_stdlib.rs` — SystemVRand | **CORRECT** | `(state>>16)&0x7FFF`, 15-bit, known output ✓ |
| `c_stdlib.rs` — WindowsMsvcRand | **CORRECT** | `(state>>16)&0x7FFF`, 15-bit, known output ✓ |
| `c_stdlib.rs` — WindowsVb6Rnd | **CORRECT** | 24-bit state, mask 0x00FF_FFFF ✓ |
| `c_stdlib.rs` — WindowsDotNetRandom | **CORRECT** | Subtractive generator, known output ✓ |
| `c_stdlib.rs` — BsdRandom | **CORRECT** | TYPE_3 deg=31 sep=3, 310 warm-up ✓ |
| `c_stdlib.rs` — BsdRandCompat | **CORRECT** | Park-Miller, known output ✓ |
| `c_stdlib.rs` — Rand48 | **CORRECT** | a=0x5DEECE66D, c=0xB, m=2^48 ✓ |
| `lcg.rs` — Msvc | **CORRECT** | shift=16, output_mask=0x7FFF ✓ |
| `lcg.rs` — AnsiC, Minstd, Borland | **CORRECT** | Parameters verified ✓ |
| `aes_ctr.rs` | **CORRECT** | NIST SP 800-38A test vector ✓ |
| `blum_blum_shub.rs` | **CORRECT** | ✓ |
| `blum_micali.rs` | **CORRECT** | ✓ |
| `bad.rs`, `os.rs`, `crypto_cprng.rs` | **CORRECT** | Test fixtures ✓ |

### `src/math.rs` — Core Math

| Function | Verdict | Notes |
|----------|---------|-------|
| `erfc` | **CORRECT** | Numerical Recipes rational approx, max error 1.2×10⁻⁷ ✓ |
| `normal_cdf` | **CORRECT** | Φ(x) = 0.5·erfc(−x/√2) ✓ |
| `lgamma` | **CORRECT** | Lanczos approximation ✓ |
| `igamc` | **CORRECT** | Upper incomplete gamma Q(a,x); Lentz CF ✓ |
| `ks_test` | **CORRECT** | Exact matrix method (n≤4999) + Stephens asymptotic ✓ |
| `chi2_pvalue` | **CORRECT** | igamc(df/2, χ²/2) ✓ |
| `fft_magnitudes` | **CORRECT** | rustfft ✓ |

---

## Priority Fix List

### Priority 1 — Wrong p-values vs reference

~~`src/nist/runs.rs` line 42~~ — **APPLIED**

~~`src/nist/random_excursions_variant.rs` line 55~~ — **APPLIED**

### Priority 2 — Remaining reference-source verification

- No live mismatches remain in the previously flagged `byte_distribution`,
  `fill_tree`, `monobit2`, `runs_float`, or `hamming_indep` code paths; each was
  checked against the upstream C and either verified or fixed.

### Priority 3 — Acknowledged approximations (documented, no fix needed)

- `research::practrand_fpf`: asymptotic chi-square vs PractRand empirical tables.
- `research::testu01_lz`: empirical μ/σ tables.

---

## Confirmed Clean (prior suspects cleared)

| Item | Cleared because |
|------|-----------------|
| `nist::linear_complexity` μ | Fixed: `(9+(−1)^M)/36` |
| `diehard::dna` boffset | Fixed: step=2, 16 samples/group |
| `dieharder::lagged_sums` | Fixed: two-sided erfc |
| `nist::runs` erfc | Fixed: removed spurious `* SQRT_2`; matches NIST STS `runs.c` |
| `nist::random_excursions_variant` erfc | Fixed: removed spurious `* SQRT_2`; all 18 sub-tests corrected |
| `rng::lcg::Msvc` output | output_mask=0x7FFF (15-bit) ✓ |
| `diehard::binary_rank_31x31` | Uses exact gf2_rank_probability() |
| `research::gorilla` one-sided | Intentional: paper detects excess missing words |
| `nist::overlapping_template` m=9 | pi table only valid for m=9, M=1032 |
| `nist::frequency` erfc | erfc(s_obs/√2) matches NIST STS |
| `nist::spectral` erfc | erfc(d/√2) matches NIST STS |
| `nist::universal` erfc | erfc(z/√2) matches NIST STS |
| All RNG implementations | Verified against known output vectors |
