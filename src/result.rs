//! Test result type used by every test in the suite.

use std::fmt;

/// Significance level recommended by NIST SP 800-22 §4.2.1.
pub const ALPHA: f64 = 0.01;

/// What a [`TestResult`] holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// A p-value, finite and in [0, 1].
    Scored,
    /// The input was too short for the test, so no statistic was computed.
    Insufficient,
    /// The parameters are outside the test's defined domain, so no statistic
    /// was computed.  A battery never passes such parameters; a run that
    /// reports one has a configuration error and has not completed.
    Unsupported,
    /// A statistic was computed but its p-value is not a probability: NaN
    /// from a failed numerical expansion, an infinity, or a value outside
    /// [0, 1].  A run that reports one has not completed.
    Error,
}

/// The outcome of a single statistical test run against one RNG.
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Fully-qualified test name, e.g. `"nist::frequency"`.
    pub name: &'static str,
    /// The p-value when `status` is [`Status::Scored`]; otherwise NaN or the
    /// invalid value that was computed.
    pub p_value: f64,
    /// Optional human-readable note (e.g. parameter values used).
    pub note: Option<String>,
    /// Whether `p_value` is a probability, and if not, why.
    pub status: Status,
}

impl TestResult {
    /// A computed p-value with no note.  The status is [`Status::Scored`] for
    /// a finite value in [0, 1] and [`Status::Error`] for anything else.  The
    /// pass/fail verdict is not stored; [`passed`](Self::passed) judges
    /// `p_value` against [`ALPHA`] on demand.
    #[must_use]
    pub fn new(name: &'static str, p_value: f64) -> Self {
        Self {
            name,
            p_value,
            note: None,
            status: status_of(p_value),
        }
    }

    /// A computed p-value with an explanatory note (see [`new`](Self::new)).
    #[must_use]
    pub fn with_note(name: &'static str, p_value: f64, note: impl Into<String>) -> Self {
        Self {
            name,
            p_value,
            note: Some(note.into()),
            status: status_of(p_value),
        }
    }

    /// A result whose preconditions were not met (n too small, etc.).
    #[must_use]
    pub fn insufficient(name: &'static str, reason: &str) -> Self {
        Self {
            name,
            p_value: f64::NAN,
            note: Some(reason.to_owned()),
            status: Status::Insufficient,
        }
    }

    /// `true` if the p-value is a probability and at least [`ALPHA`].
    #[must_use]
    pub fn passed(&self) -> bool {
        self.status == Status::Scored && self.p_value >= ALPHA
    }

    /// `true` if the p-value is a probability below [`ALPHA`].
    #[must_use]
    pub fn failed(&self) -> bool {
        self.status == Status::Scored && self.p_value < ALPHA
    }

    /// A result whose parameters are outside the test's domain.
    #[must_use]
    pub fn unsupported(name: &'static str, reason: &str) -> Self {
        Self {
            name,
            p_value: f64::NAN,
            note: Some(reason.to_owned()),
            status: Status::Unsupported,
        }
    }

    /// `true` if the input was too short.
    #[must_use]
    pub fn skipped(&self) -> bool {
        self.status == Status::Insufficient
    }

    /// `true` if the parameters were outside the test's domain.
    #[must_use]
    pub fn is_unsupported(&self) -> bool {
        self.status == Status::Unsupported
    }

    /// `true` if the computed p-value is not a probability.
    #[must_use]
    pub fn errored(&self) -> bool {
        self.status == Status::Error
    }
}

impl Status {
    /// Lower-case name used in structured output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Scored => "scored",
            Status::Insufficient => "insufficient",
            Status::Unsupported => "unsupported",
            Status::Error => "error",
        }
    }
}

