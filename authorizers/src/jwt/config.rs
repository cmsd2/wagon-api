use crate::result::AuthResult;
use super::types::OpenIdConfiguration;

pub async fn fetch_config(uri: &str) -> AuthResult<OpenIdConfiguration> {
    let resp = reqwest::get(uri)
        .await?
        .json::<OpenIdConfiguration>()
        .await?;
    Ok(resp)
}
