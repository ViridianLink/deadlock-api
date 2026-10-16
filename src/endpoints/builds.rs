use crate::DeadlockClient;
use crate::client::ApiRequest;
use crate::error::Result;
use crate::limiter::{Bucket, Priority};
use crate::model::Build;

impl DeadlockClient {
    pub async fn builds(
        &self,
        hero_id: u32,
        priority: Priority,
    ) -> Result<Vec<Build>> {
        let req = ApiRequest {
            path: "/v1/builds",
            query: vec![
                ("hero_id", hero_id.to_string()),
                ("only_latest", "true".to_owned()),
                ("sort_by", "weekly_favorites".to_owned()),
                ("limit", "5".to_owned()),
            ],
            bucket: Bucket::Assets,
        };

        self.get(&req, priority).await
    }
}
