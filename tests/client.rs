#![expect(clippy::unwrap_used, reason = "Unwrap allowed in tests")]

use std::fmt::Write as _;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use deadlock_api::{
    Bucket,
    BucketConfig,
    DeadlockApiError,
    DeadlockClient,
    Limiter,
    Priority,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const RANKS: &str = include_str!("fixtures/ranks.json");
const RANK_BATCH: &str = include_str!("fixtures/rank_batch.json");
const METADATA_BULK: &str = include_str!("fixtures/metadata_bulk.json");

type Requests = Arc<Mutex<Vec<String>>>;

struct Reply {
    status: u16,
    headers: Vec<(&'static str, &'static str)>,
    body: &'static str,
}

impl Reply {
    const fn ok(body: &'static str) -> Self {
        Self { status: 200, headers: Vec::new(), body }
    }

    fn render(&self) -> String {
        let mut extra = String::new();
        for (name, value) in &self.headers {
            write!(extra, "{name}: {value}\r\n").unwrap();
        }

        format!(
            "HTTP/1.1 {} Mock\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n{extra}\r\n{}",
            self.status,
            self.body.len(),
            self.body
        )
    }
}

async fn read_head(socket: &mut TcpStream) -> Option<String> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 1024];

    loop {
        let n = match socket.read(&mut chunk).await {
            Ok(0) | Err(_) => return None,
            Ok(n) => n,
        };
        buf.extend_from_slice(chunk.get(..n)?);

        if buf.windows(4).any(|window| window == b"\r\n\r\n") {
            return Some(String::from_utf8_lossy(&buf).into_owned());
        }
    }
}

async fn spawn_mock(reply: Reply) -> (String, Requests) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let requests = Requests::default();
    let recorder = Arc::clone(&requests);
    let response = reply.render();

    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };

            if let Some(head) = read_head(&mut socket).await {
                recorder.lock().unwrap().push(head);
            }

            let _ = socket.write_all(response.as_bytes()).await;
            let _ = socket.shutdown().await;
        }
    });

    (base, requests)
}

fn client(base: &str, key: Option<&str>) -> DeadlockClient {
    let builder = DeadlockClient::builder().base_url(base);
    let builder = match key {
        Some(key) => builder.api_key(key),
        None => builder,
    };

    builder.build().unwrap()
}

fn count(requests: &Requests) -> usize {
    requests.lock().unwrap().len()
}

fn first_head(requests: &Requests) -> String {
    requests.lock().unwrap().first().cloned().unwrap()
}

fn busy_reply() -> Reply {
    Reply { status: 503, headers: vec![("retry-after", "7")], body: "" }
}

#[tokio::test]
async fn the_api_key_is_sent_only_when_configured() {
    let (keyed_base, keyed) = spawn_mock(Reply::ok(RANKS)).await;
    client(&keyed_base, Some("secret-key"))
        .ranks(Priority::Interactive)
        .await
        .unwrap();

    assert!(first_head(&keyed).to_lowercase().contains("x-api-key: secret-key"));

    let (open_base, open) = spawn_mock(Reply::ok(RANKS)).await;
    client(&open_base, None).ranks(Priority::Interactive).await.unwrap();

    assert!(!first_head(&open).to_lowercase().contains("x-api-key"));
}

#[tokio::test]
async fn a_busy_api_blocks_the_bucket_for_the_retry_after_delay() {
    let (base, requests) = spawn_mock(busy_reply()).await;
    let client = client(&base, None);

    let err = client.ranks(Priority::Interactive).await.unwrap_err();

    assert!(matches!(err, DeadlockApiError::Busy { .. }), "{err:?}");
    assert_eq!(err.retry_after(), Some(Duration::from_secs(7)));
    assert_eq!(count(&requests), 1);

    let err = client.heroes(None, Priority::Background).await.unwrap_err();

    let retry_after = err.retry_after().unwrap();
    assert!(retry_after > Duration::from_secs(5), "{retry_after:?}");
    assert!(retry_after <= Duration::from_secs(7), "{retry_after:?}");
    assert_eq!(count(&requests), 1);
}

