use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::value::StatValue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub class_name: String,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type")]
    pub kind: ItemKind,
    pub item_slot_type: Option<String>,
    pub item_tier: Option<u8>,
    pub cost: Option<u32>,
    #[serde(default)]
    pub shopable: bool,
    pub image: Option<String>,
    #[serde(default)]
    pub hero: Option<u32>,
    #[serde(default)]
    pub heroes: Vec<u32>,
    #[serde(default)]
    pub ability_type: Option<String>,
    #[serde(default)]
    pub properties: BTreeMap<String, ItemProperty>,
    #[serde(default)]
    pub upgrades: Vec<AbilityUpgrade>,
    #[serde(default)]
    pub weapon_info: WeaponInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Ability,
    Upgrade,
    Weapon,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemProperty {
    #[serde(default)]
    pub value: Option<StatValue>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub postfix: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AbilityUpgrade {
    #[serde(default)]
    pub property_upgrades: Vec<PropertyUpgrade>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyUpgrade {
    pub name: String,
    #[serde(default)]
    pub bonus: Option<StatValue>,
}

/// Curated numeric weapon stats; every other `weapon_info` key is ignored.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WeaponInfo {
    #[serde(default)]
    pub bullet_damage: Option<f64>,
    #[serde(default)]
    pub bullets: Option<f64>,
    #[serde(default)]
    pub clip_size: Option<f64>,
    #[serde(default)]
    pub cycle_time: Option<f64>,
    #[serde(default)]
    pub reload_duration: Option<f64>,
    #[serde(default)]
    pub bullet_speed: Option<f64>,
    #[serde(default)]
    pub crit_bonus_start: Option<f64>,
    #[serde(default)]
    pub damage_falloff_start_range: Option<f64>,
    #[serde(default)]
    pub damage_falloff_end_range: Option<f64>,
}
