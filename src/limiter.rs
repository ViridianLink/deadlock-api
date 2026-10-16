use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use tokio::time::{Instant, sleep};

use crate::error::{DeadlockApiError, Result};

const MAX_WAIT: Duration = Duration::from_secs(5);
const EPSILON: f64 = 1e-9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bucket {
    Analytics,
    Ranks,
    BulkMetadata,
    SingleMetadata,
    Assets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Interactive,
    Background,
}

#[derive(Debug, Clone, Copy)]
pub struct BucketConfig {
    pub capacity: u32,
    pub per: Duration,
    pub reserved: u32,
}

impl BucketConfig {
    fn rate(self) -> f64 {
        f64::from(self.capacity) / self.per.as_secs_f64()
    }

    fn wait_for(self, tokens: f64, needed: f64) -> Duration {
        Duration::try_from_secs_f64((needed - tokens) / self.rate())
            .unwrap_or(Duration::MAX)
    }
}

#[derive(Debug)]
struct State {
    tokens: f64,
    refilled_at: Instant,
    blocked_until: Option<Instant>,
}

enum Decision {
    Granted,
    Wait(Duration),
    Busy(Duration),
}

#[derive(Debug)]
pub struct Limiter {
    configs: HashMap<Bucket, BucketConfig>,
    max_wait: Duration,
    states: Mutex<HashMap<Bucket, State>>,
}

impl Limiter {
    #[must_use]
    pub fn new(has_api_key: bool) -> Self {
        let analytics = if has_api_key {
            BucketConfig {
                capacity: 300,
                per: Duration::from_mins(1),
                reserved: 120,
            }
        } else {
            BucketConfig { capacity: 150, per: Duration::from_mins(1), reserved: 60 }
        };

        Self::with_configs(
            [
                (Bucket::Analytics, analytics),
                (Bucket::Ranks, BucketConfig {
                    capacity: 10,
                    per: Duration::from_mins(1),
                    reserved: 0,
                }),
                (Bucket::BulkMetadata, BucketConfig {
                    capacity: 15,
                    per: Duration::from_mins(1),
                    reserved: 0,
                }),
                (Bucket::SingleMetadata, BucketConfig {
                    capacity: 2,
                    per: Duration::from_hours(1),
                    reserved: 0,
                }),
                (Bucket::Assets, BucketConfig {
                    capacity: 5,
                    per: Duration::from_secs(1),
                    reserved: 0,
                }),
            ],
            MAX_WAIT,
        )
    }

    #[must_use]
    pub fn with_configs(
        configs: impl IntoIterator<Item = (Bucket, BucketConfig)>,
        max_wait: Duration,
    ) -> Self {
        Self {
            configs: configs.into_iter().collect(),
            max_wait,
            states: Mutex::new(HashMap::new()),
        }
    }

    pub async fn acquire(&self, bucket: Bucket, priority: Priority) -> Result<()> {
        let Some(config) = self.configs.get(&bucket).copied() else {
            return Ok(());
        };

        let started = Instant::now();

        loop {
            match self.decide(bucket, config, priority, started) {
                Decision::Granted => return Ok(()),
                Decision::Wait(duration) => sleep(duration).await,
                Decision::Busy(retry_after) => {
                    return Err(DeadlockApiError::Busy { retry_after });
                },
            }
        }
    }

    pub fn block(&self, bucket: Bucket, duration: Duration) {
        let Some(config) = self.configs.get(&bucket).copied() else {
            return;
        };

        let now = Instant::now();
        let Some(until) = now.checked_add(duration) else {
            return;
        };

        self.with_state(bucket, config, now, |state| {
            let later =
                state.blocked_until.map_or(until, |current| current.max(until));
            state.blocked_until = Some(later);
        });
    }

    fn with_state<R>(
        &self,
        bucket: Bucket,
        config: BucketConfig,
        now: Instant,
        f: impl FnOnce(&mut State) -> R,
    ) -> R {
        let mut states = self.states.lock().unwrap_or_else(PoisonError::into_inner);
        f(state_for(&mut states, bucket, config, now))
    }

    fn decide(
        &self,
        bucket: Bucket,
        config: BucketConfig,
        priority: Priority,
        started: Instant,
    ) -> Decision {
        let now = Instant::now();
        let spent = now.saturating_duration_since(started);
        let budget = self.max_wait.saturating_sub(spent);

        self.with_state(bucket, config, now, |state| {
            decide_state(state, config, priority, budget, now)
        })
    }
}

fn decide_state(
    state: &mut State,
    config: BucketConfig,
    priority: Priority,
    budget: Duration,
    now: Instant,
) -> Decision {
    let elapsed = now.saturating_duration_since(state.refilled_at);
    state.tokens = elapsed
        .as_secs_f64()
        .mul_add(config.rate(), state.tokens)
        .min(f64::from(config.capacity));
    state.refilled_at = now;

    if let Some(until) = state.blocked_until {
        let remaining = until.saturating_duration_since(now);
        if remaining.is_zero() {
            state.blocked_until = None;
        } else {
            return match priority {
                Priority::Interactive if remaining <= budget => {
                    Decision::Wait(remaining)
                },
                Priority::Interactive | Priority::Background => {
                    Decision::Busy(remaining)
                },
            };
        }
    }

    let needed = match priority {
        Priority::Interactive => 1.0,
        Priority::Background => 1.0 + f64::from(config.reserved),
    };

    if state.tokens + EPSILON >= needed {
        state.tokens -= 1.0;
        return Decision::Granted;
    }

    let wait = config.wait_for(state.tokens, needed);
    match priority {
        Priority::Interactive if wait <= budget => Decision::Wait(wait),
        Priority::Interactive | Priority::Background => Decision::Busy(wait),
    }
}

fn state_for(
    states: &mut HashMap<Bucket, State>,
    bucket: Bucket,
    config: BucketConfig,
    now: Instant,
) -> &mut State {
    states.entry(bucket).or_insert_with(|| State {
        tokens: f64::from(config.capacity),
        refilled_at: now,
        blocked_until: None,
    })
}
