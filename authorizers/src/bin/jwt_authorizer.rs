use std::collections::HashMap;

use serde_json::json;
use aws_lambda_events::event::apigw;
use authorizers::jwt::IdToken;
use authorizers::iam::ApiGatewayCustomAuthorizerPolicyBuilder;
use authorizers::error::AuthError;
use env_logger;
use lambda::{lambda, Context};
use log::{info, debug};
use simple_error::bail;
use authorizers::jwt::TokenDecoder;
use lazy_static::lazy_static;
use std::env;

type ApiError = Box<dyn std::error::Error + Send + Sync + 'static>;
type ApiResult<T> = std::result::Result<T, ApiError>;

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
) -> ApiResult<apigw::ApiGatewayCustomAuthorizerResponse<serde_json::Value>> {
    drop(env_logger::try_init());

    authorizer_handler(req, ctx).await
}

async fn authenticate(event: &apigw::ApiGatewayCustomAuthorizerRequest) -> ApiResult<IdToken> {
    let authorization_header = event.authorization_token.as_ref().map(|s| &s[..]).ok_or_else(|| "missing auth token".to_string())?;

    let parts = authorization_header.split(" ").collect::<Vec<&str>>();
    let bearer: Option<&str> = parts.get(0).map(|s| *s);
    let token: Option<&str> = parts.get(1).map(|s| *s);

    if bearer != Some("Bearer") {
        return Err(AuthError::InputError("token does not start with Bearer".to_string()).into());
    }

    if let Some(token) = token {
        if token == "" {
            return Err(AuthError::InputError("empty bearer token".to_string()).into());
        }
        return Ok(TOKEN_DECODER.decode(token).await?);
    } else {
        return Err(AuthError::InputError("missing bearer token".to_string()).into());
    }
}

fn authorize(event: &apigw::ApiGatewayCustomAuthorizerRequest, claims: &IdToken) -> ApiResult<apigw::ApiGatewayCustomAuthorizerPolicy> {
    Ok(policy_builder_for_method(&event)
        .allow_all_methods()?
        .build())
}

async fn auth(event: &apigw::ApiGatewayCustomAuthorizerRequest) -> ApiResult<(Option<String>, apigw::ApiGatewayCustomAuthorizerPolicy)> {
    authenticate(&event).await
        .and_then(|claims| {
            authorize(&event, &claims)
                .map(|policy| (Some(claims.sub.to_string()), policy))
        })
        .or_else(|err| {
            info!("authentication failure: {:?}", err);

            Ok((None, policy_builder_for_method(&event)
                .deny_all_methods()?
                .build()))
        })
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

    let (principal_id, policy) = auth(&event).await?;
    

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
    Ok(apigw::ApiGatewayCustomAuthorizerResponse {
        principal_id: principal_id,
        policy_document: policy,
        context: HashMap::new(),
        usage_identifier_key: None,
    })
}

pub fn policy_builder_for_method(event: &apigw::ApiGatewayCustomAuthorizerRequest) -> ApiGatewayCustomAuthorizerPolicyBuilder {
    let tmp: Vec<&str> = event.method_arn.as_ref().map(|s| &s[..]).unwrap().split(":").collect();
    let api_gateway_arn_tmp: Vec<&str> = tmp[5].split("/").collect();
    let aws_account_id = tmp[4];
    let region = tmp[3];
    let rest_api_id = api_gateway_arn_tmp[0];
    let stage = api_gateway_arn_tmp[1];

    ApiGatewayCustomAuthorizerPolicyBuilder::new(region, aws_account_id, rest_api_id, stage)
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_policy_builder_for_event() {
        let req = apigw::ApiGatewayCustomAuthorizerRequest {
            type_: Some("TOKEN".to_string()),
            authorization_token: Some("foo".to_string()),
            method_arn: Some("arn:aws:execute-api:region:account:apiId/stage/verb/resource/childResource]".to_string()),
        };
        let builder = policy_builder_for_method(&req)
            .allow_all_methods()
            .expect("allow all");
        assert_eq!(builder.region, "region".to_string());
        assert_eq!(builder.aws_account_id, "account".to_string());
        assert_eq!(builder.rest_api_id, "apiId".to_string());
        assert_eq!(builder.stage, "stage".to_string());
        assert_eq!(builder.policy.statement, vec![
            apigw::IamPolicyStatement {
                action: vec!["execute-api:Invoke".to_string()],
                effect: Some("Allow".to_string()),
                resource: vec!["arn:aws:execute-api:region:account:apiId/stage/*/*".to_string()]
            }
        ]);
    }
}