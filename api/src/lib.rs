use crate::result::ApiResult;
use lambda_http::{http, Request, RequestExt, Response};
use serde_json;
use std::future::Future;
use std::pin::Pin;

pub mod error;
pub mod ext;
pub mod response;
pub mod result;
pub mod tokens;

pub type ApiFuture<'a> = Pin<Box<dyn Future<Output = ApiResult<Response<String>>> + Send + 'a>>;

pub struct User {
    pub id: String,
}

pub struct ApiRequestContext {
    pub user: User,
    pub body: serde_json::Value,
}

pub fn get_method(req: &Request) -> ApiResult<http_router::Method> {
    match *req.method() {
        http::Method::GET => Ok(http_router::Method::GET),
        http::Method::POST => Ok(http_router::Method::POST),
        http::Method::HEAD => Ok(http_router::Method::HEAD),
        http::Method::PUT => Ok(http_router::Method::PUT),
        http::Method::DELETE => Ok(http_router::Method::DELETE),
        http::Method::OPTIONS => Ok(http_router::Method::OPTIONS),
        http::Method::PATCH => Ok(http_router::Method::PATCH),
        http::Method::TRACE => Ok(http_router::Method::TRACE),
        _ => Err("Unsupported Method".into()),
    }
}

pub fn get_path(req: &Request) -> ApiResult<&str> {
    Ok(req.raw_http_path())
}
