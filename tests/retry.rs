#![expect(clippy::unwrap_used, reason = "Unwrap allowed in tests")]

use std::time::Duration;

use deadlock_api::retry_after;
use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};

fn headers(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(RETRY_AFTER, HeaderValue::from_str(value).unwrap());
    headers
}

#[test]
fn integer_seconds_are_honoured() {
    assert_eq!(retry_after(&headers("7")), Duration::from_secs(7));
}

#[test]
fn fractional_seconds_are_honoured() {
    assert_eq!(retry_after(&headers("2.5")), Duration::from_millis(2500));
}

#[test]
fn a_missing_header_defaults_to_thirty_seconds() {
    assert_eq!(retry_after(&HeaderMap::new()), Duration::from_secs(30));
}

#[test]
fn a_huge_value_is_clamped_to_two_minutes() {
    assert_eq!(retry_after(&headers("99999")), Duration::from_secs(120));
}

#[test]
fn zero_is_raised_to_one_second() {
    assert_eq!(retry_after(&headers("0")), Duration::from_secs(1));
}

#[test]
fn garbage_defaults_to_thirty_seconds() {
    assert_eq!(retry_after(&headers("soon")), Duration::from_secs(30));
    assert_eq!(retry_after(&headers("NaN")), Duration::from_secs(30));
}
