use aws_lambda_events::event::apigw;
use authorizers::iam::{Method, policy_builder_for_method};
use authorizers::error::AuthError;
use authorizers::result::AuthResult;
use authorizers::token::{AuthorizationHeader, lookup_token};
use authorizers::{authorizer_handler, BoxFuture, Authenticator, Claims};
use env_logger;
use lambda::{lambda, Context};
use authorizers::jwt::TokenDecoder;
use lazy_static::lazy_static;
use std::env;

type LambdaRuntimeError = Box<dyn std::error::Error + Send + Sync + 'static>;
type LambdaRuntimeResult<T> = std::result::Result<T, LambdaRuntimeError>;

lazy_static! {
    static ref TOKEN_DECODER: TokenDecoder = TokenDecoder::new(
        &env::var("OPENID_CONFIGURATION_URI").unwrap(),
        &env::var("OPENID_AUD").unwrap()
    );
}

#[lambda]
#[tokio::main]
async fn main(
    req: apigw::ApiGatewayCustomAuthorizerRequest,
    ctx: Context,
) -> LambdaRuntimeResult<apigw::ApiGatewayCustomAuthorizerResponse<serde_json::Value>> {
    drop(env_logger::try_init());

    Ok(authorizer_handler::<TokenAuthenticator>(req, ctx).await)
}

pub struct TokenAuthenticator;

impl Authenticator for TokenAuthenticator {
    fn authenticate(event: &apigw::ApiGatewayCustomAuthorizerRequest) -> BoxFuture<AuthResult<Claims>> {
        Box::pin(async move {
            match AuthorizationHeader::from_request(event) {
                AuthorizationHeader::BearerToken(token) => {
                    let id_token = TOKEN_DECODER.decode(&token).await?;
                    Ok(Claims {
                        principal_id: id_token.sub,
                        scopes: vec!["user", "api"],
                    })
                },
                AuthorizationHeader::ApiKey(key) => {
                    let principal_id = lookup_token(&key).await?
                        .ok_or_else(|| AuthError::ApiKeyError("api key not found".to_string()))?;
                    Ok(Claims {
                        principal_id: principal_id,
                        scopes: vec!["api"],
                    })
                },
                _other => {
                    Err(AuthError::InputError(format!("bearer token expected")))
                }
            }
        })
    }

    fn authorize<'a>(event: &'a apigw::ApiGatewayCustomAuthorizerRequest, claims: &'a Claims) -> BoxFuture<'a, AuthResult<apigw::ApiGatewayCustomAuthorizerPolicy>> {
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
