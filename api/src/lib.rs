use serde_json;

pub mod error;
pub mod result;
pub mod tokens;

pub struct User {
    pub id: String,
}

pub struct ApiRequestContext {
    pub user: User,
    pub body: serde_json::Value,
}
