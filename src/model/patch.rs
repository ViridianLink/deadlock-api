use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchNote {
    pub source: String,
    pub title: String,
    pub pub_date: String,
    pub link: String,
    pub guid: PatchGuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchGuid {
    pub text: String,
}
