use serde::{Serialize, Deserialize};
use validator::{Validate};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetOwnersOutput {
    pub users: Vec<GetOwnersOutputUser>,
}

pub type GetOwnersOutputLogin = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetOwnersOutputUser {
    pub id: u32,
    pub login: GetOwnersOutputLogin,
    pub name: Option<String>,
}

pub type AddOwnerInputLogin = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Validate)]
pub struct AddOwnerInput {
    pub users: Vec<AddOwnerInputLogin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AddOwnerOutput {
    pub ok: bool,
    pub msg: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Validate)]
pub struct RemoveOwnerInput {
    pub users: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RemoveOwnerOutput {
    pub ok: bool,
}