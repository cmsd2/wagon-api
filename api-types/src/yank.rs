use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct YankCrateOutput {
    pub ok: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UnYankCrateOutput {
    pub ok: bool,
}