//! The fourteen seeded generators that `bib_tests`, `gorilla`, `testu01_lz`
//! and `upstream_tests` run, with the labels and seeds those binaries print.
//!
//! Each binary includes this file (and `cli.rs`) with `#[path]`.  A binary
//! supplies a [`Visit`] implementation; generators reach it one at a time, as
//! concrete types, and only generators whose label passes `--rng` are ever
//! constructed.

use crate::cli::RngFilter;
use entropy::rng::{
    AesCtr, BsdRandom, CryptoCtrDrbg, Lcg32, LcgVariant, LinuxLibcRandom, Mt19937, Rand48, Rng,
    SystemVRand, WindowsDotNetRandom, WindowsMsvcRand, WindowsVb6Rnd, Xorshift32, Xorshift64,
};
use entropy::seed::seed_material;

/// Receives the generators of the family.
pub trait Visit {
    /// One generator: its label and a constructor for it.
    fn case<R: Rng>(&mut self, label: &'static str, make: impl FnOnce() -> R);
}

/// Hand every generator whose label passes `filter` to `visit`, in the order
/// the binaries print them, and return how many there were.
pub fn visit_matching(filter: &RngFilter, visit: &mut impl Visit) -> usize {
    let mut matching = Matching {
        filter,
        visit,
        matched: 0,
    };
    seeded_family(&mut matching);
    matching.matched
}

struct Matching<'a, V> {
    filter: &'a RngFilter,
    visit: &'a mut V,
    matched: usize,
}

impl<V: Visit> Visit for Matching<'_, V> {
    fn case<R: Rng>(&mut self, label: &'static str, make: impl FnOnce() -> R) {
        if self.filter.matches(label) {
            self.matched += 1;
            self.visit.case(label, make);
        }
    }
}

fn seeded_family(visit: &mut impl Visit) {
    visit.case("MT19937", || Mt19937::new(19650218));
    visit.case("Xorshift32", || Xorshift32::new(1));
    visit.case("Xorshift64", || Xorshift64::new(1));
    visit.case("BAD Unix System V rand()", || SystemVRand::new(1));
    visit.case("BAD Unix System V mrand48()", || Rand48::new(1));
    visit.case("BAD Unix BSD random()", || BsdRandom::new(1));
    visit.case("BAD Unix Linux glibc rand()/random()", || {
        LinuxLibcRandom::new(1)
    });
    visit.case("BAD Windows CRT rand()", || WindowsMsvcRand::new(1));
    visit.case("BAD Windows VB6/VBA Rnd()", || WindowsVb6Rnd::new(1));
    visit.case("BAD Windows .NET Random(seed)", || {
        WindowsDotNetRandom::new(1)
    });
    visit.case("ANSI C sample LCG", || Lcg32::new(LcgVariant::AnsiC, 1));
    visit.case("LCG MINSTD", || Lcg32::new(LcgVariant::Minstd, 1));
    visit.case("AES-128-CTR", || AesCtr::new(&seed_material::<16>(1), 0));
    visit.case("cryptography::CtrDrbgAes256", || {
        CryptoCtrDrbg::new(&seed_material::<48>(1))
    });
}
