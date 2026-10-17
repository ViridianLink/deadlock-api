use crate::DeadlockClient;
use crate::client::ApiRequest;
use crate::error::Result;
use crate::limiter::{Bucket, Priority};
use crate::model::{
    AbilityOrderStats,
    HeroCounterStats,
    ItemFlowStats,
    ItemStats,
    LaneMatchupStats,
    LaneSoulCurve,
    PerformancePoint,
};
use crate::rank::BadgeRange;

fn request(
    path: &'static str,
    mut query: Vec<(&'static str, String)>,
    badge: Option<BadgeRange>,
) -> ApiRequest {
    if let Some(range) = badge {
        query.push(("min_average_badge", range.min.to_string()));
        query.push(("max_average_badge", range.max.to_string()));
    }

    ApiRequest { path, query, bucket: Bucket::Analytics }
}

impl DeadlockClient {
    pub async fn item_flow(
        &self,
        hero_id: u32,
        badge: Option<BadgeRange>,
        priority: Priority,
    ) -> Result<ItemFlowStats> {
        let req = request(
            "/v1/analytics/item-flow-stats",
            vec![("hero_ids", hero_id.to_string())],
            badge,
        );

        self.get(&req, priority).await
    }

    pub async fn ability_orders(
        &self,
        hero_id: u32,
        badge: Option<BadgeRange>,
        priority: Priority,
    ) -> Result<Vec<AbilityOrderStats>> {
        let req = request(
            "/v1/analytics/ability-order-stats",
            vec![("hero_id", hero_id.to_string())],
            badge,
        );

        self.get(&req, priority).await
    }

    pub async fn hero_counters(
        &self,
        badge: Option<BadgeRange>,
        priority: Priority,
    ) -> Result<Vec<HeroCounterStats>> {
        let req = request("/v1/analytics/hero-counter-stats", Vec::new(), badge);

        self.get(&req, priority).await
    }

    pub async fn lane_matchups(
        &self,
        hero_id: u32,
        enemy_hero_id: u32,
        badge: Option<BadgeRange>,
        priority: Priority,
    ) -> Result<Vec<LaneMatchupStats>> {
        let req = request(
            "/v1/analytics/lane-matchup-stats",
            vec![
                ("hero_ids", hero_id.to_string()),
                ("enemy_hero_ids", enemy_hero_id.to_string()),
            ],
            badge,
        );

        self.get(&req, priority).await
    }

    pub async fn lane_soul_curves(
        &self,
        hero_id: u32,
        enemy_hero_id: u32,
        badge: Option<BadgeRange>,
        priority: Priority,
    ) -> Result<Vec<LaneSoulCurve>> {
        let req = request(
            "/v1/analytics/lane-soul-curve",
            vec![
                ("hero_ids", hero_id.to_string()),
                ("enemy_hero_ids", enemy_hero_id.to_string()),
            ],
            badge,
        );

        self.get(&req, priority).await
    }

    pub async fn item_stats(
        &self,
        hero_id: u32,
        badge: Option<BadgeRange>,
        priority: Priority,
    ) -> Result<Vec<ItemStats>> {
        let req = request(
            "/v1/analytics/item-stats",
            vec![("hero_id", hero_id.to_string())],
            badge,
        );

        self.get(&req, priority).await
    }

    pub async fn performance_curve(
        &self,
        hero_id: u32,
        badge: Option<BadgeRange>,
        priority: Priority,
    ) -> Result<Vec<PerformancePoint>> {
        let req = request(
            "/v1/analytics/player-performance-curve",
            vec![("hero_ids", hero_id.to_string()), ("resolution", "0".to_owned())],
            badge,
        );

        self.get(&req, priority).await
    }
}
