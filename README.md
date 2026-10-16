# deadlock-api

A typed async client for the [deadlock-api.com](https://deadlock-api.com/) Deadlock API.

Not endorsed by Valve or affiliated with deadlock-api.com.

## Usage

```toml
[dependencies]
deadlock-api = { git = "https://github.com/ViridianLink/deadlock-api" }
```

```rust
use deadlock_api::{DeadlockClient, Priority};

async fn run() -> deadlock_api::Result<()> {
    let client = DeadlockClient::builder()
        .api_key("your-api-key") // optional
        .build()?;

    let heroes = client.heroes(None, Priority::Interactive).await?;
    println!("{} heroes", heroes.len());

    let ranks = client.player_ranks(&[35_803_007], Priority::Background).await?;
    println!("{ranks:?}");
    Ok(())
}
```

## Rate limits

Every request goes through a client-side token bucket per endpoint group, sized
to deadlock-api.com's published limits (larger when an API key is set).
`Priority::Interactive` requests may wait a few seconds for a token, while
`Priority::Background` requests leave a reserve for interactive use and fail
immediately with `DeadlockApiError::Busy` when the bucket is short.

When the API answers 429 or 503, the client reads `Retry-After` and blocks that
bucket for the duration; calls until then return `Busy` without sending a
request. `DeadlockApiError::retry_after` exposes the delay.

## Development

The crate uses the nightly toolchain pinned in `rust-toolchain.toml` (for
`rustfmt`'s unstable options).

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
