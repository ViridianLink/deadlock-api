use crate::DeadlockClient;
use crate::client::ApiRequest;
use crate::error::Result;
use crate::limiter::{Bucket, Priority};
use crate::model::{Hero, Item, PatchNote, Rank};

fn version_query(version: Option<u32>) -> Vec<(&'static str, String)> {
    version
        .map(|version| ("client_version", version.to_string()))
        .into_iter()
        .collect()
}

impl DeadlockClient {
    pub async fn client_versions(&self, priority: Priority) -> Result<Vec<u32>> {
        let req = ApiRequest {
            path: "/v1/assets/client-versions",
            query: Vec::new(),
            bucket: Bucket::Assets,
        };

        self.get(&req, priority).await
    }

    pub async fn heroes(
        &self,
        client_version: Option<u32>,
        priority: Priority,
    ) -> Result<Vec<Hero>> {
        let req = ApiRequest {
            path: "/v1/assets/heroes",
            query: version_query(client_version),
            bucket: Bucket::Assets,
        };

        self.get(&req, priority).await
    }

    pub async fn items(
        &self,
        client_version: Option<u32>,
        priority: Priority,
    ) -> Result<Vec<Item>> {
        let req = ApiRequest {
            path: "/v1/assets/items",
            query: version_query(client_version),
            bucket: Bucket::Assets,
        };

        self.get(&req, priority).await
    }

    pub async fn ranks(&self, priority: Priority) -> Result<Vec<Rank>> {
        let req = ApiRequest {
            path: "/v1/assets/ranks",
            query: Vec::new(),
            bucket: Bucket::Assets,
        };

        self.get(&req, priority).await
    }

    pub async fn patch_notes(&self, priority: Priority) -> Result<Vec<PatchNote>> {
        let req = ApiRequest {
            path: "/v2/patches",
            query: Vec::new(),
            bucket: Bucket::Assets,
        };

        self.get(&req, priority).await
    }
}
