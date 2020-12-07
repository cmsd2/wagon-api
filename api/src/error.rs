use std::fmt;
use std::error;

#[derive(Debug, Clone)]
pub enum ApiError {
    NotAuthorized(String),
    Other(String),
    SerializationError(String),
    Database(String),
    InvalidInput(String),
}

impl error::Error for ApiError {}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&str> for ApiError {
    fn from(s: &str) -> Self {
        ApiError::Other(s.to_owned())
    }
}