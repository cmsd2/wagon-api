use std::collections::HashMap;
use std::error::Error;

use aws_lambda_events::event::apigw;
use lambda_runtime::{error::HandlerError, lambda, Context};
use log;
use serde::{Deserialize, Serialize};
use simple_error::bail;
use simple_logger;

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
    simple_logger::SimpleLogger::from_env().init()?;
    lambda!(my_handler);

    Ok(())
}

fn my_handler(
    e: apigw::ApiGatewayProxyRequest,
    _c: Context,
) -> Result<apigw::ApiGatewayProxyResponse, HandlerError> {
    log::info!("{:?} {:?}", e.http_method, e.path);

    Ok(apigw::ApiGatewayProxyResponse {
        body: None,
        status_code: 200,
        headers: HashMap::new(),
        multi_value_headers: HashMap::new(),
        is_base64_encoded: Some(false),
    })
}
