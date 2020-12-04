use std::collections::HashMap;

use serde_json::json;
use aws_lambda_events::event::apigw;
use authorizers::jwt::IdToken;
use authorizers::iam::{ApiGatewayCustomAuthorizerPolicyBuilder, policy_builder_for_method};
use authorizers::error::AuthError;
use authorizers::result::AuthResult;
use authorizers::token::AuthorizationHeader;
use env_logger;
use lambda::{lambda, Context};
use log::{info, debug};
use simple_error::bail;
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

pub struct Claims {
    pub principal_id: String,
}

#[lambda]
#[tokio::main]
async fn main(
    req: apigw::ApiGatewayCustomAuthorizerRequest,
    ctx: Context,
) -> LambdaRuntimeResult<apigw::ApiGatewayCustomAuthorizerResponse<serde_json::Value>> {
    drop(env_logger::try_init());

    Ok(authorizer_handler(req, ctx).await)
}

async fn lookup_api_key(key: &str) -> AuthResult<Claims> {
    todo!()
}

async fn authenticate(event: &apigw::ApiGatewayCustomAuthorizerRequest) -> AuthResult<Claims> {
    match AuthorizationHeader::from_request(event) {
        AuthorizationHeader::BearerToken(token) => {
            let id_token = TOKEN_DECODER.decode(&token).await?;
            Ok(Claims {
                principal_id: id_token.sub
            })
        },
        AuthorizationHeader::ApiKey(key) => {
            lookup_api_key(&key).await
        },
        _other => {
            Err(AuthError::InputError(format!("bearer token expected")))
        }
    }
}

fn authorize(event: &apigw::ApiGatewayCustomAuthorizerRequest) -> AuthResult<apigw::ApiGatewayCustomAuthorizerPolicy> {
    Ok(policy_builder_for_method(&event)
        .allow_all_methods()
        .build())
}

async fn auth(event: &apigw::ApiGatewayCustomAuthorizerRequest) -> AuthResult<(Option<String>, apigw::ApiGatewayCustomAuthorizerPolicy)> {
    let claims = authenticate(&event).await?;
    let policy = authorize(&event)?;
    Ok((Some(claims.principal_id), policy))
}

async fn authorizer_handler(
    event: apigw::ApiGatewayCustomAuthorizerRequest,
    ctx: Context,
) -> apigw::ApiGatewayCustomAuthorizerResponse<serde_json::Value> {
    info!("Client token: {:?}", event.authorization_token);
    info!("Method ARN: {:?}", event.method_arn);

    // validate the incoming token
    // and produce the principal user identifier associated with the token

    // this could be accomplished in a number of ways:
    // 1. Call out to OAuth provider
    // 2. Decode a JWT token inline
    // 3. Lookup in a self-managed DB

    let (principal_id, policy) = auth(&event).await.unwrap_or_else(|err| {
        info!("authentication failure: {:?}", err);

        (None, policy_builder_for_method(&event)
                .deny_all_methods()
                .build())
    });
    

    // you can send a 401 Unauthorized response to the client by failing like so:
    // Err(HandlerError{ msg: "Unauthorized".to_string(), backtrace: None });

    // if the token is valid, a policy must be generated which will allow or deny access to the client

    // if access is denied, the client will recieve a 403 Access Denied response
    // if access is allowed, API Gateway will proceed with the backend integration configured on the method that was called

    // this function must generate a policy that is associated with the recognized principal user identifier.
    // depending on your use case, you might store policies in a DB, or generate them on the fly

    // keep in mind, the policy is cached for 5 minutes by default (TTL is configurable in the authorizer)
    // and will apply to subsequent calls to any method/resource in the RestApi
    // made with the same token

    //the example policy below denies access to all resources in the RestApi
    
    // new! -- add additional key-value pairs associated with the authenticated principal
    // these are made available by APIGW like so: $context.authorizer.<key>
    // additional context is cached
    apigw::ApiGatewayCustomAuthorizerResponse {
        principal_id: principal_id,
        policy_document: policy,
        context: HashMap::new(),
        usage_identifier_key: None,
    }
}
