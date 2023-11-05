use crate::ApiFuture;
use lambda_http::http;
use lambda_http::{Request, Response};
use serde::Serialize;

use crate::ApiResult;

pub const APPLICATION_JSON: &'static str = "application/json";
pub const TEXT_PLAIN: &'static str = "text/plain";

pub fn not_found<'a>(_req: &'a Request) -> ApiFuture<'a> {
    Box::pin(async {
        Ok(text_response(
            http::StatusCode::NOT_FOUND,
            TEXT_PLAIN,
            "Not Found",
        ))
    })
}

pub async fn not_implemented() -> ApiResult<Response<String>> {
    Ok(text_response(
        http::StatusCode::INTERNAL_SERVER_ERROR,
        TEXT_PLAIN,
        "Not Implemented",
    ))
}

pub fn json_response<T: Serialize>(status: http::StatusCode, body: T) -> Response<String> {
    match serde_json::to_string(&body) {
        Ok(json_body) => text_response(status, APPLICATION_JSON, json_body),
        Err(json_err) => text_response(
            http::StatusCode::INTERNAL_SERVER_ERROR,
            TEXT_PLAIN,
            format!("json error: {:?}", json_err),
        ),
    }
}

pub fn text_response<S: Into<String>>(
    status: http::StatusCode,
    content_type: &str,
    body: S,
) -> Response<String> {
    Response::builder()
        .status(status)
        .header("content-type", content_type)
        .body(body.into())
        .map_err(Box::new)
        .expect("failed to render response")
}
