use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::value::StatValue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hero {
    pub id: u32,
    pub class_name: String,
    pub name: String,
    #[serde(default)]
    pub player_selectable: bool,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub in_development: bool,
    #[serde(default)]
    pub images: HeroImages,
    #[serde(default)]
    pub starting_stats: BTreeMap<String, HeroStat>,
    #[serde(default)]
    pub items: BTreeMap<String, String>,
    #[serde(default)]
    pub standard_level_up_upgrades: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeroImages {
    pub icon_image_small: Option<String>,
    pub icon_hero_card: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeroStat {
    pub value: StatValue,
    #[serde(default)]
    pub display_stat_name: Option<String>,
}
