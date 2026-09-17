#!/usr/bin/env python3
"""Derive and check the Lanczos coefficients behind `math::ln_gamma`.

Construction. Lanczos's approximation writes, for Re z > -1/2,

    Gamma(z + 1) = sqrt(2 pi) t^(z + 1/2) e^(-t) A_g(z),   t = z + g + 1/2,

and approximates the analytic factor A_g by the partial-fraction sum

    A_g(z) ~ c_0 + sum_{k=1}^{n-1} c_k / (z + k)

(Lanczos, J. SIAM Numer. Anal. B 1 (1964), 86-96). With g = 7 and n = 9 the
coefficients are the unique solution of the n linear equations that make the
sum equal A_g exactly at z = 0, 1, ..., n - 1, where A_g(z) is known from
Gamma(z + 1) = z!. The shipped table is that solution, each value rounded to
the nearest IEEE double; `--check` confirms all nine agree bit for bit.

Error separation, for ln Gamma(x) = ln Gamma(z + 1) with z = x - 1:
  approximation error   the exact coefficients against mpmath's loggamma;
  coefficient rounding  the double coefficients, still evaluated exactly;
  floating evaluation   measured by the Rust tests against `--reference` and
                        `--sweep`, absolute on [0.1, 3] and relative elsewhere.
`--check` covers the approximation's own range, x >= 1/2: errors absolute
for 1/2 <= x <= 3, which holds both zeros of ln Gamma (x = 1 and x = 2, where
a relative error is undefined), and relative above.

Usage (in a virtual environment with mpmath):
  lanczos_coefficients.py --check       derive, compare with src, report errors
  lanczos_coefficients.py --reference   print Rust reference pairs (x, ln Gamma rounded to nearest)
  lanczos_coefficients.py --sweep FILE  write the dense sweep for the ignored test
"""
import re
import sys
from pathlib import Path

from mpmath import exp, loggamma, lu_solve, matrix, mp, mpf, pi, sqrt, gamma

G = 7
N = 9
SOURCE = Path(__file__).resolve().parent.parent / "src" / "math" / "ln_gamma.rs"


def a_g(z):
    t = z + G + mpf(1) / 2
    return gamma(z + 1) / (sqrt(2 * pi) * t ** (z + mpf(1) / 2) * exp(-t))


def derive():
    system, rhs = matrix(N, N), matrix(N, 1)
    for row in range(N):
        z = mpf(row)
        system[row, 0] = 1
        for k in range(1, N):
            system[row, k] = 1 / (z + k)
        rhs[row] = a_g(z)
    solution = lu_solve(system, rhs)
    return [solution[i] for i in range(N)]


def shipped():
    text = SOURCE.read_text(encoding="utf-8")
    body = re.search(r"const COEFFICIENTS: \[f64; 9\] = \[(.*?)\];", text, re.S).group(1)
    return [float(v.replace("_", "")) for v in re.findall(r"-?[0-9][0-9_.]*(?:e-?[0-9]+)?", body)]


def finite_limit():
    """The largest double whose ln Gamma rounds to a finite double, by
    bisection over doubles at 80 digits: ln Gamma(x) < MAX + ulp(MAX)/2."""
    import math

    mp.dps = 80
    limit = mpf(sys.float_info.max) + mpf(2) ** (1023 - 53)
    lo, hi = 2.0, sys.float_info.max
    while True:
        mid = lo + (hi - lo) / 2
        if mid in (lo, hi):
            break
        if loggamma(mpf(mid)) < limit:
            lo = mid
        else:
            hi = mid
    while loggamma(mpf(math.nextafter(lo, math.inf))) < limit:
        lo = math.nextafter(lo, math.inf)
    while loggamma(mpf(lo)) >= limit:
        lo = math.nextafter(lo, -math.inf)
    return lo


def shipped_limit():
    text = SOURCE.read_text(encoding="utf-8")
    value = re.search(r"const LN_GAMMA_FINITE_BELOW: f64 = ([0-9_.e]+);", text).group(1)
    return float(value.replace("_", ""))


def ln_gamma_lanczos(x, coefficients):
    z = mpf(x) - 1
    t = z + G + mpf(1) / 2
    series = coefficients[0] + sum(c / (z + k) for k, c in enumerate(coefficients) if k)
    return mp.log(2 * pi) / 2 + (z + mpf(1) / 2) * mp.log(t) - t + mp.log(series)


