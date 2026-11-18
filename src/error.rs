use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::time::Duration;

use reqwest::StatusCode;
use reqwest::header::InvalidHeaderValue;

pub type Result<T> = std::result::Result<T, DeadlockApiError>;

#[derive(Debug)]
#[non_exhaustive]
pub enum DeadlockApiError {
    Busy { retry_after: Duration },
    NotFound,
    Http { status: StatusCode },
    Decode { path: String, source: serde_json::Error },
    Request(reqwest::Error),
    InvalidHeaderValue(InvalidHeaderValue),
    InvalidPath { path: String },
    InvalidBaseUrl { url: String, source: Option<url::ParseError> },
}

impl DeadlockApiError {
    #[must_use]
    pub const fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::Busy { retry_after } => Some(*retry_after),
            Self::NotFound
            | Self::Http { .. }
            | Self::Decode { .. }
            | Self::Request(_)
            | Self::InvalidHeaderValue(_)
            | Self::InvalidPath { .. }
            | Self::InvalidBaseUrl { .. } => None,
        }
    }
}

impl Display for DeadlockApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Busy { retry_after } => {
                let seconds = retry_after.as_secs().max(1);
                write!(f, "deadlock-api is busy; retry in {seconds}s")
            },
            Self::NotFound => {
                f.write_str("deadlock-api has no data for that request")
            },
            Self::Http { status } => write!(f, "HTTP error {status}"),
            Self::Decode { path, source } => {
                write!(f, "failed to decode {path}: {source}")
            },
            Self::Request(e) => write!(f, "request failed: {e}"),
            Self::InvalidHeaderValue(e) => write!(f, "invalid header value: {e}"),
            Self::InvalidPath { path } => {
                write!(f, "invalid request path `{path}`: it must start with `/`")
            },
            Self::InvalidBaseUrl { url, source } => {
                write!(f, "invalid base URL `{url}`")?;
                source.as_ref().map_or(Ok(()), |e| write!(f, ": {e}"))
            },
        }
    }
}

impl Error for DeadlockApiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Decode { source, .. } => Some(source),
            Self::Request(e) => Some(e),
            Self::InvalidHeaderValue(e) => Some(e),
            Self::InvalidBaseUrl { source: Some(e), .. } => Some(e),
            Self::Busy { .. }
            | Self::NotFound
            | Self::Http { .. }
            | Self::InvalidPath { .. }
            | Self::InvalidBaseUrl { source: None, .. } => None,
        }
    }
}

impl From<reqwest::Error> for DeadlockApiError {
    fn from(value: reqwest::Error) -> Self {
        Self::Request(value)
    }
}

impl From<InvalidHeaderValue> for DeadlockApiError {
    fn from(value: InvalidHeaderValue) -> Self {
        Self::InvalidHeaderValue(value)
    }
}
