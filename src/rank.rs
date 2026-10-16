use crate::model::Rank;

pub const MAX_TIER: u8 = 11;
pub const MAX_SUBRANK: u8 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Badge(u16);

impl Badge {
    pub const UNRANKED: Self = Self(0);

    #[must_use]
    pub fn new(tier: u8, subrank: u8) -> Option<Self> {
        if tier > MAX_TIER || !(1..=MAX_SUBRANK).contains(&subrank) {
            return None;
        }

        Some(Self(u16::from(tier) * 10 + u16::from(subrank)))
    }

    #[must_use]
    pub fn from_raw(raw: u16) -> Option<Self> {
        if raw == 0 {
            return Some(Self::UNRANKED);
        }

        let tier = u8::try_from(raw / 10).ok()?;
        let subrank = u8::try_from(raw % 10).ok()?;
        Self::new(tier, subrank)
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    #[must_use]
    pub fn tier(self) -> u8 {
        u8::try_from(self.0 / 10).unwrap_or_default()
    }

    #[must_use]
    pub fn subrank(self) -> u8 {
        u8::try_from(self.0 % 10).unwrap_or_default()
    }

    #[must_use]
    pub const fn is_ranked(self) -> bool {
        self.0 != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BadgeRange {
    pub min: u16,
    pub max: u16,
}

impl BadgeRange {
    #[must_use]
    pub fn tier(tier: u8) -> Option<Self> {
        if tier > MAX_TIER {
            return None;
        }

        let min = u16::from(tier) * 10;
        Some(Self { min, max: min + u16::from(MAX_SUBRANK) })
    }
}

#[must_use]
pub fn badge_label(ranks: &[Rank], badge: Badge) -> String {
    if !badge.is_ranked() {
        return "Unranked".to_owned();
    }

    let tier = badge.tier();
    let subrank = badge.subrank();

    ranks.iter().find(|rank| rank.tier == tier).map_or_else(
        || format!("Tier {tier} {subrank}"),
        |rank| format!("{} {subrank}", rank.name),
    )
}