def sample_points():
    points = [mpf(1) / 2 + mpf(k) / 64 for k in range(0, 97)]  # [0.5, 2]
    points += [mpf(1) + mpf(2) ** -e for e in range(1, 50)]  # just above 1
    points += [mpf(2) + mpf(2) ** -e for e in range(1, 50)]  # just above 2
    points += [mpf(1) - mpf(2) ** -e for e in range(2, 50)]  # just below 1
    points += [mpf(2) - mpf(2) ** -e for e in range(1, 50)]  # just below 2
    points += [mpf(10) ** (e / mpf(8)) for e in range(0, 8 * 300)]  # 1 .. 1e300
    return points


def check():
    mp.dps = 60
    exact = derive()
    table = shipped()
    ok = True
    for i, (e, t) in enumerate(zip(exact, table)):
        same = float(e) == t
        ok &= same
        print(f"c[{i}] derived {float(e).hex():>24}  shipped {t.hex():>24}  {'ok' if same else 'MISMATCH'}")
    rounded = [mpf(c) for c in table]
    near = {"exact": mpf(0), "double": mpf(0)}  # absolute, 1/2 <= x <= 3
    far = {"exact": mpf(0), "double": mpf(0)}  # relative, x > 3
    for x in sample_points():
        truth = loggamma(x)
        for name, coefficients in (("exact", exact), ("double", rounded)):
            error = abs(ln_gamma_lanczos(x, coefficients) - truth)
            if x <= 3:
                near[name] = max(near[name], error)
            else:
                far[name] = max(far[name], error / abs(truth))
    print("approximation error (exact coefficients):")
    print(f"  absolute, 1/2 <= x <= 3: {mp.nstr(near['exact'], 3)}   relative, x > 3: {mp.nstr(far['exact'], 3)}")
    print("with the double coefficients, evaluated exactly:")
    print(f"  absolute, 1/2 <= x <= 3: {mp.nstr(near['double'], 3)}   relative, x > 3: {mp.nstr(far['double'], 3)}")
    derived, limit = finite_limit(), shipped_limit()
    same = derived == limit
    ok &= same
    print(f"largest x with finite ln Gamma: derived {derived.hex()}  shipped {limit.hex()}  {'ok' if same else 'MISMATCH'}")
    return 0 if ok else 1


def reference():
    mp.dps = 50
    xs = [5e-324, 1e-310, 2.2250738585072014e-308, 1e-300, 1e-100, 1e-20, 1e-10, 1e-5, 0.001,
          0.1, 0.25, 0.4999999999999999, 0.5, 0.5000000000000001, 0.75,
          1 - 2 ** -52, 1.0, 1 + 2 ** -52, 1 + 1e-8, 1.4616321449683622, 1.5, 1.999999, 2 - 2 ** -51,
          2.0, 2 + 2 ** -51, 2.000001, 2.5, 3.0, 7.5, 10.0, 33.3, 100.0, 171.5, 1e3, 1e5, 1e10,
          1e15, 1e100, 1e300, 2.5e305, 2.557e305, 2.558e305, 2.559e305]
    for x in xs:
        print(f"        ({float(x)!r}, {float(loggamma(mpf(x)))!r}),")
    return 0


def sweep(path):
    import math
    import random

    mp.dps = 40
    rng = random.Random(20260916)
    xs = set()
    for _ in range(20000):
        xs.add(rng.uniform(0.5, 3.0))
    for centre in (0.5, 1.0, 2.0):
        for e in range(1, 53):
            xs.add(centre + 2.0 ** -e)
            xs.add(centre - 2.0 ** -e)
        for _ in range(5000):
            xs.add(centre + rng.uniform(-0.05, 0.05))
    for _ in range(5000):
        xs.add(10 ** rng.uniform(-323, 0))
    for _ in range(10000):
        xs.add(10 ** rng.uniform(0.47, 305))
    for _ in range(2000):
        xs.add(1e7 * (1 + rng.uniform(-1e-3, 1e-3)))  # the switch to Stirling
    for _ in range(2000):
        xs.add(10 ** rng.uniform(305, math.log10(finite_limit())))  # up to the finite limit
    with open(path, "w", encoding="utf-8") as out:
        for x in sorted(v for v in xs if v > 0):
            out.write(f"{x!r} {mp.nstr(loggamma(mpf(x)), 25)}\n")
    return 0


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--sweep":
        sys.exit(sweep(sys.argv[2]))
    if sys.argv[1:] == ["--check"]:
        sys.exit(check())
    if sys.argv[1:] == ["--reference"]:
        sys.exit(reference())
    print(__doc__)
    sys.exit(2)
