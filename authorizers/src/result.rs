use crate::error::AuthError;

pub type AuthResult<T> = std::result::Result<T, AuthError>;
