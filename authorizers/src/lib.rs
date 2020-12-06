use std::collections::HashMap;
use futures::Future;
use std::pin::Pin;
use aws_lambda_events::event::apigw;
use lambda::Context;

pub mod result;
pub mod error;
pub mod iam;
pub mod jwt;
pub mod token;

use result::AuthResult;
use iam::policy_builder_for_method;

pub struct Claims {
    pub principal_id: String,
    pub scopes: Vec<&'static str>,
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output=T> + Send + 'a>>;

pub trait Authenticator {
    fn authenticate(event: &apigw::ApiGatewayCustomAuthorizerRequest) -> BoxFuture<AuthResult<Claims>>;

    fn authorize<'a>(event: &'a apigw::ApiGatewayCustomAuthorizerRequest, claims: &'a Claims) -> BoxFuture<'a, AuthResult<apigw::ApiGatewayCustomAuthorizerPolicy>>;

    fn auth(event: &apigw::ApiGatewayCustomAuthorizerRequest) -> BoxFuture<AuthResult<(Option<String>, apigw::ApiGatewayCustomAuthorizerPolicy)>> {
        Box::pin(async move {
            let claims = Self::authenticate(&event).await?;
            let policy = Self::authorize(&event, &claims).await?;
            Ok((Some(claims.principal_id), policy))
        })
    }
}

pub async fn authorizer_handler<T: Authenticator>(
    event: apigw::ApiGatewayCustomAuthorizerRequest,
    _ctx: Context,
) -> apigw::ApiGatewayCustomAuthorizerResponse<serde_json::Value> {
    log::info!("Client token: {:?}", event.authorization_token);
    log::info!("Method ARN: {:?}", event.method_arn);

    // validate the incoming token
    // and produce the principal user identifier associated with the token

    // this could be accomplished in a number of ways:
    // 1. Call out to OAuth provider
    // 2. Decode a JWT token inline
    // 3. Lookup in a self-managed DB

    let (principal_id, policy) = T::auth(&event).await.unwrap_or_else(|err| {
        log::info!("authentication failure: {:?}", err);

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
