use rusoto_core::RusotoError;
use rusoto_dynamodb::{GetItemError, PutItemError, UpdateItemError};
use std::num::ParseIntError;

#[derive(Debug)]
pub enum DbError {
    GetItemError(RusotoError<GetItemError>),
    PutItemError(RusotoError<PutItemError>),
    UpdateItemError(RusotoError<UpdateItemError>),
    RegistryNotFound(String),
    InvalidRegistry(String),
    NoUpdateAttributes,
    InvalidValue(String),
    ParseIntError(ParseIntError),
    ConfigError(String),
}

impl From<ParseIntError> for DbError {
    fn from(e: ParseIntError) -> DbError {
        DbError::ParseIntError(e)
    }
}
