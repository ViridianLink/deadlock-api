use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rank {
    pub tier: u8,
    pub name: String,
    pub color: Option<String>,
    #[serde(default)]
    pub images: RankImages,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RankImages {
    pub small: Option<String>,
    pub large: Option<String>,
}
