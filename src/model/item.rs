use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