impl TestResult {
    /// One JSON object for this result: its name, status, unrounded p-value
    /// (`null` when not finite) and note, preceded by the caller's `context`
    /// members, each a key and an already-encoded JSON value.
    #[must_use]
    pub fn to_json(&self, context: &[(&str, String)]) -> String {
        let mut members: Vec<String> = context
            .iter()
            .map(|(key, value)| format!("{}:{value}", json_string(key)))
            .collect();
        members.push(format!("\"name\":{}", json_string(self.name)));
        members.push(format!("\"status\":\"{}\"", self.status.as_str()));
        let p = if self.p_value.is_finite() {
            format!("{:?}", self.p_value)
        } else {
            "null".to_owned()
        };
        members.push(format!("\"p_value\":{p}"));
        if let Some(note) = &self.note {
            members.push(format!("\"note\":{}", json_string(note)));
        }
        format!("{{{}}}", members.join(","))
    }
}

/// `s` as a JSON string literal.
#[must_use]
pub fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// [`Status::Scored`] for a finite p-value in [0, 1], else [`Status::Error`].
fn status_of(p_value: f64) -> Status {
    if (0.0..=1.0).contains(&p_value) {
        Status::Scored
    } else {
        Status::Error
    }
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status {
            Status::Insufficient => write!(f, "[SKIP] {:<48}  p = N/A", self.name)?,
            Status::Unsupported => write!(f, "[UNSUPPORTED] {:<41}  p = N/A", self.name)?,
            Status::Error => write!(f, "[ERROR] {:<47}  p = {}", self.name, self.p_value)?,
            Status::Scored => {
                let status = if self.passed() { "PASS" } else { "FAIL" };
                if self.p_value > 0.0 && self.p_value < 1e-6 {
                    // Tiny but non-zero p-values would round to "0.000000"; use
                    // scientific notation so the magnitude of the failure is visible.
                    write!(f, "[{status}] {:<48}  p = {:.3e}", self.name, self.p_value)?;
                } else {
                    write!(f, "[{status}] {:<48}  p = {:.6}", self.name, self.p_value)?;
                }
            }
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

    /// Values that are not probabilities are errors, never PASS or SKIP.
    #[test]
    fn invalid_p_values_are_errors() {
        for p in [f64::INFINITY, f64::NEG_INFINITY, 1.25, -0.5, f64::NAN] {
            let r = TestResult::new("t", p);
            assert!(
                r.errored() && !r.passed() && !r.failed() && !r.skipped(),
                "{p}"
            );
            assert!(r.to_string().starts_with("[ERROR]"), "{r}");
        }
        assert!(TestResult::new("t", 1.0).passed());
        assert!(TestResult::new("t", 0.0).failed());
    }

    #[test]
    fn skip_pass_fail_triage() {
        assert!(TestResult::insufficient("t", "why").skipped());
        assert!(!TestResult::insufficient("t", "why").passed());
        assert!(TestResult::new("t", 0.5).passed());
        assert!(!TestResult::new("t", 0.001).passed());
    }

    /// Unsupported parameters are neither a pass, a failure, a skip nor an
    /// error, and print their own tag.
    #[test]
    fn unsupported_is_its_own_status() {
        let r = TestResult::unsupported("t", "d must be 2..=5");
        assert!(r.is_unsupported());
        assert!(!r.passed() && !r.failed() && !r.skipped() && !r.errored());
        assert!(r.to_string().starts_with("[UNSUPPORTED] t"), "{r}");
    }

    /// Structured output keeps the unrounded p-value and escapes strings.
    #[test]
    fn json_keeps_the_unrounded_p_value() {
        let r = TestResult::with_note("t::x", 0.1 + 0.2, "a \"q\"\\\n\u{1}");
        assert_eq!(
            r.to_json(&[
                ("generator", json_string("G")),
                ("input_start", "1048576".into())
            ]),
            r#"{"generator":"G","input_start":1048576,"name":"t::x","status":"scored","p_value":0.30000000000000004,"note":"a \"q\"\\\n\u0001"}"#
        );
        let skip = TestResult::insufficient("t::y", "short");
        assert!(skip
            .to_json(&[])
            .contains(r#""status":"insufficient","p_value":null"#));
    }
}
