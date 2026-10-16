use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeroImages {
    pub icon_image_small: Option<String>,
    pub icon_hero_card: Option<String>,
}
