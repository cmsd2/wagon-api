use std::collections::BTreeMap;
use std::borrow::Cow;
use serde::{Serialize, Deserialize};
use serde_json::{Map, Value};
use validator::{Validate, ValidationError};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_ALPHA_NUM: Regex = Regex::new(r"[0-9A-Za-z_-]").unwrap();
}

pub trait ValidationErrorExt {
    fn new2<S: Into<String>>(msg: S) -> ValidationError;
}

impl ValidationErrorExt for ValidationError {
    fn new2<S: Into<String>>(msg: S) -> ValidationError {
        ValidationError {
            code: Cow::Owned(msg.into()),
            message: None,
            params: Default::default(),
        }
    }
}

pub fn validate_create_crate_input(input: &CreateCrateInput) -> std::result::Result<(), ValidationError> {
    if input.license.is_none() && input.license_file.is_none() {
        return Err(ValidationError::new("either license or license_file is required"));
    }

    Ok(())
}

pub fn validate_features_map(map: &FeaturesMap) -> std::result::Result<(), ValidationError> {
    for key in map.keys() {
        if !RE_ALPHA_NUM.is_match(key) {
            return Err(ValidationError::new2(format!("feature name {} does not match regex {}", key, &RE_ALPHA_NUM as &Regex)));
        }
    }

    Ok(())
}

pub type FeaturesMap = BTreeMap<String, Vec<String>>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Validate)]
#[validate(schema(function = "validate_create_crate_input", skip_on_field_errors = false))]
pub struct CreateCrateInput {
    pub name: String,
    pub vers: String,
    pub deps: Vec<CreateCrateInputDependency>,
    #[validate(custom = "validate_features_map")]
    pub features: FeaturesMap,
    #[validate(length(min = 1))]
    pub authors: Vec<String>,
    #[validate(required, length(min = 1))]
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub readme: Option<String>,
    pub readme_file: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub repository: Option<String>,
    pub badges: Map<String, Value>,
    pub links: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CreateCrateInputDependency {
    pub name: String,
    pub version_req: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: CreateCrateInputDependencyKind,
    pub registry: Option<String>,
    pub explicit_name_in_toml: Option<String>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub enum CreateCrateInputDependencyKind {
    #[serde(rename="dev")]
    Dev,
    #[serde(rename="build")]
    Build,
    #[serde(rename="normal")]
    Normal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CreateCrateOutput {
    pub warnings: CreateCrateOutputWarnings,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CreateCrateOutputWarnings {
    pub invalid_categories: Vec<String>,
    pub invalid_badges: Vec<String>,
    pub other: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path;
    use serde_json::json;
    use serde::de::DeserializeOwned;
    use maplit::btreemap;

    #[test]
    fn test_create_crate_input_validate() {
        let input: CreateCrateInput = load_yaml("create-crate-input.yaml");
        input.validate().expect("valid");
    }

    #[test]
    fn test_create_crate_input_load() {
        let input: CreateCrateInput = load_yaml("create-crate-input.yaml");
        let expected = CreateCrateInput {
            name: "foo".to_string(),
            vers: "0.1.0".to_string(),
            deps: vec![
                CreateCrateInputDependency {
                    name: "rand".to_string(),
                    version_req: "^0.6".to_string(),
                    features: vec![
                        "i128_support".to_string(),
                    ],
                    optional: false,
                    default_features: true,
                    target: None,
                    kind: CreateCrateInputDependencyKind::Normal,
                    registry: None,
                    explicit_name_in_toml: None,
                }
            ],
            features: btreemap!(
                "extras".to_string() => vec![
                    "rand/simd_support".to_string(),
                ]
            ),
            authors: vec![
                "Alice <a@example.com>".to_string(),
            ],
            description: Some("A nice description.".to_string()),
            documentation: None,
            homepage: None,
            readme: None,
            readme_file: None,
            keywords: vec![],
            categories: vec![],
            license: None,
            license_file: Some("LICENSE".to_string()),
            repository: None,
            badges: json!({
                "travis-ci": {
                    "branch": "master",
                    "repository": "rust-lang/cargo",
                }
            }).as_object().ok_or("unexpected type").expect("expected object").to_owned(),
            links: None,
        };
        assert_eq!(input, expected);
    }

    fn load_yaml<T: DeserializeOwned>(name: &str) -> T {
        let s = load_file(name);
        serde_yaml::from_str(&s).expect("from str")
    }

    fn load_file(name: &str) -> String {
        let mut d = path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test");
        d.push(name);
        fs::read_to_string(&d)
            .expect("read file")
    }
}

