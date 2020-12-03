use super::error::DbError;
use super::result::DbResult;
use super::DbConfig;
use rusoto_dynamodb::{
    AttributeValue, DynamoDb, DynamoDbClient, GetItemInput, PutItemInput,
    UpdateItemInput,
};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Registry {
    pub url: String,
    pub version: i64,
    pub head: Option<String>,
    pub head_commit_id: Option<String>,
}

pub struct RegistryBuilder {
    pub url: Option<String>,
    pub version: Option<i64>,
    pub head: Option<String>,
    pub head_commit_id: Option<String>,
}

impl Default for RegistryBuilder {
    fn default() -> Self {
        RegistryBuilder {
            url: None,
            version: None,
            head: None,
            head_commit_id: None,
        }
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
            version: self.version.unwrap_or_default(),
            head: self.head,
            head_commit_id: self.head_commit_id,
        })
    }

    pub fn from_attributes(attrs: HashMap<String, AttributeValue>) -> DbResult<RegistryBuilder> {
        let mut b = RegistryBuilder::default();
        b.url = attrs.get("url").and_then(|attr| attr.s.clone());
        b.version = attrs
            .get("version")
            .and_then(|attr| attr.n.as_ref())
            .map(|s| s.parse::<i64>())
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|e| DbError::InvalidValue(format!("invalid version: {:?}", e)))?;
        b.head = attrs.get("head").and_then(|attr| attr.s.clone());
        b.head_commit_id = attrs.get("head_commit_id").and_then(|attr| attr.s.clone());
        Ok(b)
    }
}

impl Registry {
    pub fn key(&self) -> HashMap<String, AttributeValue> {
        hashmap! {"url".to_string() => string_attr_value(self.url.clone())}
    }

    pub fn put_item_values(&self) -> HashMap<String, AttributeValue> {
        let mut values = HashMap::new();
        add_item_value(&mut values, "url", string_attr_value(self.url.clone()));
        add_item_value(&mut values, "version", long_attr_value(self.version));
        add_item_value(
            &mut values,
            "head",
            maybe_string_attr_value(self.head.clone()),
        );
        add_item_value(
            &mut values,
            "head_commit_id",
            maybe_string_attr_value(self.head_commit_id.clone()),
        );
        values
    }

    pub fn update_expression_values(&self) -> HashMap<String, AttributeValue> {
        let mut values = HashMap::new();
        add_item_value(&mut values, ":version", long_attr_value(self.version));
        add_item_value(
            &mut values,
            ":head",
            maybe_string_attr_value(self.head.clone()),
        );
        add_item_value(
            &mut values,
            ":head_commit_id",
            maybe_string_attr_value(self.head_commit_id.clone()),
        );
        values
    }
}

pub async fn get_registry(
    url: &str,
    client: &DynamoDbClient,
    config: &DbConfig,
) -> DbResult<Option<Registry>> {
    log::debug!("get registry {}", url);
    let output = client
        .get_item(GetItemInput {
            key: hashmap! {"url".to_owned() => AttributeValue { s: Some(url.to_owned()), ..Default::default() }},
            table_name: config.registries_table.clone(),
            ..Default::default()
        })
        .await
        .map_err(|e| DbError::GetItemError(e))?;
    output
        .item
        .map(|item| {
            let registry = load_registry(item)?;
            log::info!("loaded {:?}", registry);
            Ok(registry)
        })
        .map_or(Ok(None), |v| v.map(Some))
}

pub fn load_registry(values: HashMap<String, AttributeValue>) -> DbResult<Registry> {
    RegistryBuilder::from_attributes(values)?.build()
}

