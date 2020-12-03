use serde::{Serialize, Deserialize, Deserializer};
use serde::de::{self, Visitor};
use std::fmt;
use std::collections::HashMap;

/*
{
    "authorization_endpoint":"https://octomonkey.auth.eu-west-1.amazoncognito.com/oauth2/authorize",
    "id_token_signing_alg_values_supported":["RS256"],
    "issuer":"https://cognito-idp.eu-west-1.amazonaws.com/eu-west-1_u98DSW85w",
    "jwks_uri":"https://cognito-idp.eu-west-1.amazonaws.com/eu-west-1_u98DSW85w/.well-known/jwks.json",
    "response_types_supported":["code","token"],
    "scopes_supported":["openid","email","phone","profile"],
    "subject_types_supported":["public"],
    "token_endpoint":"https://octomonkey.auth.eu-west-1.amazoncognito.com/oauth2/token",
    "token_endpoint_auth_methods_supported":["client_secret_basic","client_secret_post"],
    "userinfo_endpoint":"https://octomonkey.auth.eu-west-1.amazoncognito.com/oauth2/userInfo"
}
*/

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct OpenIdConfiguration {
    pub authorization_endpoint: Option<String>,
    pub id_token_signing_alg_values_supported: Option<Vec<String>>,
    pub issuer: String,
    pub jwks_uri: Option<String>,
    pub response_types_supported: Vec<ResponseType>,
    pub scopes_supported: Option<Vec<Scope>>,
    pub subject_types_supported: Vec<SubjectType>,
    pub token_endpoint: Option<String>,
    pub token_endpoint_auth_methods_supported: Option<Vec<TokenEndpointAuthMethod>>,
    pub userinfo_endpoint: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ResponseType {
    #[serde(rename="code")]
    Code,
    #[serde(rename="token")]
    Token,
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Scope {
    #[serde(rename="openid")]
    OpenId,
    #[serde(rename="email")]
    Email,
    #[serde(rename="phone")]
    Phone,
    #[serde(rename="profile")]
    Profile,

    Custom(String)
}

impl<'de> Deserialize<'de> for Scope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ScopeVisitor)
    }
}

pub struct ScopeVisitor;

impl<'de> Visitor<'de> for ScopeVisitor {
    type Value = Scope;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a Scope string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(match s {
            "openid" => Scope::OpenId,
            "email" => Scope::Email,
            "phone" => Scope::Phone,
            "profile" => Scope::Profile,
            other => Scope::Custom(other.to_owned())
        })
    }
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum SubjectType {
    #[serde(rename="public")]
    Public,

    Other(String)
}

impl<'de> Deserialize<'de> for SubjectType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SubjectTypeVisitor)
    }
}

pub struct SubjectTypeVisitor;

impl<'de> Visitor<'de> for SubjectTypeVisitor {
    type Value = SubjectType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a SubjectType string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(match s {
            "public" => SubjectType::Public,
            other => SubjectType::Other(other.to_owned())
        })
    }
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum TokenEndpointAuthMethod {
    #[serde(rename="client_secret_basic")]
    ClientSecretBasic,
    #[serde(rename="client_secret_post")]
    ClientSecretPost,
    
    Other(String)
}

impl<'de> Deserialize<'de> for TokenEndpointAuthMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TokenEndpointAuthMethodVisitor)
    }
}

pub struct TokenEndpointAuthMethodVisitor;

impl<'de> Visitor<'de> for TokenEndpointAuthMethodVisitor {
    type Value = TokenEndpointAuthMethod;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a TokenEndpointAuthMethod string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(match s {
            "client_secret_basic" => TokenEndpointAuthMethod::ClientSecretBasic,
            "client_secret_post" => TokenEndpointAuthMethod::ClientSecretPost,
            other => TokenEndpointAuthMethod::Other(other.to_owned())
        })
    }
}

