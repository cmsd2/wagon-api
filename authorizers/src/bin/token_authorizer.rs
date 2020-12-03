use std::collections::HashMap;

use serde_json::json;
use aws_lambda_events::event::apigw;
use authorizers::iam::ApiGatewayCustomAuthorizerPolicyBuilder;
use env_logger;
use lambda::{lambda, Context};
use log::{info, debug};
use simple_error::bail;

type ApiError = Box<dyn std::error::Error + Send + Sync + 'static>;
type ApiResult<T> = std::result::Result<T, ApiError>;

#[lambda]
#[tokio::main]
async fn main(
    req: apigw::ApiGatewayCustomAuthorizerRequest,
    ctx: Context,
) -> ApiResult<apigw::ApiGatewayCustomAuthorizerResponse<serde_json::Value>> {
    drop(env_logger::try_init());
    authorizer_handler(req, ctx).await
}

async fn authorizer_handler(
    event: apigw::ApiGatewayCustomAuthorizerRequest,
    ctx: Context,
) -> ApiResult<apigw::ApiGatewayCustomAuthorizerResponse<serde_json::Value>> {
    info!("Client token: {:?}", event.authorization_token);
    info!("Method ARN: {:?}", event.method_arn);

    // validate the incoming token
    // and produce the principal user identifier associated with the token

    // this could be accomplished in a number of ways:
    // 1. Call out to OAuth provider
    // 2. Decode a JWT token inline
    // 3. Lookup in a self-managed DB
    let principal_id = "user|a1b2c3d4";

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
    let tmp: Vec<&str> = event.method_arn.as_ref().unwrap().split(":").collect();
    let api_gateway_arn_tmp: Vec<&str> = tmp[5].split("/").collect();
    let aws_account_id = tmp[4];
    let region = tmp[3];
    let rest_api_id = api_gateway_arn_tmp[0];
    let stage = api_gateway_arn_tmp[1];

    let policy = ApiGatewayCustomAuthorizerPolicyBuilder::new(region, aws_account_id, rest_api_id, stage)
        .deny_all_methods()?
        .build();
    
    // new! -- add additional key-value pairs associated with the authenticated principal
    // these are made available by APIGW like so: $context.authorizer.<key>
    // additional context is cached
    Ok(apigw::ApiGatewayCustomAuthorizerResponse {
        principal_id: Some(principal_id.to_string()),
        policy_document: policy,
        context: HashMap::new(),
        usage_identifier_key: None,
    })
}

