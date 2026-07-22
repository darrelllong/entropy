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
    /// Optional human-readable note (e.g. parameter values used).
    pub note: Option<String>,
}

impl TestResult {
    /// Construct a result with no note.  The pass/fail verdict is not stored;
    /// [`passed`](Self::passed) judges `p_value` against [`ALPHA`] on demand.
    #[must_use]
    pub fn new(name: &'static str, p_value: f64) -> Self {
        Self {
            name,
            p_value,
            note: None,
        }
    }

    /// Construct a result with an explanatory note.
    #[must_use]
    pub fn with_note(name: &'static str, p_value: f64, note: impl Into<String>) -> Self {
        Self {
            name,
            p_value,
            note: Some(note.into()),
        }
    }

    /// A result whose preconditions were not met (n too small, etc.).
    #[must_use]
    pub fn insufficient(name: &'static str, reason: &str) -> Self {
        Self {
            name,
            p_value: f64::NAN,
            note: Some(reason.to_owned()),
        }
    }

    /// `true` if p_value ≥ alpha (the sequence is not rejected at this level).
    ///
    /// A NaN p-value (a skipped test) also returns `false`, so callers that
    /// must distinguish FAIL from SKIP should check
    /// [`skipped`](Self::skipped) first.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.p_value >= ALPHA
    }

    /// `true` if the preconditions were not met (`p_value` is NaN).
    #[must_use]
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
        } else if self.p_value > 0.0 && self.p_value < 1e-6 {
            // Tiny but non-zero p-values would round to "0.000000"; use
            // scientific notation so the magnitude of the failure is visible.
            write!(f, "[{status}] {:<48}  p = {:.3e}", self.name, self.p_value)?;
        } else {
            write!(f, "[{status}] {:<48}  p = {:.6}", self.name, self.p_value)?;
        }
        if let Some(n) = &self.note {
            write!(f, "  ({n})")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_uses_scientific_notation_for_tiny_p() {
        let tiny = TestResult::new("t", 3.2e-9);
        assert!(tiny.to_string().contains("p = 3.200e-9"), "{tiny}");
        // Exact zero keeps the fixed-decimal form (deliberate).
        let zero = TestResult::new("t", 0.0);
        assert!(zero.to_string().contains("p = 0.000000"), "{zero}");
        // Ordinary p-values keep six decimals.
        let mid = TestResult::new("t", 0.5);
        assert!(mid.to_string().contains("p = 0.500000"), "{mid}");
    }

    #[test]
    fn skip_pass_fail_triage() {
        assert!(TestResult::insufficient("t", "why").skipped());
        assert!(!TestResult::insufficient("t", "why").passed());
        assert!(TestResult::new("t", 0.5).passed());
        assert!(!TestResult::new("t", 0.001).passed());
    }
}