/*
{
  "keys": [
    {
      "alg": "RS256",
      "e": "AQAB",
      "kid": "jOm4Jn3r0fxcMI8SNeOApuvbjung6kg5+vaTqUO326k=",
      "kty": "RSA",
      "n": "4LPvRAM6F38dCo69FJh-Pp54j5oezXAOJEtgZEd4vnNo5t-7m93S7eYORbZFm5tUPDs4YJGzgAwyBNEGLwvXXebhWSobrOL22lTy79jn13zlnWyWJXlLBllfWyjf1WxLFwYEfBIx0cRzSLD_EW19yZAd8A3muEj6LPnroGdZ_wMHSce27Xp-GeXlkIg0uWBnp97turhyZNNKaQx-74KUsVxB5zzWowsl4zgwyWu2FI1F_ICB4e3gatRfMpmYJaxD7RwIfdp5c3qCmm-gWXUFPth2tvunYRbIx2Ap7n-Tdxbk3wSjoDv36FDZKpYZO8VIwHz5QUio8Cl4-8knQ8wYrw",
      "use": "sig"
    },
    {
      "alg": "RS256",
      "e": "AQAB",
      "kid": "j5VSbmnQ4S1Hen2u1m/9k7pgE/ryc4ERLRI/+cY9o+A=",
      "kty": "RSA",
      "n": "gaqN7tBXjeQM9WszZL28g-pt_kZaTFN1_Xx4FaK0p_O99AsVlben8wfGfW1KEhHzBOUGambto3q5cfROjLpFDDtuO6k7l5WgXI4siHzV93XrGQAJV6103eZm5J-GA-Xkw5l5VV_0aiIg-W_NKdMGBFoh8aPp4he6wa7KTBHKzNmS_omD1nDAxKqp84Q_LjGcwmCRZMmXPSfb9Y181g_rCdBhV6fD9k4eNjlgDRxdoZDMtCRu1cvMnvP0mGklptn-axFrkjvAQ2h2JrkB-JgOLtxSuSDZaLDanSfWV0RmWXCvAcHRSxhJeNzCtcfuaoaRKjl-Hnlp7YZuzrA5ucFf0Q",
      "use": "sig"
    }
  ]
}
*/

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct KeySet {
    pub keys: Vec<Key>,
}

impl KeySet {
    pub fn find_key<'a>(&'a self, kid: &str) -> Option<&'a Key> {
        self.keys.iter().find(|k| k.kid() == kid)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag="kty")]
pub enum Key {
    RSA(RSAKey),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RSAKey {
    pub alg: String,
    pub e: String,
    pub kid: String,
    pub n: String,
    #[serde(rename="use")]
    pub use_: String,
}

impl Key {
    pub fn kid<'a>(&'a self) -> &str {
        match self {
            Key::RSA(k) => {
                &k.kid
            }
        }
    }
}

