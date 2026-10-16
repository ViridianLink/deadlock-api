use std::time::Duration;

use reqwest::header::{HeaderMap, RETRY_AFTER};

const DEFAULT: Duration = Duration::from_secs(30);
const MIN_SECS: f64 = 1.0;
const MAX_SECS: f64 = 120.0;

#[must_use]
pub fn retry_after(headers: &HeaderMap) -> Duration {
    let seconds = headers
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<f64>().ok())
        .filter(|seconds| seconds.is_finite());

    seconds.map_or(DEFAULT, |seconds| {
        Duration::try_from_secs_f64(seconds.clamp(MIN_SECS, MAX_SECS))
            .unwrap_or(DEFAULT)
    })
}