#[tokio::test]
async fn no_matches_is_an_empty_list() {
    let (base, requests) =
        spawn_mock(Reply { status: 404, headers: Vec::new(), body: "" }).await;
    let since = jiff::Timestamp::from_second(1_760_000_000).unwrap();

    let matches = client(&base, None)
        .recent_matches(&[1, 2], since, Priority::Background)
        .await
        .unwrap();

    assert!(matches.is_empty());
    assert_eq!(count(&requests), 1);
}

#[tokio::test]
async fn recent_matches_decode_and_send_comma_joined_accounts() {
    let (base, requests) = spawn_mock(Reply::ok(METADATA_BULK)).await;
    let since = jiff::Timestamp::from_second(1_760_000_000).unwrap();

    let matches = client(&base, None)
        .recent_matches(&[1, 2], since, Priority::Background)
        .await
        .unwrap();

    assert_eq!(matches.len(), 1);
    let head = first_head(&requests).replace("%2C", ",");
    assert!(head.contains("account_ids=1,2"), "{head}");
    assert!(head.contains("min_unix_timestamp=1760000000"), "{head}");
}

#[tokio::test]
async fn empty_account_lists_make_no_request() {
    let (base, requests) = spawn_mock(Reply::ok("[]")).await;
    let client = client(&base, None);
    let since = jiff::Timestamp::from_second(1_760_000_000).unwrap();

    assert!(
        client
            .recent_matches(&[], since, Priority::Background)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client.player_ranks(&[], Priority::Background).await.unwrap().is_empty()
    );
    assert_eq!(count(&requests), 0);
}

#[tokio::test]
async fn player_ranks_are_requested_in_chunks_of_a_thousand() {
    let (base, requests) = spawn_mock(Reply::ok(RANK_BATCH)).await;
    let ids: Vec<u32> = (1..=1001).collect();

    let ranks =
        client(&base, None).player_ranks(&ids, Priority::Background).await.unwrap();

    assert_eq!(ranks.len(), 4);
    assert_eq!(count(&requests), 2);
}

#[tokio::test]
async fn get_raw_returns_the_body_verbatim() {
    let body = "{\"a\": 1,  \"b\":[2]}";
    let (base, requests) = spawn_mock(Reply::ok(body)).await;

    let got = client(&base, None)
        .get_raw(
            "/v1/assets/heroes?client_version=6123",
            Bucket::Assets,
            Priority::Interactive,
        )
        .await
        .unwrap();

    assert_eq!(got, body);
    let head = first_head(&requests);
    assert!(
        head.starts_with("GET /v1/assets/heroes?client_version=6123 HTTP/1.1"),
        "{head}"
    );
}

#[tokio::test]
async fn get_raw_maps_not_found() {
    let (base, _) =
        spawn_mock(Reply { status: 404, headers: Vec::new(), body: "" }).await;

    let err = client(&base, None)
        .get_raw("/v1/x", Bucket::Assets, Priority::Interactive)
        .await
        .unwrap_err();

    assert!(matches!(err, DeadlockApiError::NotFound), "{err:?}");
}

#[tokio::test]
async fn get_raw_maps_server_errors() {
    for status in [500, 400] {
        let (base, _) =
            spawn_mock(Reply { status, headers: Vec::new(), body: "" }).await;

        let err = client(&base, None)
            .get_raw("/v1/x", Bucket::Assets, Priority::Interactive)
            .await
            .unwrap_err();

        assert!(
            matches!(&err, DeadlockApiError::Http { status: s } if s.as_u16() == status),
            "{err:?}"
        );
    }
}

