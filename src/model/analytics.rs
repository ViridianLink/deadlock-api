use serde::{Deserialize, Serialize};

#[expect(
    clippy::cast_precision_loss,
    reason = "match counts stay far below 2^53, so the f64 conversion is exact"
)]
fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        return 0.0;
    }

    numerator as f64 / denominator as f64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemFlowStats {
    pub nodes: Vec<ItemFlowNode>,
    #[serde(default)]
    pub edges: Vec<ItemFlowEdge>,
    pub summary: FlowSummary,
    #[serde(default)]
    pub reached_per_column: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemFlowNode {
    pub column: u8,
    pub item_id: u32,
    pub wins: u64,
    pub losses: u64,
    pub matches: u64,
    pub adjusted_win_rate: Option<f64>,
    pub avg_net_worth_at_buy: Option<f64>,
}

impl ItemFlowNode {
    #[must_use]
    pub fn win_rate(&self) -> f64 {
        ratio(self.wins, self.matches)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemFlowEdge {
    pub from_column: u8,
    pub from_item_id: u32,
    pub to_item_id: u32,
    pub wins: u64,
    pub losses: u64,
    pub matches: u64,
}

impl ItemFlowEdge {
    #[must_use]
    pub fn win_rate(&self) -> f64 {
        ratio(self.wins, self.matches)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSummary {
    pub wins: u64,
    pub losses: u64,
    pub matches: u64,
}

impl FlowSummary {
    #[must_use]
    pub fn win_rate(&self) -> f64 {
        ratio(self.wins, self.matches)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityOrderStats {
    pub abilities: Vec<u32>,
    #[serde(default)]
    pub avg_upgrade_times_s: Vec<f64>,
    pub wins: u64,
    pub losses: u64,
    pub matches: u64,
}

impl AbilityOrderStats {
    #[must_use]
    pub fn win_rate(&self) -> f64 {
        ratio(self.wins, self.matches)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeroCounterStats {
    pub hero_id: u32,
    pub enemy_hero_id: u32,
    pub wins: u64,
    pub matches_played: u64,
    #[serde(default)]
    pub kills: u64,
    #[serde(default)]
    pub deaths: u64,
    #[serde(default)]
    pub enemy_kills: u64,
    #[serde(default)]
    pub enemy_deaths: u64,
}

impl HeroCounterStats {
    #[must_use]
    pub fn win_rate(&self) -> f64 {
        ratio(self.wins, self.matches_played)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneMatchupStats {
    pub assigned_lane: u8,
    pub hero_ids: Vec<u32>,
    pub enemy_hero_ids: Vec<u32>,
    pub wins: u64,
    pub matches_played: u64,
    pub sample_time_s: u32,
    pub net_worth_diff: f64,
    pub sample_matches: u64,
}

impl LaneMatchupStats {
    #[must_use]
    pub fn win_rate(&self) -> f64 {
        ratio(self.wins, self.matches_played)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneSoulCurve {
    pub assigned_lane: u8,
    pub hero_ids: Vec<u32>,
    pub enemy_hero_ids: Vec<u32>,
    pub sample_times_s: Vec<u32>,
    pub sample_matches: Vec<u64>,
    pub matches_played: u64,
    pub net_worth_diff: Vec<f64>,
    #[serde(default)]
    pub net_worth_diff_std: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStats {
    pub item_id: u32,
    pub wins: u64,
    pub losses: u64,
    pub matches: u64,
    #[serde(default)]
    pub avg_buy_time_s: Option<f64>,
    #[serde(default)]
    pub avg_sell_time_s: Option<f64>,
}

impl ItemStats {
    #[must_use]
    pub fn win_rate(&self) -> f64 {
        ratio(self.wins, self.matches)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePoint {
    pub game_time: u32,
    pub net_worth_avg: f64,
    #[serde(default)]
    pub net_worth_std: f64,
    #[serde(default)]
    pub kills_avg: f64,
    #[serde(default)]
    pub deaths_avg: f64,
    #[serde(default)]
    pub assists_avg: f64,
    #[serde(default)]
    pub creep_kills_avg: f64,
    #[serde(default)]
    pub denies_avg: f64,
}
