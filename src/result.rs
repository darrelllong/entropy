//! Test result type used by every test in the suite.

use std::fmt;

/// Significance level recommended by NIST SP 800-22 §4.2.1.
pub const ALPHA: f64 = 0.01;

/// The outcome of a single statistical test run against one RNG.
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Fully-qualified test name, e.g. `"nist::frequency"`.
    pub name: &'static str,
    /// Computed p-value.  `NAN` indicates a pre-condition failure
    /// (sequence too short, etc.).
    pub p_value: f64,
    /// Significance level used (default [`ALPHA`] = 0.01).
    pub alpha: f64,
    /// Optional human-readable note (e.g. parameter values used).
    pub note: Option<String>,
}

impl TestResult {
    /// Construct a result with the default significance level.
    pub fn new(name: &'static str, p_value: f64) -> Self {
        Self {
            name,
            p_value,
            alpha: ALPHA,
            note: None,
        }
    }

    /// Construct a result with an explanatory note.
    pub fn with_note(name: &'static str, p_value: f64, note: impl Into<String>) -> Self {
        Self {
            name,
            p_value,
            alpha: ALPHA,
            note: Some(note.into()),
        }
    }

    /// A result whose preconditions were not met (n too small, etc.).
    pub fn insufficient(name: &'static str, reason: &str) -> Self {
        Self {
            name,
            p_value: f64::NAN,
            alpha: ALPHA,
            note: Some(reason.to_owned()),
        }
    }

    /// `true` if p_value ≥ alpha (the sequence is not rejected at this level).
    pub fn passed(&self) -> bool {
        self.p_value >= self.alpha
    }

    /// `true` if the preconditions were not met (`p_value` is NaN).
    pub fn skipped(&self) -> bool {
        self.p_value.is_nan()
    }
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.skipped() {
            "SKIP"
        } else if self.passed() {
            "PASS"
        } else {
            "FAIL"
        };
        if self.skipped() {
            write!(f, "[{status}] {:<48}  p = N/A", self.name)?;
        } else {
            write!(f, "[{status}] {:<48}  p = {:.6}", self.name, self.p_value)?;
        }
        if let Some(n) = &self.note {
            write!(f, "  ({n})")?;
        }
        Ok(())
    }
}
