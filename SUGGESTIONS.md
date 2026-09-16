# Suggestions for entropy

Reviewed 2026-09-11 against commit `ffb3aee`. This file owns the DIEHARD
preservation and reusable battery-input recommendations previously placed in
cryptography's suggestions. Cryptography's cipher corpus, integration and R
calibration work stays in [its SUGGESTIONS.md](../cryptography/SUGGESTIONS.md).
This update changes documentation only.

## Preserve historical DIEHARD sources and data

Source-distribution recovery is no longer an open gap: the repository now
tracks the following readable archives. Their SHA-256 digests were checked
against the provenance entries in [pubs/SOURCES.tsv](pubs/SOURCES.tsv):

| Archive | SHA-256 |
|---|---|
| `pubs/diehard-fortran-1996.tar.gz` | `99666a65c58b39802d8fabf545341c835a2746f41bf0d130e292805053eb3fe1` |
| `pubs/diehard-f2c-source-1996.tar.gz` | `e68b14abec617f459ee3f3232d0eca957444093f9ac038abef880f15329b9750` |
| `pubs/diehard-c-wang-1998.tar.gz` | `e028a755c1441e5af90f060cd5d8fceaa5967a6a66fbc1f4c66410be712208ae` |

The archived download origins are recorded in that manifest. The separate
`pubs/Diehard.zip` contains DOS binaries, data and documentation; it is not
the Fortran/C source distribution. These checks establish what is available
now, not the identity of the owner's particular missing copy.

Keep source distributions, reference data and provenance as tracked artifacts
when disabling a test. The active Rust implementations remain in
[src/diehard](src/diehard) and [src/dieharder](src/dieharder). The removed
`operm5.rs` and `overlapping_sums.rs` implementations remain readable from
the parent of deletion commit `3b41af8`, without changing the checkout:

```sh
git show 3b41af8^:src/diehard/operm5.rs
git show 3b41af8^:src/diehard/overlapping_sums.rs
```

Make those historical implementations discoverable in a repository-local
inventory, with their revision and the reason each test was disabled. If a
historical comparison mode is provided, label its variants and limitations
explicitly. Evaluate disputed statistics before restoring a test to the
default battery; a calibration problem does not make its source disposable.
Preserving a reference implementation does not authorize copying its code
into independently derived cryptographic primitives.

## Support reproducible testing of finite byte corpora

Entropy should own a reusable finite-input adapter so downstream users can
run the maintained battery over exactly the bytes they previously tested.
The existing stream and block-CTR RNG wrappers cover generated streams;
replaying a saved ciphertext corpus needs an explicit finite-input contract.

Specify word endianness, bit order, partial-word handling, bytes consumed,
and whether each test starts at the beginning or consumes a disjoint segment.
Report insufficient input explicitly. Never silently cycle a short file or
pad it with zeros. Include the corpus digest, test variant, parameters,
consumption and skipped tests in the output so another run is reproducible.

Verify the adapter with a short, known byte sequence and an exhausted input.
Confirm that generated-stream and finite-corpus paths consume identical
bytes when given the same sequence. Cryptography can then integrate this
adapter without maintaining a second DIEHARD implementation in its R script.
