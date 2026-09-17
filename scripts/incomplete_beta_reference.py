#!/usr/bin/env python3
"""High-precision references for `regularized_incomplete_beta` and
`student_t_quantile`, written to tests/data/.

incomplete_beta.txt: `x a b I` per line, I_x(a, b) from mpmath's betainc at 50
digits, printed to 30. The shapes are every pair from
{1e-300, 1e-100, 1e-20, 1e-3, 0.1, 0.5, 1, 2.5, 10, 100, 1e3, 1e4}; for each pair x is taken at the
mean, at 1, 3, 10 and 40 standard deviations either side, just either side of
the continued fraction's switch (a+1)/(a+b+2), at 1/2, and at four uniform
draws (seed 20260917). Points outside (0, 1) are dropped, and a case mpmath has not finished within
20 seconds is left out and named on stderr. Shapes beyond 1e4
are out of mpmath's practical reach (some take hours); the Rust tests check them by exact
identities and symmetry instead.

student_t.txt: `freedom probability t` per line, the one-sided Student t
quantile from mpmath at 50 digits by solving I_{nu/(nu+t^2)}(nu/2, 1/2) = 2(1-p)
for t by bisection. The grid is the regime of factoring's polynomial race: freedom
(arms - 1)(blocks - 1) for 2 to 32 arms and 2 to 64 blocks, and probability
1 - 0.05/(2 (field - 1) 16) for fields of 2 to 32, plus the two-sided
5%, 1% and 0.1% points at 1 to 30 and 60, 120 degrees of freedom.

Usage (in a virtual environment with mpmath):
  incomplete_beta_reference.py [WORKERS] [--beta-only]
"""
import multiprocessing
import random
import sys
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "tests" / "data"
LIMIT = 20


def beta_cases():
    from mpmath import mp, mpf, sqrt

    mp.dps = 30
    rng = random.Random(20260917)
    shapes = [1e-300, 1e-100, 1e-20, 1e-3, 0.1, 0.5, 1.0, 2.5, 10.0, 100.0, 1e3, 1e4]
    cases = []
    for a in shapes:
        for b in shapes:
            mean = a / (a + b)
            sd = float(sqrt(mpf(a) * b / ((mpf(a) + b) ** 2 * (mpf(a) + b + 1))))
            xs = {mean + k * sd for k in (-40, -10, -3, -1, 0, 1, 3, 10, 40)}
            switch = (a + 1) / (a + b + 2)
            xs |= {switch * (1 - 1e-9), switch * (1 + 1e-9), 0.5}
            xs |= {rng.random() for _ in range(4)}
            cases += [(x, a, b) for x in sorted(xs) if 0 < x < 1]
    return cases


def beta_value(case):
    from mpmath import betainc, mp, mpf

    mp.dps = 50
    x, a, b = case
    value = betainc(mpf(a), mpf(b), 0, mpf(x), regularized=True)
    return f"{x!r} {a!r} {b!r} {mp.nstr(value, 30)}"


def t_cases():
    cases = set()
    for arms in range(2, 33):
        for blocks in range(2, 65):
            freedom = (arms - 1) * (blocks - 1)
            if freedom < 2:
                continue
            for field in (2, 4, 8, 16, 32):
                cases.add((freedom, 1 - 0.05 / (2 * (field - 1) * 16)))
    for freedom in list(range(1, 31)) + [60, 120]:
        for two_sided in (0.05, 0.01, 0.001):
            cases.add((freedom, 1 - two_sided / 2))
    return sorted(cases)


def t_value(case):
    from mpmath import betainc, mp, mpf

    mp.dps = 50
    freedom, probability = case
    nu = mpf(freedom)
    tail = 2 * (1 - mpf(probability))
    f = lambda t: betainc(nu / 2, mpf(1) / 2, 0, nu / (nu + t * t), regularized=True) - tail
    # The two-sided tail falls as t grows: bisection to 150 bits.
    lo, hi = mpf(0), mpf(1)
    while f(hi) > 0:
        hi *= 2
    for _ in range(150):
        mid = (lo + hi) / 2
        if f(mid) > 0:
            lo = mid
        else:
            hi = mid
    t = (lo + hi) / 2
    return f"{freedom} {probability!r} {mp.nstr(t, 30)}"


def main():
    numbers = [a for a in sys.argv[1:] if a.isdigit()]
    workers = int(numbers[0]) if numbers else multiprocessing.cpu_count()
    OUT.mkdir(parents=True, exist_ok=True)
    with multiprocessing.Pool(workers) as pool:
        if "--beta-only" not in sys.argv:
            lines = pool.map(t_value, t_cases(), chunksize=4)
            (OUT / "student_t.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")
        # mpmath takes minutes to hours on a few extreme tails; a case not done
        # within LIMIT seconds is left out and named on stderr.
        pending = [(case, pool.apply_async(beta_value, (case,))) for case in beta_cases()]
        lines, skipped = [], []
        for case, result in pending:
            try:
                lines.append(result.get(timeout=LIMIT))
            except multiprocessing.TimeoutError:
                skipped.append(case)
        (OUT / "incomplete_beta.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")
        for case in skipped:
            print(f"skipped after {LIMIT} s: x={case[0]!r} a={case[1]!r} b={case[2]!r}", file=sys.stderr)
        pool.terminate()
    return 0


if __name__ == "__main__":
    sys.exit(main())
