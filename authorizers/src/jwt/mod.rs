use crate::result::AuthResult;
use crate::error::AuthError;
use serde::de::DeserializeOwned;
use double_checked_cell_async::DoubleCheckedCell;

pub mod types;
pub mod config;
pub mod keys;

pub use types::*;
pub use config::*;
pub use keys::*;

pub struct TokenDecoder {
    pub config_uri: String,
    pub aud: String,
    config_cell: DoubleCheckedCell<OpenIdConfiguration>,
    key_set_cell: DoubleCheckedCell<KeySet>,
}

impl TokenDecoder {
    pub fn new(uri: &str, aud: &str) -> Self {
        TokenDecoder {
            config_uri: uri.to_owned(),
            aud: aud.to_owned(),
            config_cell: DoubleCheckedCell::new(),
            key_set_cell: DoubleCheckedCell::new(),
        }
    }

    pub async fn config<'a>(&'a self) -> AuthResult<&'a OpenIdConfiguration> {
        self.config_cell.get_or_try_init(async {
            fetch_config(&self.config_uri).await
        }).await
    }

    pub async fn key_set<'a>(&'a self) -> AuthResult<&'a KeySet> {
        self.key_set_cell.get_or_try_init(async {
            let config = self.config().await?;

            if let Some(ref jwks_uri) = config.jwks_uri {
                fetch_key_set(jwks_uri).await
            } else {
                Err(AuthError::JwtError("no jwks uri configured".to_string()))
            }
        }).await
    }

    pub async fn decode(&self, token: &str) -> AuthResult<IdToken> {
        let config = self.config().await?;
        let key_set = self.key_set().await?;

        decode_and_validate_jwt(token, &key_set, &self.aud, &config.issuer)
    }
}

pub fn decode_and_validate_jwt<T: DeserializeOwned>(token: &str, key_set: &KeySet, aud: &str, iss: &str) -> AuthResult<T> {
    let header = jsonwebtoken::decode_header(token)?;
    if let Some(ref kid) = header.kid {
        if let Some(key) = key_set.find_key(kid) {
            let algorithm = algorithm_from_key(key)?;
            let mut validation = jsonwebtoken::Validation::new(algorithm);
            validation.set_audience(&[aud]);
            validation.iss = Some(iss.to_owned());

            decode_and_validate_jwt_with_key(token, key, &validation)
        } else {
            return Err(AuthError::JwtError("unknown jwt kid".to_string()))
        }
    } else {
        return Err(AuthError::JwtError("missing jwt kid".to_string()))
    }
}

fn decode_and_validate_jwt_with_key<T: DeserializeOwned>(token: &str, key: &Key, validation: &jsonwebtoken::Validation) -> AuthResult<T> {
    let decoding_key = decoding_key_from_key(key)?;
    
    let token_data = jsonwebtoken::decode(token, &decoding_key, validation)?;

    Ok(token_data.claims)
}

fn decoding_key_from_key(key: &Key) -> AuthResult<jsonwebtoken::DecodingKey> {
    match key {
        Key::RSA(k) => {
            Ok(jsonwebtoken::DecodingKey::from_rsa_components(&k.n, &k.e))
        }
    }
}

fn algorithm_from_key(key: &Key) -> AuthResult<jsonwebtoken::Algorithm> {
    match key {
        Key::RSA(k) => {
            match &k.alg[..] {
                "RS256" => {
                    Ok(jsonwebtoken::Algorithm::RS256)
                },
                other => {
                    Err(AuthError::JwtError(format!("unsupported algorithm {}", other)))
                }
            }
        }
    }
}