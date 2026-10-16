#![expect(clippy::unwrap_used, reason = "Unwrap allowed in tests")]

use std::time::Duration;

use deadlock_api::{Bucket, BucketConfig, DeadlockApiError, Limiter, Priority};
use tokio::time::{Instant, advance};

const BUCKET: Bucket = Bucket::Assets;

fn limiter(capacity: u32, per: u64, reserved: u32, max_wait: u64) -> Limiter {
    Limiter::with_configs(
        [(BUCKET, BucketConfig {
            capacity,
            per: Duration::from_secs(per),
            reserved,
        })],
        Duration::from_secs(max_wait),
    )
}

fn busy_for(result: Result<(), DeadlockApiError>) -> Duration {
    result.unwrap_err().retry_after().unwrap()
}

fn assert_close(actual: Duration, expected: Duration) {
    let difference = actual.abs_diff(expected);
    assert!(
        difference < Duration::from_millis(5),
        "expected about {expected:?}, got {actual:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn capacity_runs_out_and_refills_over_time() {
    let limiter = limiter(2, 10, 0, 0);

    limiter.acquire(BUCKET, Priority::Interactive).await.unwrap();
    limiter.acquire(BUCKET, Priority::Interactive).await.unwrap();

    let retry_after = busy_for(limiter.acquire(BUCKET, Priority::Interactive).await);
    assert_close(retry_after, Duration::from_secs(5));

    advance(Duration::from_secs(5)).await;
    limiter.acquire(BUCKET, Priority::Interactive).await.unwrap();
    busy_for(limiter.acquire(BUCKET, Priority::Interactive).await);
}

#[tokio::test(start_paused = true)]
async fn background_stops_at_the_reserve_while_interactive_continues() {
    let limiter = limiter(4, 60, 2, 0);

    limiter.acquire(BUCKET, Priority::Background).await.unwrap();
    limiter.acquire(BUCKET, Priority::Background).await.unwrap();
    busy_for(limiter.acquire(BUCKET, Priority::Background).await);

    limiter.acquire(BUCKET, Priority::Interactive).await.unwrap();
    limiter.acquire(BUCKET, Priority::Interactive).await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn interactive_waits_for_a_token_within_the_budget() {
    let limiter = limiter(1, 2, 0, 5);
    limiter.acquire(BUCKET, Priority::Interactive).await.unwrap();

    let started = Instant::now();
    limiter.acquire(BUCKET, Priority::Interactive).await.unwrap();

    assert_close(started.elapsed(), Duration::from_secs(2));
}

#[tokio::test(start_paused = true)]
async fn interactive_beyond_the_budget_is_busy() {
    let limiter = limiter(1, 60, 0, 5);
    limiter.acquire(BUCKET, Priority::Interactive).await.unwrap();

    let started = Instant::now();
    let retry_after = busy_for(limiter.acquire(BUCKET, Priority::Interactive).await);

    assert_close(retry_after, Duration::from_secs(60));
    assert_eq!(started.elapsed(), Duration::ZERO);
}

#[tokio::test(start_paused = true)]
async fn a_block_refuses_background_immediately() {
    let limiter = limiter(100, 60, 0, 5);
    limiter.block(BUCKET, Duration::from_secs(7));

    let retry_after = busy_for(limiter.acquire(BUCKET, Priority::Background).await);
    assert_close(retry_after, Duration::from_secs(7));

    advance(Duration::from_secs(2)).await;
    let retry_after = busy_for(limiter.acquire(BUCKET, Priority::Background).await);
    assert_close(retry_after, Duration::from_secs(5));

    advance(Duration::from_secs(5)).await;
    limiter.acquire(BUCKET, Priority::Background).await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn a_short_block_makes_interactive_wait() {
    let limiter = limiter(100, 60, 0, 5);
    limiter.block(BUCKET, Duration::from_secs(3));

    let started = Instant::now();
    limiter.acquire(BUCKET, Priority::Interactive).await.unwrap();

    assert_close(started.elapsed(), Duration::from_secs(3));
}

#[tokio::test(start_paused = true)]
async fn a_long_block_makes_interactive_busy() {
    let limiter = limiter(100, 60, 0, 5);
    limiter.block(BUCKET, Duration::from_secs(30));
    limiter.block(BUCKET, Duration::from_secs(10));

    let retry_after = busy_for(limiter.acquire(BUCKET, Priority::Interactive).await);

    assert_close(retry_after, Duration::from_secs(30));
}

#[tokio::test(start_paused = true)]
async fn an_unconfigured_bucket_is_unlimited() {
    let limiter = Limiter::with_configs([], Duration::ZERO);

    for _ in 0..50 {
        limiter.acquire(Bucket::Analytics, Priority::Background).await.unwrap();
    }
}
