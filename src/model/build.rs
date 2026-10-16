use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    pub hero_build: HeroBuild,
    pub num_favorites: Option<u64>,
    pub num_weekly_favorites: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeroBuild {
    pub hero_id: u32,
    pub hero_build_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub version: u32,
    pub last_updated_timestamp: i64,
    pub details: BuildDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildDetails {
    #[serde(default)]
    pub mod_categories: Vec<ModCategory>,
    pub ability_order: Option<BuildAbilityOrder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModCategory {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub mods: Vec<BuildMod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMod {
    pub ability_id: u32,
    pub annotation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildAbilityOrder {
    #[serde(default)]
    pub currency_changes: Vec<CurrencyChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyChange {
    pub ability_id: u32,
    pub currency_type: u8,
    pub delta: i32,
}
