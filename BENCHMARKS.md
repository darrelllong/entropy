# Benchmarks

Pilot throughput run for the current `entropy` worktree on `darby.local` (`Linux aarch64`), using `pilot-bench` `run_program --preset normal`.

This pass intentionally excludes `OsRng`, `BBS`, and `Blum-Micali`, per the pilot brief. It does include the historical Unix and Windows generators, including:

- System V `rand()` and `mrand48()`
- BSD `random()`
- Linux glibc `rand()/random()`
- FreeBSD `rand_r()` compatibility
- Windows CRT `rand()`
- VB6/VBA `Rnd()`
- classic `.NET Random(seed)` compatibility

Source log: [darby-pilot.log](/Users/darrell/entropy/darby-pilot.log)

## Results

Throughput is reported in millions of 32-bit words per second (`MW/s`). `Runs` is the number of Pilot samples used to hit the normal-preset confidence target.

| Generator | MW/s | 95% CI | Runs |
|---|---:|---:|---:|
| `MT19937 (seed=19650218)` | 210.2 | ±3.792 | 50 |
| `Xorshift64 (seed=1)` | 673.3 | ±1.179 | 50 |
| `Xorshift32 (seed=1)` | 719.6 | ±2.382 | 50 |
| `BAD Unix System V rand() (seed=1)` | 307.9 | ±0.3244 | 80 |
| `BAD Unix System V mrand48() (seed=1)` | 573.7 | ±0.6346 | 50 |
| `BAD Unix BSD random() TYPE_3 (seed=1)` | 183.5 | ±0.3348 | 80 |
| `BAD Unix Linux glibc rand()/random() (seed=1)` | 182.8 | ±0.0463 | 110 |
| `BAD Unix FreeBSD12 rand_r() compat (seed=1)` | 123.3 | ±0.07599 | 80 |
| `BAD Windows CRT rand() (seed=1)` | 307.6 | ±0.482 | 50 |
| `BAD Windows VB6/VBA Rnd() (seed=1)` | 269.6 | ±1.768 | 110 |
| `BAD Windows .NET Random(seed=1) compat` | 149.5 | ±0.2996 | 110 |
| `ANSI C sample LCG (seed=1)` | 137.0 | ±0.7252 | 50 |
| `LCG MINSTD (seed=1)` | 116.2 | ±2.029 | 260 |
| `AES-128-CTR (NIST key)` | 53.33 | ±0.4426 | 50 |
| `cryptography::CtrDrbgAes256 (seed=00..2f)` | 0.7594 | ±0.003181 | 110 |
| `Constant (0xDEAD_DEAD)` | 6356 | ±76.85 | 140 |
| `Counter (0,1,2,...)` | 6376 | ±61.02 | 50 |

The synthetic ceiling generators dominate raw throughput, so the visual uses normalized `log10(MW/s)` rather than a linear scale.

![Radar chart of pilot throughput on a normalized log scale](/Users/darrell/entropy/assets/benchmarks-radar.svg)
