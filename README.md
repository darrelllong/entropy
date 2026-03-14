# entropy

A pure, safe Rust statistical test suite for pseudorandom number generators.

Implements every test from:
- **NIST SP 800-22 Rev 1a** — the canonical 15-test battery for cryptographic RNG evaluation
- **DIEHARD** (Marsaglia, 1995) — tests not already in NIST SP 800-22
- **DIEHARDER** (Brown, 2006) — tests not already in NIST or DIEHARD

Original reference PDFs are in `pubs/`.

## Running

```sh
cargo run --release
```

This runs every test against several RNGs available on macOS:

| RNG | Expected outcome |
|-----|-----------------|
| `/dev/urandom` (OsRng) | Pass all tests |
| Xorshift32 (Marsaglia) | Pass most; may fail linear-complexity |
| LCG-bad (glibc `rand` parameters) | Fail spectral, runs, serial, … |
| MINSTD (Park-Miller) | Fail several |
| Constant (`0xDEAD_DEAD`) | Fail everything |
| Counter (`0, 1, 2, …`) | Fail everything |

## Attribution rules

Any function whose algorithm is adapted from DIEHARD or DIEHARDER carries
a `# Author` line in its doc-comment citing the original author.

---

## Bibliography

All referenced documents are in `pubs/`. BibTeX entries for every source:

```bibtex
@techreport{nist-sp-800-22,
  author      = {Rukhin, Andrew and Soto, Juan and Nechvatal, James and
                 Smid, Miles and Barker, Elaine and Leigh, Stefan and
                 Levenson, Mark and Vangel, Mark and Banks, David and
                 Heckert, Alan and Dray, James and Vo, San},
  title       = {{A Statistical Test Suite for Random and Pseudorandom
                  Number Generators for Cryptographic Applications}},
  institution = {National Institute of Standards and Technology},
  year        = {2010},
  number      = {SP 800-22 Rev 1a},
  type        = {Special Publication},
  doi         = {10.6028/NIST.SP.800-22r1a},
}

@techreport{nist-sp-800-90a,
  author      = {Barker, Elaine and Kelsey, John},
  title       = {{Recommendation for Random Number Generation Using
                  Deterministic Random Bit Generators}},
  institution = {National Institute of Standards and Technology},
  year        = {2015},
  number      = {SP 800-90A Rev 1},
  type        = {Special Publication},
  doi         = {10.6028/NIST.SP.800-90Ar1},
}

@techreport{nist-sp-800-90b,
  author      = {Turan, Meltem S{\"o}nmez and Barker, Elaine and Kelsey,
                 John and McKay, Kerry A. and Baish, Mary L. and Boyle, Mike},
  title       = {{Recommendation for the Entropy Sources Used for Random
                  Bit Generation}},
  institution = {National Institute of Standards and Technology},
  year        = {2018},
  number      = {SP 800-90B},
  type        = {Special Publication},
  doi         = {10.6028/NIST.SP.800-90B},
}

@techreport{nist-sp-800-90c,
  author      = {Barker, Elaine and Kelsey, John and McKay, Kerry},
  title       = {{Recommendation for Random Bit Generator (RBG) Constructions}},
  institution = {National Institute of Standards and Technology},
  year        = {2024},
  number      = {SP 800-90C},
  type        = {Special Publication},
}

@techreport{fips-140-3,
  author      = {{National Institute of Standards and Technology}},
  title       = {{Security Requirements for Cryptographic Modules}},
  institution = {National Institute of Standards and Technology},
  year        = {2019},
  number      = {FIPS 140-3},
  type        = {Federal Information Processing Standard},
  doi         = {10.6028/NIST.FIPS.140-3},
}

@misc{marsaglia1995diehard,
  author      = {Marsaglia, George},
  title       = {{DIEHARD: A Battery of Tests of Randomness}},
  year        = {1995},
  howpublished= {CD-ROM. Florida State University.
                 \url{https://stat.fsu.edu/pub/diehard/}},
}

@misc{brown2006dieharder,
  author      = {Brown, Robert G.},
  title       = {{Dieharder: A Random Number Test Suite}},
  year        = {2006},
  note        = {Version 3.31.1},
  howpublished= {\url{https://webhome.phy.duke.edu/~rgb/General/dieharder.php}},
}

@article{marsaglia2002gcd,
  author  = {Marsaglia, George and Tsang, Wai Wan},
  title   = {{Some Difficult-to-pass Tests of Randomness}},
  journal = {Journal of Statistical Software},
  year    = {2002},
  volume  = {7},
  number  = {3},
  pages   = {1--9},
  doi     = {10.18637/jss.v007.i03},
}

@phdthesis{hughes2021badrandom,
  author  = {Hughes, James Prescott},
  title   = {{BADRANDOM: The Effect and Mitigations for Low Entropy
              Random Numbers in TLS}},
  school  = {University of California, Santa Cruz},
  year    = {2021},
  month   = {December},
  note    = {Committee chair: Professor Darrell Long},
}
```
