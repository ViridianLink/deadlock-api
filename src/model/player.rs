use serde::{Deserialize, Serialize};

use crate::rank::Badge;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRank {
    pub account_id: u32,
    pub badge: u16,
    pub rank: u8,
    pub subrank: u8,
    pub last_match: Option<RankedMatch>,
}

impl AccountRank {
    #[must_use]
    pub fn badge(&self) -> Badge {
        Badge::from_raw(self.badge).unwrap_or(Badge::UNRANKED)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedMatch {
    pub match_id: u64,
    pub start_time: i64,
}
