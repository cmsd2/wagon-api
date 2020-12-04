use lazy_static::lazy_static;
use maplit::hashmap;
use std::env;
use aws_lambda_events::event::apigw;
use crate::result::AuthResult;
use crate::error::AuthError;
use rusoto_core::Region;
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, QueryInput};

lazy_static! {
    static ref DYNAMODB_CLIENT: DynamoDbClient = DynamoDbClient::new(Region::default());
    static ref TOKENS_TABLE: String = env::var("TOKENS_TABLE").unwrap();
    static ref TOKENS_TABLE_TOKENS_INDEX: String = env::var("TOKENS_TABLE_TOKENS_INDEX").unwrap();
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthorizationHeader {
    NotPresent,
    Empty,
    BearerToken(String),
    ApiKey(String),
}

impl AuthorizationHeader {
    pub fn from_request(req: &apigw::ApiGatewayCustomAuthorizerRequest) -> Self {
        req
            .authorization_token
            .as_ref()
            .map(|s| &s[..])
            .map(Self::from_value)
            .unwrap_or(AuthorizationHeader::NotPresent)
    }

    pub fn from_value(value: &str) -> Self {
        if value == "" {
            return AuthorizationHeader::Empty;
        }

        let bearer_prefix = "Bearer ";
        if value.starts_with(bearer_prefix) {
            AuthorizationHeader::BearerToken(value[bearer_prefix.len()..].to_owned())
        } else {
            AuthorizationHeader::ApiKey(value.to_owned())
        }
    }
}

pub async fn lookup_token(token: &str) -> AuthResult<Option<String>> {
    let results = DYNAMODB_CLIENT.query(QueryInput {
        key_condition_expression: Some("token = :token".to_string()),
        expression_attribute_values: Some(hashmap! {
            ":token".to_string() => AttributeValue { s: Some(token.to_owned()), ..Default::default() }
        }),
        table_name: TOKENS_TABLE.clone(),
        index_name: Some(TOKENS_TABLE_TOKENS_INDEX.clone()),
        ..Default::default()
    }).await
        .map_err(|err| {
            log::info!("error querying tokens index: {:?}", err);

            AuthError::DatabaseError(format!("error querying database for api key"))
        })?;

    Ok(results
        .items
        .and_then(|items| items.into_iter().next())
        .and_then(|attrs| attrs.get("user_id").map(|v| v.to_owned()))
        .and_then(|attr_value| attr_value.s))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_auth_bearer_token_from_value() {
        let value = "Bearer foo";
        let expected = AuthorizationHeader::BearerToken("foo".to_owned());
        assert_eq!(AuthorizationHeader::from_value(value), expected);
    }

    #[test]
    fn test_auth_api_key_from_value() {
        let value = "api key";
        let expected = AuthorizationHeader::ApiKey("api key".to_owned());
        assert_eq!(AuthorizationHeader::from_value(value), expected);
    }

    #[test]
    fn test_auth_bearer_token_from_req() {
        let req = apigw::ApiGatewayCustomAuthorizerRequest {
            authorization_token: Some("Bearer foo".to_string()),
            method_arn: Some("arn".to_string()),
            type_: Some("TOKEN".to_string()),
        };
        let expected = AuthorizationHeader::BearerToken("foo".to_owned());
        assert_eq!(AuthorizationHeader::from_request(&req), expected);
    }
}