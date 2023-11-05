#[macro_use]
extern crate http_router;

use aws_lambda_events::http;
use env_logger;
use lambda_http::{run, service_fn, Error, Request, RequestExt, Response};
use log;
use serde::Serialize;

use api::error::ApiError;
use api::ext::*;
use api::response::*;
use api::result::ApiResult;
use api::tokens;
use api::ApiFuture;
use api::{get_method, get_path};

type LambdaError = Box<dyn std::error::Error + Send + Sync + 'static>;
type LambdaResult<T> = std::result::Result<T, LambdaError>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    drop(env_logger::try_init());

    run(service_fn(function_handler)).await
}

async fn function_handler(request: Request) -> LambdaResult<Response<String>> {
    api_handler(request).await.or_else(|err| {
        log::error!("error: {:?}", err);
        Ok(match err {
            ApiError::NotAuthorized(_) => text_response(
                http::StatusCode::UNAUTHORIZED,
                TEXT_PLAIN,
                format!("Unauthorized"),
            ),
            ApiError::Database(s) => text_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                TEXT_PLAIN,
                format!("Internal Server Error: {}", s),
            ),
            ApiError::Other(s) => text_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                TEXT_PLAIN,
                format!("Internal Server Error: {}", s),
            ),
            ApiError::SerializationError(s) => text_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                TEXT_PLAIN,
                format!("Internal Server Error: {}", s),
            ),
            ApiError::InvalidInput(s) => text_response(
                http::StatusCode::BAD_REQUEST,
                TEXT_PLAIN,
                format!("Bad Request: {}", s),
            ),
        })
    })
}

async fn api_handler(req: Request) -> ApiResult<Response<String>> {
    log::info!("{:?} {:?}", req.method(), req.uri(),);
    log::debug!("{:?}", req.lambda_context());
    log::debug!("{:?}", req.query_string_parameters());
    log::debug!("{:?}", req.headers());

    let router = router!(
        GET / => get_root,
        GET /api/token => get_token,
        POST /api/token => create_token,
        _ => not_found,
    );

    let method = get_method(&req)?;
    let path = get_path(&req)?;

    router(&req, method, &path).await
}

pub fn get_root<'a>(_req: &'a Request) -> ApiFuture<'a> {
    Box::pin(async { not_implemented().await })
}

#[derive(Serialize, Debug, Clone)]
pub struct GetTokenResponse {
    pub token: Option<String>,
}

pub fn get_token<'a>(req: &'a Request) -> ApiFuture<'a> {
    Box::pin(async move {
        let principal_id = req.claims()?.principal_id();

        let token = tokens::get_or_create_token(&principal_id).await?;

        Ok(json_response(
            http::StatusCode::OK,
            GetTokenResponse { token: Some(token) },
        ))
    })
}

pub fn create_token<'a>(req: &'a Request) -> ApiFuture<'a> {
    Box::pin(async move {
        let principal_id = req.claims()?.principal_id();

        let token = tokens::create_user_token(&principal_id).await?;
        Ok(json_response(
            http::StatusCode::CREATED,
            GetTokenResponse { token: Some(token) },
        ))
    })
}

#[cfg(test)]
mod test {
    #[test]
    fn parse_create_crate_input() {}
}
