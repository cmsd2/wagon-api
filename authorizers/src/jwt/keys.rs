use crate::result::AuthResult;
use super::types::KeySet;

pub async fn fetch_key_set(uri: &str) -> AuthResult<KeySet> {
    let resp = reqwest::get(uri)
        .await?
        .json::<KeySet>()
        .await?;
    Ok(resp)
}