#[tokio::test]
async fn get_raw_busy_blocks_the_given_bucket() {
    let (base, requests) = spawn_mock(busy_reply()).await;
    let cfg =
        BucketConfig { capacity: 100, per: Duration::from_secs(1), reserved: 0 };
    let limiter = Limiter::with_configs(
        [(Bucket::Assets, cfg), (Bucket::Analytics, cfg)],
        Duration::ZERO,
    );
    let client =
        DeadlockClient::builder().base_url(&base).limiter(limiter).build().unwrap();

    let err = client
        .get_raw("/v1/x", Bucket::Assets, Priority::Interactive)
        .await
        .unwrap_err();

    assert!(matches!(err, DeadlockApiError::Busy { .. }), "{err:?}");
    assert_eq!(err.retry_after(), Some(Duration::from_secs(7)));
    assert_eq!(count(&requests), 1);

    let err = client
        .get_raw("/v1/x", Bucket::Assets, Priority::Interactive)
        .await
        .unwrap_err();

    assert!(matches!(err, DeadlockApiError::Busy { .. }), "{err:?}");
    assert_eq!(count(&requests), 1);

    let _ = client.get_raw("/v1/x", Bucket::Analytics, Priority::Interactive).await;

    assert_eq!(count(&requests), 2);
}

#[tokio::test]
async fn get_raw_rejects_a_relative_path() {
    let (base, requests) = spawn_mock(Reply::ok("{}")).await;
    let client = client(&base, None);

    for path in ["v1/assets/heroes", ".evil.example/x"] {
        let err = client
            .get_raw(path, Bucket::Assets, Priority::Interactive)
            .await
            .unwrap_err();

        assert!(matches!(err, DeadlockApiError::InvalidPath { .. }), "{err:?}");
    }

    assert_eq!(count(&requests), 0);
}

#[tokio::test]
async fn get_raw_sends_the_api_key() {
    let (base, requests) = spawn_mock(Reply::ok("{}")).await;

    client(&base, Some("k"))
        .get_raw("/v1/x", Bucket::Assets, Priority::Interactive)
        .await
        .unwrap();

    assert!(first_head(&requests).to_lowercase().contains("x-api-key: k"));
}

#[tokio::test]
async fn a_base_path_prefix_is_kept() {
    let (base, requests) = spawn_mock(Reply::ok(RANKS)).await;
    let client = client(&format!("{base}/prefix/"), None);

    client
        .get_raw(
            "/v1/assets/heroes?client_version=1",
            Bucket::Assets,
            Priority::Interactive,
        )
        .await
        .unwrap();
    let head = first_head(&requests);
    assert!(
        head.starts_with("GET /prefix/v1/assets/heroes?client_version=1 HTTP/1.1"),
        "{head}"
    );

    client.ranks(Priority::Interactive).await.unwrap();
    let head = requests.lock().unwrap().get(1).cloned().unwrap();
    assert!(head.starts_with("GET /prefix/v1/assets/ranks HTTP/1.1"), "{head}");
}

#[test]
fn an_invalid_base_url_is_rejected() {
    let err = DeadlockClient::builder().base_url("not a url").build().unwrap_err();
    assert!(
        matches!(err, DeadlockApiError::InvalidBaseUrl { source: Some(_), .. }),
        "{err:?}"
    );

    for url in ["mailto:a@b.c", "ftp://example.com", "https://example.com/?x=1"] {
        let err = DeadlockClient::builder().base_url(url).build().unwrap_err();
        assert!(
            matches!(err, DeadlockApiError::InvalidBaseUrl { source: None, .. }),
            "{url}: {err:?}"
        );
    }
}

#[tokio::test]
async fn get_raw_drops_an_empty_query() {
    let (base, requests) = spawn_mock(Reply::ok(RANKS)).await;

    client(&base, None)
        .get_raw("/v1/assets/ranks?", Bucket::Assets, Priority::Interactive)
        .await
        .unwrap();

    let head = first_head(&requests);
    assert!(head.starts_with("GET /v1/assets/ranks HTTP/1.1"), "{head}");
}
