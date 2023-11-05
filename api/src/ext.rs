use std::collections::HashMap;

use crate::error::ApiError;
use crate::result::ApiResult;
use aws_lambda_events::apigw;
use lambda_http::{Request, RequestExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    auth_time: String, // id_token, access_token
    client_id: Option<String>,
    event_id: Option<String>,
    exp: String, // id_token, access_token
    iat: String, // id_token, access_token
    iss: String, // id_token, access_token
    jti: Option<String>,
    origin_jti: Option<String>,
    scope: Option<String>,          // access_token
    sub: String,                    // id_token, access_token
    aud: Option<String>,            // id_token
    email: Option<String>,          // id_token
    email_verified: Option<String>, // id_token
    token_use: String,              // id_token ("id"), access_token ("access")
    #[serde(rename = "cognito:username")]
    cognito_username: Option<String>, // id_token
    username: Option<String>,       // access_token
    version: Option<String>,
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

impl Claims {
    pub fn scopes(&self) -> HashSet<String> {
        self.scope
            .as_ref()
            .map(|s| s.split(' ').map(|s| s.to_owned()).collect())
            .unwrap_or_default()
    }

    pub fn principal_id(&self) -> String {
        self.sub.to_owned()
    }

    pub fn principal_id_ref(&self) -> &str {
        &self.sub
    }
}

pub trait AuthContext {
    fn apigw_request_context(&self) -> ApiResult<apigw::ApiGatewayProxyRequestContext>;

    fn claims(&self) -> ApiResult<Claims>;
}

impl AuthContext for Request {
    fn apigw_request_context(&self) -> ApiResult<apigw::ApiGatewayProxyRequestContext> {
        if let lambda_http::request::RequestContext::ApiGatewayV1(context) = self.request_context()
        {
            Ok(context)
        } else {
            Err(ApiError::Other(format!("no apigw context")))
        }
    }

    fn claims(&self) -> ApiResult<Claims> {
        let claims = self
            .apigw_request_context()?
            .authorizer
            .remove("claims")
            .ok_or_else(|| ApiError::Other(format!("missing claims")))?;

        serde_json::from_value(claims)
            .map_err(|_e| ApiError::Other(format!("deserialisation error")))
    }
}
