use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchCrateOutput {
    pub crates: Vec<SearchCrateOutputItem>,
    pub meta: SearchCrateOutputMeta,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchCrateOutputItem {
    pub name: String,
    pub max_version: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchCrateOutputMeta {
    pub total: u32,
}