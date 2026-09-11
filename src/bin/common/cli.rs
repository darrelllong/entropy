//! Command-line plumbing shared by the research binaries (`bib_tests`,
//! `bitplane_complexity`, `gorilla`, `testu01_lz`, `upstream_tests` and
//! `webster_tavares`): an argument cursor, the `--rng` label filter, and the
//! usage-error exit.
//!
//! Each binary includes this file with `#[path]`.  Its conventions are the
//! ones those binaries already shared: `--help` or `-h` wherever an option is
//! expected prints the usage text to stderr and exits 0; a usage error prints
//! `error: <message>` to stderr and exits 1; an option's value is the next
//! argument, taken verbatim even if it begins with `-`.

use std::process;

/// Why parsing stopped without producing arguments.
#[derive(Debug, PartialEq, Eq)]
pub enum Stop {
    /// `--help` or `-h` where an option was expected.
    Help,
    /// A usage error, reported as `error: <message>`.
    Usage(String),
}

/// A usage error with `message`.
pub fn usage(message: impl Into<String>) -> Stop {
    Stop::Usage(message.into())
}

/// The usage error for an argument no option matched.
pub fn unknown_option(option: &str) -> Stop {
    usage(format!("unknown option '{option}'"))
}

/// A cursor over the arguments that follow the program name.
pub struct Argv {
    args: std::vec::IntoIter<String>,
}

impl Argv {
    /// Wrap `args`, which must not include the program name.
    pub fn new(args: impl IntoIterator<Item = String>) -> Self {
        Self {
            args: args.into_iter().collect::<Vec<_>>().into_iter(),
        }
    }

    /// The next option, `Ok(None)` once the arguments run out, or
    /// [`Stop::Help`] for `--help` or `-h`.
    pub fn next_option(&mut self) -> Result<Option<String>, Stop> {
        match self.args.next() {
            Some(arg) if arg == "--help" || arg == "-h" => Err(Stop::Help),
            next => Ok(next),
        }
    }

    /// The argument after `flag`, verbatim.
    pub fn value(&mut self, flag: &str) -> Result<String, Stop> {
        self.args
            .next()
            .ok_or_else(|| usage(format!("{flag} requires an argument")))
    }

    /// The argument after `flag` as a `usize`, parsed by `str::parse`; a value
    /// that does not parse is reported as `invalid <flag> value`.
    // `gorilla` and `upstream_tests` take no option read this way.
    #[allow(dead_code)]
    pub fn usize_value(&mut self, flag: &str) -> Result<usize, Stop> {
        self.value(flag)?
            .parse()
            .map_err(|_| usage(format!("invalid {flag} value")))
    }
}

/// The `--rng` patterns given so far.  A label matches if it contains any
/// pattern, ignoring case; with no patterns every label matches.
#[derive(Debug, Default)]
pub struct RngFilter {
    patterns: Vec<String>,
}

impl RngFilter {
    /// Add one `--rng` pattern.
    pub fn push(&mut self, pattern: String) {
        self.patterns.push(pattern);
    }

    /// Whether `label` passes the filter.
    pub fn matches(&self, label: &str) -> bool {
        let label = label.to_lowercase();
        self.patterns.is_empty()
            || self
                .patterns
                .iter()
                .any(|pattern| label.contains(&pattern.to_lowercase()))
    }
}

/// Print `error: <message>` to stderr and exit 1.
pub fn die(message: &str) -> ! {
    eprintln!("error: {message}");
    process::exit(1);
}

/// Exit through [`die`] because no generator label matched `--rng`.
pub fn die_no_rng_matched() -> ! {
    die("no RNG labels matched --rng filter");
}

/// Parse the process arguments with `parse`.  On [`Stop::Help`] print the
/// usage text and exit 0; on [`Stop::Usage`] exit through [`die`].
pub fn parse_or_exit<T>(parse: impl FnOnce(Argv) -> Result<T, Stop>, print_usage: fn()) -> T {
    match parse(Argv::new(std::env::args().skip(1))) {
        Ok(args) => args,
        Err(Stop::Help) => {
            print_usage();
            process::exit(0);
        }
        Err(Stop::Usage(message)) => die(&message),
    }
}

#[cfg(test)]
mod tests {
    use super::{unknown_option, usage, Argv, RngFilter, Stop};

    fn argv(args: &[&str]) -> Argv {
        Argv::new(args.iter().map(|arg| arg.to_string()))
    }

    #[test]
    fn options_run_out_and_help_stops_parsing() {
        assert_eq!(Ok(None), argv(&[]).next_option());
        let mut args = argv(&["--k", "-h", "--help"]);
        assert_eq!(Ok(Some("--k".to_string())), args.next_option());
        assert_eq!(Err(Stop::Help), args.next_option());
        assert_eq!(Err(Stop::Help), args.next_option());
        assert_eq!(Ok(None), args.next_option());
    }

    /// A value is the next argument whatever it looks like, so `--rng --help`
    /// filters on the text `--help` instead of printing help.
    #[test]
    fn values_are_taken_verbatim() {
        let mut args = argv(&["--rng", "--help", "--rng", "", "-h"]);
        for want in ["--help", ""] {
            assert_eq!(Ok(Some("--rng".to_string())), args.next_option());
            assert_eq!(Ok(want.to_string()), args.value("--rng"));
        }
        assert_eq!(Err(Stop::Help), args.next_option());
    }

    #[test]
    fn missing_and_invalid_values_name_the_flag() {
        let mut args = argv(&[]);
        assert_eq!(
            Err(usage("--words requires an argument")),
            args.usize_value("--words")
        );
        for bad in ["x", "-1", "1.5", "", " 7", "18446744073709551616"] {
            assert_eq!(
                Err(usage("invalid --k value")),
                argv(&[bad]).usize_value("--k"),
                "{bad:?}"
            );
        }
        // `str::parse` accepts a leading plus sign.
        assert_eq!(Ok(5), argv(&["+5"]).usize_value("--k"));
        assert_eq!(Ok(0), argv(&["0"]).usize_value("--k"));
    }

    #[test]
    fn unknown_options_are_quoted() {
        assert_eq!(
            Stop::Usage("unknown option '--rng=AES'".to_string()),
            unknown_option("--rng=AES")
        );
    }

    #[test]
    fn rng_filter_matches_any_pattern_as_a_case_insensitive_substring() {
        let mut filter = RngFilter::default();
        assert!(filter.matches("MT19937"), "no pattern matches everything");
        filter.push("aes".to_string());
        assert!(filter.matches("AES-128-CTR"));
        assert!(filter.matches("cryptography::CtrDrbgAes256"));
        assert!(!filter.matches("MT19937"));
        filter.push("GLIBC RAND".to_string());
        assert!(filter.matches("BAD Unix Linux glibc rand()/random()"));
        assert!(!filter.matches("BAD Unix BSD random()"));
    }
}
