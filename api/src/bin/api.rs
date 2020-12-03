#[macro_use]
extern crate http_router;

use std::collections::HashMap;

use aws_lambda_events::event::apigw;
use env_logger;
use http_router::Method;
use lambda::{lambda, Context};
use log;
use simple_error::bail;
use serde::{Serialize};
use futures_core::Future;
use std::pin::Pin;

use api::error::ApiError;
use api::result::ApiResult;
use api::tokens;

type LambdaError = Box<dyn std::error::Error + Send + Sync + 'static>;
type LambdaResult<T> = std::result::Result<T, LambdaError>;

type ApiFuture<'a> = Pin<Box<dyn Future<Output = ApiResult<apigw::ApiGatewayProxyResponse>> + Send + 'a>>;

#[lambda]
#[tokio::main]
async fn main(
    req: apigw::ApiGatewayProxyRequest,
    ctx: Context,
) -> LambdaResult<apigw::ApiGatewayProxyResponse> {
    drop(env_logger::try_init());
    api_handler(req, ctx).await.map_err(|e| e.into())
}

async fn api_handler(
    req: apigw::ApiGatewayProxyRequest,
    ctx: Context,
) -> ApiResult<apigw::ApiGatewayProxyResponse> {
    log::info!("{:?} {:?}", req.http_method, req.path);
    log::debug!("{:?}", ctx);
    log::debug!("{:?}", req.request_context);

    let router = router!(
        GET / => get_root,
        GET /api/token => get_token,
        POST /api/token => create_token,
        GET /api/v1/crates/{library: String}/{version: String}/download => download_crate,
        _ => not_found,
    );

    let method = get_method(&req)?;
    let path = get_path(&req)?;

    router(&req.request_context, method, &path).await
}

pub fn get_root<'a>(_context: &'a apigw::ApiGatewayProxyRequestContext) -> ApiFuture<'a> {
    Box::pin(async {
        not_implemented().await
    })
}

#[derive(Serialize, Debug, Clone)]
pub struct GetTokenResponse {
    pub token: Option<String>,
}

pub fn get_token<'a>(context: &'a apigw::ApiGatewayProxyRequestContext) -> ApiFuture<'a> {
    Box::pin(async move {
        let principal_id = context.principal()?;
        if let Some(token) = tokens::get_user_token(&principal_id).await? {
            Ok(json_response(200, GetTokenResponse { token: Some(token) }))
        } else {
            Ok(json_response(404, GetTokenResponse { token: None }))
        }
    })
}

pub fn create_token<'a>(context: &'a apigw::ApiGatewayProxyRequestContext) -> ApiFuture<'a> {
    Box::pin(async move {
        let principal_id = context.principal()?;
        let token = tokens::create_user_token(&principal_id).await?;
        Ok(json_response(201, GetTokenResponse { token: Some(token) }))
    })
}

pub fn download_crate<'a>(
    _context: &'a apigw::ApiGatewayProxyRequestContext,
    _library: String,
    _version: String,
) -> ApiFuture<'a> {
    Box::pin(async {
        not_implemented().await
    })
}

pub fn not_found<'a>(_context: &'a apigw::ApiGatewayProxyRequestContext) -> ApiFuture<'a> {
    Box::pin(async {
        Ok(text_response(404, "Not Found"))
    })
}

pub async fn not_implemented() -> ApiResult<apigw::ApiGatewayProxyResponse> {
    Ok(text_response(500, "Not Implemented"))
}

pub fn json_response<T: Serialize>(status: i64, body: T) -> apigw::ApiGatewayProxyResponse {
    match serde_json::to_string(&body) {
        Ok(json_body) => text_response(status, json_body),
        Err(json_err) => text_response(500, format!("json error: {:?}", json_err))
    }
}

pub fn text_response<S: Into<String>>(status: i64, body: S) -> apigw::ApiGatewayProxyResponse {
    apigw::ApiGatewayProxyResponse {
        body: Some(body.into()),
        status_code: status,
        headers: HashMap::new(),
        multi_value_headers: HashMap::new(),
        is_base64_encoded: Some(false),
    }
}

pub fn get_method(req: &apigw::ApiGatewayProxyRequest) -> ApiResult<Method> {
    if let Some(ref method) = req.http_method {
        match method.to_uppercase().as_str() {
            "GET" => Ok(Method::GET),
            "HEAD" => Ok(Method::HEAD),
            "OPTIONS" => Ok(Method::OPTIONS),
            "PATCH" => Ok(Method::PATCH),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "TRACE" => Ok(Method::TRACE),
            "CONNECT" => Ok(Method::CONNECT),
            "DELETE" => Ok(Method::DELETE),
            _ => Err("Unsupported Method".into()),
        }
    } else {
        bail!("Unrecognised Method");
    }
}

pub fn get_path(req: &apigw::ApiGatewayProxyRequest) -> ApiResult<String> {
    Ok(req.path.as_ref().map(|s| s.to_owned()).unwrap_or("/".to_string()))
}

pub trait OpenIdContext {
    fn principal(&self) -> ApiResult<String>;
}

impl OpenIdContext for apigw::ApiGatewayProxyRequestContext {
    fn principal(&self) -> ApiResult<String> {
        self.authorizer
            .get("principalId")
            .ok_or_else(|| ApiError::NotAuthorized("no principal".into()))
            .and_then(|v| 
                serde_json::from_value(v.to_owned())
                    .map_err(|err| ApiError::NotAuthorized(format!("principal deserialization error: {}", err))))
            
    }
}