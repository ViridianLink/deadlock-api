use std::fmt;

use serde::{Deserialize, Serialize};

/// A tooltip/stat value: the API sends either a JSON number or a string such as
/// `"20m"` or `"0.1"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StatValue {
    Number(f64),
    Text(String),
}

impl StatValue {
    /// Number → itself. Text → parse after trimming whitespace and stripping a
    /// trailing run of ASCII alphabetic chars and `%` (so "20m" → 20.0, "1.5s" →
    /// 1.5, "12%" → 12.0); None if that fails.
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            Self::Text(text) => text
                .trim()
                .trim_end_matches(|c: char| c.is_ascii_alphabetic() || c == '%')
                .parse()
                .ok(),
        }
    }
}

impl fmt::Display for StatValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::Text(text) => f.write_str(text),
        }
    }
}
