use authorizers::error::AuthError;
use authorizers::iam::{policy_builder_for_method, Method};
use authorizers::jwt::TokenDecoder;
use authorizers::result::AuthResult;
use authorizers::token::{lookup_token, AuthorizationHeader};
use authorizers::{authorizer_handler, Authenticator, BoxFuture, Claims};
use aws_lambda_events::apigw;
use env_logger;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::env;

lazy_static! {
    static ref TOKEN_DECODER: TokenDecoder = TokenDecoder::new(
        &env::var("OPENID_CONFIGURATION_URI").unwrap(),
        &env::var("OPENID_AUD").unwrap()
    );
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    drop(env_logger::try_init());

    run(service_fn(function_handler)).await
}

async fn function_handler(
    event: LambdaEvent<apigw::ApiGatewayCustomAuthorizerRequest>,
) -> Result<apigw::ApiGatewayCustomAuthorizerResponse<impl Serialize + DeserializeOwned>, Error> {
    let resp = authorizer_handler::<TokenAuthenticator>(event.payload, event.context).await;

    Ok(resp)
}

pub struct TokenAuthenticator;

impl Authenticator for TokenAuthenticator {
    fn authenticate(
        event: &apigw::ApiGatewayCustomAuthorizerRequest,
    ) -> BoxFuture<AuthResult<Claims>> {
        Box::pin(async move {
            match AuthorizationHeader::from_request(event) {
                AuthorizationHeader::BearerToken(token) => {
                    let id_token = TOKEN_DECODER.decode(&token).await?;
                    Ok(Claims {
                        principal_id: id_token.sub,
                        scopes: vec!["user", "api"],
                    })
                }
                AuthorizationHeader::ApiKey(key) => {
                    let principal_id = lookup_token(&key)
                        .await?
                        .ok_or_else(|| AuthError::ApiKeyError("api key not found".to_string()))?;
                    Ok(Claims {
                        principal_id: principal_id,
                        scopes: vec!["api"],
                    })
                }
                _other => Err(AuthError::InputError(format!("bearer token expected"))),
            }
        })
    }

    fn authorize<'a>(
        event: &'a apigw::ApiGatewayCustomAuthorizerRequest,
        claims: &'a Claims,
    ) -> BoxFuture<'a, AuthResult<apigw::ApiGatewayCustomAuthorizerPolicy>> {
        Box::pin(async move {
            let mut builder = policy_builder_for_method(&event);

            if claims.scopes.contains(&"user") {
                builder = builder.allow_method(Method::All, "/api/token")?;
            }

            if claims.scopes.contains(&"api") {
                builder = builder.allow_method(Method::All, "/api/v1/*")?;
            }

            Ok(builder.build())
        })
    }
}
