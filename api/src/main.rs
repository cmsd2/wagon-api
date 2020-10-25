#[macro_use]
extern crate http_router;

use std::collections::HashMap;
use std::error::Error;

use aws_lambda_events::event::apigw;
use env_logger;
use http_router::Method;
use lambda_runtime::{error::HandlerError, lambda, Context};
use log;
use serde::{Deserialize, Serialize};
use simple_error::bail;

pub type HandlerResult<T> = Result<T, HandlerError>;

#[derive(Deserialize)]
struct CustomEvent {
    #[serde(rename = "firstName")]
    first_name: String,
}

#[derive(Serialize)]
struct CustomOutput {
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    lambda!(my_handler);

    Ok(())
}

fn my_handler(
    req: apigw::ApiGatewayProxyRequest,
    ctx: Context,
) -> HandlerResult<apigw::ApiGatewayProxyResponse> {
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

pub fn get_root(context: &Context) -> HandlerResult<apigw::ApiGatewayProxyResponse> {
    not_implemented()
}

pub fn download_crate(
    context: &Context,
    _library: String,
    _version: String,
) -> HandlerResult<apigw::ApiGatewayProxyResponse> {
    not_implemented()
}

pub fn not_found(context: &Context) -> HandlerResult<apigw::ApiGatewayProxyResponse> {
    Ok(text_response(404, "Not Found"))
}

pub fn not_implemented() -> HandlerResult<apigw::ApiGatewayProxyResponse> {
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

pub fn get_method(req: &apigw::ApiGatewayProxyRequest) -> HandlerResult<Method> {
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

pub fn get_path(req: &apigw::ApiGatewayProxyRequest) -> HandlerResult<&str> {
    Ok(req.path.as_ref().map(|s| &s[..]).unwrap_or("/"))
}