/*
{
  "sub": "a1234567-b123-c123-d123-e1234567890a",
  "email_verified": true,
  "iss": "https://cognito-idp.eu-west-1.amazonaws.com/eu-west-1_foo",
  "phone_number_verified": false,
  "cognito:username": "a1234567-b123-c123-d123-e1234567890a",
  "aud": "app id",
  "event_id": "bar",
  "token_use": "id",
  "auth_time": 1605040679,
  "phone_number": "+4407123456789",
  "exp": 1606986655,
  "iat": 1606983055,
  "email": "me@example.com"
}
*/
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IdToken {
    pub sub: String,
    pub email_verified: Option<bool>,
    pub iss: String,
    pub aud: String,
    pub token_use: Option<String>,
    pub exp: i64,
    pub iat: i64,
    pub email: Option<String>,

    #[serde(flatten)]
    pub claims: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashmap;
    use serde_json::json;

    #[test]
    fn test_deserialize_id_token() { 
        let json_str = r#"{
            "sub": "a1234567-b123-c123-d123-e1234567890a",
            "email_verified": true,
            "iss": "https://cognito-idp.eu-west-1.amazonaws.com/eu-west-1_foo",
            "phone_number_verified": false,
            "cognito:username": "a1234567-b123-c123-d123-e1234567890a",
            "aud": "app id",
            "event_id": "bar",
            "token_use": "id",
            "auth_time": 1605040679,
            "phone_number": "+4407123456789",
            "exp": 1606986655,
            "iat": 1606983055,
            "email": "me@example.com"
          }"#;
        let id_token = serde_json::from_str::<IdToken>(json_str).expect("from_str");
        let expected = IdToken {
            sub: "a1234567-b123-c123-d123-e1234567890a".to_string(),
            email_verified: Some(true),
            iss: "https://cognito-idp.eu-west-1.amazonaws.com/eu-west-1_foo".to_string(),
            aud: "app id".to_string(),
            token_use: Some("id".to_string()),
            exp: 1606986655,
            iat: 1606983055,
            email: Some("me@example.com".to_string()),
            claims: hashmap!{
                "cognito:username".to_string() => json!("a1234567-b123-c123-d123-e1234567890a"),
                "event_id".to_string() => json!("bar"),
                "phone_number".to_string() => json!("+4407123456789"),
                "phone_number_verified".to_string() => json!(false),
                "auth_time".to_string() => json!(1605040679),
            }
        };
        assert_eq!(id_token, expected);
    }

    #[test]
    fn test_serialize_custom_scope() {
        let expected = r#""foo""#;
        let scope = Scope::Custom("foo".to_string());
        let json_str = serde_json::to_string(&scope).expect("to_string");
        assert_eq!(json_str, expected);
    }

    #[test]
    fn test_deserialize_custom_scope() {
        let json_str = r#""foo""#;
        let expected = Scope::Custom("foo".to_string());
        let scope = serde_json::from_str::<Scope>(json_str).expect("from_str");
        assert_eq!(scope, expected);
    }

    #[test]
    fn test_serialize_custom_subject_type() {
        let expected = r#""foo""#;
        let subject_type = SubjectType::Other("foo".to_string());
        let json_str = serde_json::to_string(&subject_type).expect("to_string");
        assert_eq!(json_str, expected);
    }

    #[test]
    fn test_deserialize_custom_subject_type() {
        let json_str = r#""foo""#;
        let expected = SubjectType::Other("foo".to_string());
        let subject_type = serde_json::from_str::<SubjectType>(json_str).expect("from_str");
        assert_eq!(subject_type, expected);
    }

    #[test]
    fn test_serialize_custom_auth_method() {
        let expected = r#""foo""#;
        let auth_method = TokenEndpointAuthMethod::Other("foo".to_string());
        let json_str = serde_json::to_string(&auth_method).expect("to_string");
        assert_eq!(json_str, expected);
    }

    #[test]
    fn test_deserialize_custom_auth_method() {
        let json_str = r#""foo""#;
        let expected = TokenEndpointAuthMethod::Other("foo".to_string());
        let auth_method = serde_json::from_str::<TokenEndpointAuthMethod>(json_str).expect("from_str");
        assert_eq!(auth_method, expected);
    }

    #[test]
    fn test_deserialize_key() {
        let json_str = r#"{
            "alg": "RS256",
            "e": "AQAB",
            "kid": "test kid",
            "kty": "RSA",
            "n": "test n param",
            "use": "sig"
          }"#;
        let expected = Key::RSA(RSAKey {
            alg: "RS256".to_string(),
            e: "AQAB".to_string(),
            kid: "test kid".to_string(),
            n: "test n param".to_string(),
            use_: "sig".to_string()
        });
        let key = serde_json::from_str::<Key>(json_str).expect("from_str");
        assert_eq!(key, expected);
    }

    #[test]
    fn test_deserialize_openid_config() {
        let json_str = r#"{
            "authorization_endpoint":"auth endpoint",
            "id_token_signing_alg_values_supported":["RS256"],
            "issuer":"iss url",
            "jwks_uri":"jwks uri",
            "response_types_supported":["code","token"],
            "scopes_supported":["openid","email","phone","profile","foo"],
            "subject_types_supported":["public"],
            "token_endpoint":"token endpoint url",
            "token_endpoint_auth_methods_supported":["client_secret_basic","client_secret_post"],
            "userinfo_endpoint":"userinfo endpoint url"
        }"#;
        let config = serde_json::from_str::<OpenIdConfiguration>(json_str).expect("from_str");
        let expected = OpenIdConfiguration {
            authorization_endpoint: Some("auth endpoint".to_string()),
            id_token_signing_alg_values_supported: Some(vec!["RS256".to_string()]),
            issuer: "iss url".to_string(),
            jwks_uri: Some("jwks uri".to_string()),
            response_types_supported: vec![ResponseType::Code, ResponseType::Token],
            scopes_supported: Some(vec![Scope::OpenId, Scope::Email, Scope::Phone, Scope::Profile, Scope::Custom("foo".to_string())]),
            subject_types_supported: vec![SubjectType::Public],
            token_endpoint: Some("token endpoint url".to_string()),
            token_endpoint_auth_methods_supported: Some(vec![TokenEndpointAuthMethod::ClientSecretBasic, TokenEndpointAuthMethod::ClientSecretPost]),
            userinfo_endpoint: Some("userinfo endpoint url".to_string())
        };
        assert_eq!(config, expected);
    }
}