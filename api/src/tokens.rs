use crate::result::ApiResult;
use lazy_static::lazy_static;
use std::env;
use maplit::hashmap;
use rusoto_core::Region;
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, GetItemInput, PutItemInput};
use rusoto_kms::{Kms, KmsClient, GenerateRandomRequest};

use crate::error::ApiError;

lazy_static! {
    static ref DYNAMODB_CLIENT: DynamoDbClient = DynamoDbClient::new(Region::default());
    static ref TOKENS_TABLE: String = env::var("TOKENS_TABLE").unwrap();
    static ref KMS_CLIENT: KmsClient = KmsClient::new(Region::default());
}

pub async fn get_user_token(user_id: &str) -> ApiResult<Option<String>> {
    let output = DYNAMODB_CLIENT.get_item(GetItemInput {
        key: hashmap! {
            "user_id".to_string() => AttributeValue { s: Some(user_id.to_owned()), ..Default::default() },
        },
        table_name: TOKENS_TABLE.clone(),
        ..Default::default()
    }).await
        .map_err(|err| ApiError::Database(format!("error fetching token: {:?}", err)))?;

    Ok(output.item.and_then(|mut attrs| attrs.remove("token")).and_then(|token_attr| token_attr.s))
}

pub async fn generate_token() -> ApiResult<String> {
    let output = KMS_CLIENT.generate_random(GenerateRandomRequest {
        number_of_bytes: Some(24),
        ..Default::default()
    }).await
        .map_err(|err| ApiError::Database(format!("error generating token: {:?}", err)))?;
    
    output.plaintext
        .ok_or_else(|| ApiError::Database(format!("error generating token: no token returned")))
        .map(|b| base64::encode(b))
}

pub async fn create_user_token(user_id: &str) -> ApiResult<String> {
    let token = generate_token().await?;

    DYNAMODB_CLIENT.put_item(PutItemInput {
        item: hashmap! {
            "user_id".to_string() => AttributeValue { s: Some(user_id.to_owned()), ..Default::default() },
            "token".to_string() => AttributeValue { s: Some(token.clone()), ..Default::default() },
        },
        table_name: TOKENS_TABLE.to_owned(),
        ..Default::default()
    }).await
        .map_err(|err| ApiError::Database(format!("error saving token: {:?}", err)))?;
    
    Ok(token)
}