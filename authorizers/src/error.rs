use serde_json;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum AuthError {
    SerializationError(serde_json::Error),
    HttpError(reqwest::Error),
    JwtError(String),
}

impl Error for AuthError {
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<serde_json::Error> for AuthError {
    fn from(e: serde_json::Error) -> Self {
        AuthError::SerializationError(e)
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(e: reqwest::Error) -> Self {
        AuthError::HttpError(e)
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        AuthError::JwtError(e.to_string())
    }
}