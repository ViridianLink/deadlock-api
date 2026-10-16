use jiff::Timestamp;

use crate::DeadlockClient;
use crate::client::ApiRequest;
use crate::error::{DeadlockApiError, Result};
use crate::limiter::{Bucket, Priority};
use crate::model::{AccountRank, MatchMetadata};

const MAX_ACCOUNT_IDS: usize = 1000;

fn join_ids(ids: &[u32]) -> String {
    ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
}

impl DeadlockClient {
    pub async fn player_ranks(
        &self,
        account_ids: &[u32],
        priority: Priority,
    ) -> Result<Vec<AccountRank>> {
        let mut ranks = Vec::new();

        for chunk in account_ids.chunks(MAX_ACCOUNT_IDS) {
            let req = ApiRequest {
                path: "/v1/players/rank",
                query: vec![("account_ids", join_ids(chunk))],
                bucket: Bucket::Ranks,
            };

            let batch: Vec<AccountRank> = self.get(&req, priority).await?;
            ranks.extend(batch);
        }

        Ok(ranks)
    }

    pub async fn recent_matches(
        &self,
        account_ids: &[u32],
        since: Timestamp,
        priority: Priority,
    ) -> Result<Vec<MatchMetadata>> {
        let mut matches = Vec::new();

        for chunk in account_ids.chunks(MAX_ACCOUNT_IDS) {
            let req = ApiRequest {
                path: "/v1/matches/metadata",
                query: vec![
                    ("account_ids", join_ids(chunk)),
                    ("min_unix_timestamp", since.as_second().to_string()),
                    ("include_info", "true".to_owned()),
                    ("include_player_info", "true".to_owned()),
                    ("include_player_kda", "true".to_owned()),
                    ("include_player_items", "true".to_owned()),
                    ("include_player_stats", "true".to_owned()),
                    ("only_filtered_players", "true".to_owned()),
                    ("game_mode", "normal".to_owned()),
                    ("order_by", "start_time".to_owned()),
                    ("order_direction", "asc".to_owned()),
                    ("limit", "10000".to_owned()),
                ],
                bucket: Bucket::BulkMetadata,
            };

            let batch: Vec<MatchMetadata> = match self.get(&req, priority).await {
                Ok(batch) => batch,
                Err(DeadlockApiError::NotFound) => continue,
                Err(error) => return Err(error),
            };
            matches.extend(batch);
        }

        Ok(matches)
    }
}
