use rusoto_core::RusotoError;
use rusoto_dynamodb::GetItemError;
pub enum DbError {
    GetItemError(RusotoError<GetItemError>),
    RegistryNotFound(String),
    InvalidRegistry(String),
}
