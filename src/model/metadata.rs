use jiff::tz::TimeZone;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

const START_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchMetadata {
    pub match_id: u64,
    #[serde(deserialize_with = "start_time")]
    pub start_time: jiff::Timestamp,
    pub winning_team: String,
    pub duration_s: u32,
    #[serde(default)]
    pub game_mode: String,
    #[serde(default)]
    pub match_mode: String,
    #[serde(default)]
    pub players: Vec<MatchPlayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchPlayer {
    pub account_id: u32,
    pub hero_id: u32,
    pub team: String,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub net_worth: u32,
    #[serde(default)]
    pub last_hits: u32,
    #[serde(default)]
    pub denies: u32,
    #[serde(default)]
    pub player_level: u32,
    #[serde(default)]
    pub assigned_lane: u8,
    pub player_match_outcome: Option<String>,
    #[serde(default)]
    pub items: Vec<PlayerItem>,
    #[serde(default)]
    pub stats: Vec<StatSnapshot>,
}

impl MatchPlayer {
    #[must_use]
    pub fn won(&self, winning_team: &str) -> bool {
        self.team == winning_team
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerItem {
    pub game_time_s: u32,
    pub item_id: u32,
    #[serde(default)]
    pub upgrade_id: u32,
    #[serde(default)]
    pub sold_time_s: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatSnapshot {
    pub time_stamp_s: u32,
    pub net_worth: u32,
    #[serde(default)]
    pub kills: u32,
    #[serde(default)]
    pub deaths: u32,
    #[serde(default)]
    pub assists: u32,
    #[serde(default)]
    pub ability_points: u32,
    #[serde(default)]
    pub player_damage: u64,
    #[serde(default)]
    pub creep_kills: u32,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawStartTime {
    Unix(i64),
    Text(String),
}

fn parse_text(text: &str) -> Result<jiff::Timestamp, jiff::Error> {
    match jiff::civil::DateTime::strptime(START_TIME_FORMAT, text) {
        Ok(datetime) => Ok(datetime.to_zoned_datetime(TimeZone::UTC)?.timestamp()),
        Err(_) => text.parse(),
    }
}

fn start_time<'de, D>(deserializer: D) -> Result<jiff::Timestamp, D::Error>
where
    D: Deserializer<'de>,
{
    match RawStartTime::deserialize(deserializer)? {
        RawStartTime::Unix(seconds) => {
            jiff::Timestamp::from_second(seconds).map_err(D::Error::custom)
        },
        RawStartTime::Text(text) => parse_text(&text).map_err(D::Error::custom),
    }
}
