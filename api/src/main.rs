#[macro_use]
extern crate http_router;

use std::collections::HashMap;

use aws_lambda_events::event::apigw;
use env_logger;
use http_router::Method;
use lambda::{lambda, Context};
use log;
use simple_error::bail;

type ApiError = Box<dyn std::error::Error + Send + Sync + 'static>;
type ApiResult<T> = std::result::Result<T, ApiError>;

#[lambda]
#[tokio::main]
async fn main(
    req: apigw::ApiGatewayProxyRequest,
    ctx: Context,
) -> ApiResult<apigw::ApiGatewayProxyResponse> {
    env_logger::init();
    api_handler(req, ctx).await
}

async fn api_handler(
    req: apigw::ApiGatewayProxyRequest,
    ctx: Context,
) -> ApiResult<apigw::ApiGatewayProxyResponse> {
    log::info!("{:?} {:?}", req.http_method, req.path);

    let router = router!(
        GET / => get_root,
        GET /api/v1/crates/{library: String}/{version: String}/download => download_crate,
        _ => not_found,
    );

    let method = get_method(&req)?;
    let path = get_path(&req)?;

    router(ctx, method, path)
}

pub fn get_root(_context: &Context) -> ApiResult<apigw::ApiGatewayProxyResponse> {
    not_implemented()
}

pub fn download_crate(
    _context: &Context,
    _library: String,
    _version: String,
) -> ApiResult<apigw::ApiGatewayProxyResponse> {
    not_implemented()
}

pub fn not_found(_context: &Context) -> ApiResult<apigw::ApiGatewayProxyResponse> {
    Ok(text_response(404, "Not Found"))
}

pub fn not_implemented() -> ApiResult<apigw::ApiGatewayProxyResponse> {
    Ok(text_response(500, "Not Implemented"))
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

pub fn get_path(req: &apigw::ApiGatewayProxyRequest) -> ApiResult<&str> {
    Ok(req.path.as_ref().map(|s| &s[..]).unwrap_or("/"))
}
