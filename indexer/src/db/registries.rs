use super::error::DbError;
use super::result::DbResult;
use super::DbConfig;
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, GetItemInput};
use std::collections::HashMap;

pub struct Registry {
    pub url: String,
}

pub struct RegistryBuilder {
    pub url: Option<String>,
}

impl Default for RegistryBuilder {
    fn default() -> Self {
        RegistryBuilder { url: None }
    }
}

impl RegistryBuilder {
    pub fn validate(&self) -> DbResult<()> {
        if self.url.is_none() {
            return Err(DbError::InvalidRegistry("no url".to_owned()));
        }
        Ok(())
    }

    pub fn build(self) -> DbResult<Registry> {
        self.validate()?;
        Ok(Registry {
            url: self.url.unwrap(),
        })
    }
}

pub async fn get_registry(
    url: &str,
    client: DynamoDbClient,
    config: &DbConfig,
) -> DbResult<Registry> {
    let output = client
        .get_item(GetItemInput {
            key: hashmap! {"url".to_owned() => AttributeValue { s: Some(url.to_owned()), ..Default::default() }},
            table_name: config.registries_table.clone(),
            ..Default::default()
        })
        .await
        .map_err(|e| DbError::GetItemError(e))?;
    if let Some(item) = output.item {
        load_registry(item)
    } else {
        Err(DbError::RegistryNotFound(url.to_owned()))
    }
}

pub fn load_registry(values: HashMap<String, AttributeValue>) -> DbResult<Registry> {
    let mut registry = RegistryBuilder::default();
    registry.url = values.get("url").and_then(|v| v.s.clone());
    registry.build()
}
