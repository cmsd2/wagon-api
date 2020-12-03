use serde::{Serialize, Deserialize};
use serde_json;
use aws_lambda_events::event::apigw;
use crate::error::AuthError;
use crate::result::AuthResult;

pub static POLICY_VERSION: &str = "2012-10-17"; // override if necessary


pub struct ApiGatewayCustomAuthorizerPolicyBuilder {
    pub region: String,
    pub aws_account_id: String,
    pub rest_api_id: String,
    pub stage: String,
    pub policy: apigw::ApiGatewayCustomAuthorizerPolicy,
}

#[derive(Serialize, Deserialize)]
pub enum Method {
    #[serde(rename = "GET")]
    Get,
    #[serde(rename = "POST")]
    Post,
    #[serde(rename = "*PUT")]
    Put,
    #[serde(rename = "DELETE")]
    Delete,
    #[serde(rename = "PATCH")]
    Patch,
    #[serde(rename = "HEAD")]
    Head,
    #[serde(rename = "OPTIONS")]
    Options,
    #[serde(rename = "*")]
    All,
}

#[derive(Serialize, Deserialize)]
pub enum Effect {
    Allow,
    Deny,
}

impl ApiGatewayCustomAuthorizerPolicyBuilder {
    pub fn new(
        region: &str,
        account_id: &str,
        api_id: &str,
        stage: &str,
    ) -> ApiGatewayCustomAuthorizerPolicyBuilder {
        Self {
            region: region.to_string(),
            aws_account_id: account_id.to_string(),
            rest_api_id: api_id.to_string(),
            stage: stage.to_string(),
            policy: apigw::ApiGatewayCustomAuthorizerPolicy {
                version: Some(POLICY_VERSION.to_string()),
                statement: vec![],
            },
        }
    }

    pub fn add_method<T: Into<String>>(
        mut self,
        effect: Effect,
        method: Method,
        resource: T,
    ) -> AuthResult<Self> {
        let resource_arn = format!(
            "arn:aws:execute-api:{}:{}:{}/{}/{}/{}",
            &self.region,
            &self.aws_account_id,
            &self.rest_api_id,
            &self.stage,
            serde_json::to_value(&method)?.as_str().unwrap(),
            resource.into().trim_start_matches("/")
        );

        let stmt = apigw::IamPolicyStatement {
            effect: Some(serde_json::to_value(&effect)?.as_str().unwrap().to_owned()),
            action: vec!["execute-api:Invoke".to_string()],
            resource: vec![resource_arn],
        };

        self.policy.statement.push(stmt);
        Ok(self)
    }

    pub fn allow_all_methods(self) -> AuthResult<Self> {
        self.add_method(Effect::Allow, Method::All, "*")
    }

    pub fn deny_all_methods(self) -> AuthResult<Self> {
        self.add_method(Effect::Deny, Method::All, "*")
    }

    pub fn allow_method(self, method: Method, resource: String) -> AuthResult<Self> {
        self.add_method(Effect::Allow, method, resource)
    }

    pub fn deny_method(self, method: Method, resource: String) -> AuthResult<Self> {
        self.add_method(Effect::Deny, method, resource)
    }

    // Creates and executes a new child thread.
    pub fn build(self) -> apigw::ApiGatewayCustomAuthorizerPolicy {
        self.policy
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deny_all_policy() {
        let policy = ApiGatewayCustomAuthorizerPolicyBuilder::new("region", "account_id", "rest_api_id", "stage")
            .deny_all_methods()
            .expect("deny")
            .build();
        let policy_str = serde_json::to_string(&policy).expect("to_json");
        assert_eq!(policy_str, r#"{"Version":"2012-10-17","Statement":[{"Action":["execute-api:Invoke"],"Effect":"Deny","Resource":["arn:aws:execute-api:region:account_id:rest_api_id/stage/*/*"]}]}"#);
    }

    #[test]
    fn test_allow_some_policy() {
        let policy = ApiGatewayCustomAuthorizerPolicyBuilder::new("region", "account_id", "rest_api_id", "stage")
            .allow_method(Method::Get, "/api/token".to_string())
            .expect("allow")
            .build();
        let policy_str = serde_json::to_string(&policy).expect("to_json");
        assert_eq!(policy_str, r#"{"Version":"2012-10-17","Statement":[{"Action":["execute-api:Invoke"],"Effect":"Allow","Resource":["arn:aws:execute-api:region:account_id:rest_api_id/stage/GET/api/token"]}]}"#);
    }
}