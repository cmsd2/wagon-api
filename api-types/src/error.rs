use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ErrorOutput {
    pub errors: Vec<ErrorItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ErrorItem {
    pub detail: String,
}