pub async fn put_registry(
    mut registry: Registry,
    client: &DynamoDbClient,
    config: &DbConfig,
) -> DbResult<Registry> {
    registry.version = i64::default();
    log::info!("put registry {:?}", registry);
    let input = PutItemInput {
        item: registry.put_item_values(),
        condition_expression: Some("attribute_not_exists(#U)".to_string()),
        expression_attribute_names: Some(hashmap! {"#U".to_string() => "url".to_string()}),
        table_name: config.registries_table.to_string(),
        ..Default::default()
    };
    client
        .put_item(input)
        .await
        .map_err(|e| DbError::PutItemError(e))?;
    Ok(registry)
}

pub async fn update_registry(
    registry: Registry,
    client: &DynamoDbClient,
    config: &DbConfig,
) -> DbResult<Registry> {
    log::info!("update registry {:?}", registry);
    let mut values = registry.update_expression_values();
    values.insert(":inc".to_string(), long_attr_value(1));
    let input = UpdateItemInput {
        key: registry.key(),
        expression_attribute_values: Some(values),
        condition_expression: Some("version = :version".to_string()),
        update_expression: Some(
            "SET version = :version + :inc, head = :head, head_commit_id = :head_commit_id"
                .to_string(),
        ),
        return_values: Some("ALL_NEW".to_string()),
        table_name: config.registries_table.to_string(),
        ..Default::default()
    };
    let output = client
        .update_item(input)
        .await
        .map_err(|e| DbError::UpdateItemError(e))?;
    output
        .attributes
        .ok_or_else(|| DbError::NoUpdateAttributes)
        .and_then(|attrs| RegistryBuilder::from_attributes(attrs))
        .and_then(|builder| {
            let registry = builder.build()?;
            log::info!("updated registry {:?}", registry);
            Ok(registry)
        })
}

pub async fn upsert_registry(
    mut registry: Registry,
    client: &DynamoDbClient,
    config: &DbConfig,
) -> DbResult<(Option<Registry>, Registry)> {
    log::info!("upsert {:?}", registry);
    if let Some(remote) = get_registry(&registry.url, &client, config).await? {
        registry.version = remote.version;
        if &remote != &registry {
            Ok((
                Some(remote),
                update_registry(registry, &client, config).await?,
            ))
        } else {
            Ok((Some(remote), registry))
        }
    } else {
        Ok((None, put_registry(registry, &client, config).await?))
    }
}
/*
pub async fn update_registry_with_retry(
    mut registry: Registry,
    client: &DynamoDbClient,
    config: &DbConfig,
    mut tries: usize,
) -> DbResult<Registry> {
    let mut result = None;
    let mut count = 0;
    while tries > 0 {
        tries -= 1;
        count += 1;
        log::info!("update registry with try {} {:?}", count, registry);
        result = match update_registry(registry.clone(), client, config).await {
            Ok(registry) => Some(Ok(registry)),
            Err(DbError::UpdateItemError(RusotoError::Service(
                UpdateItemError::ConditionalCheckFailed(msg),
            ))) => {
                log::info!(
                    "update registry try {} failed condition check: {}",
                    count,
                    msg
                );
                if let Some(remote) = get_registry(&registry.url, client, config).await? {
                    registry.version = remote.version;
                }
                None
            }
            Err(e) => Some(Err(e)),
        };
        if result.is_some() {
            break;
        }
    }
    result.unwrap_or(Err(DbError::InvalidValue(
        "update registry: tries is 0".to_string(),
    )))
}*/

fn add_item_value<K: Into<String>>(
    m: &mut HashMap<String, AttributeValue>,
    key: K,
    v: AttributeValue,
) {
    m.insert(key.into(), v);
}

fn long_attr_value(v: i64) -> AttributeValue {
    AttributeValue {
        n: Some(v.to_string()),
        ..Default::default()
    }
}

fn maybe_string_attr_value<S: Into<String>>(s: Option<S>) -> AttributeValue {
    AttributeValue {
        s: s.map(|s| s.into()),
        ..Default::default()
    }
}

fn string_attr_value<S: Into<String>>(s: S) -> AttributeValue {
    maybe_string_attr_value(Some(s))
}
