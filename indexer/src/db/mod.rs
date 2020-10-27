pub mod error;
pub mod registries;
pub mod result;

use error::DbError;
use result::DbResult;

pub struct DbConfig {
    pub registries_table: String,
}

pub struct DbConfigBuilder {
    pub registries_table: Option<String>,
}

impl Default for DbConfigBuilder {
    fn default() -> Self {
        DbConfigBuilder {
            registries_table: None,
        }
    }
}

impl DbConfig {
    pub fn load() -> DbResult<DbConfig> {
        let mut config = DbConfigBuilder::default();
        config.load()?;
        config.build()
    }
}

impl DbConfigBuilder {
    pub fn load(&mut self) -> DbResult<()> {
        if let Some(table) = maybe_get_env_var("INDEXER_REGISTRIES_TABLE")? {
            self.registries_table = Some(table);
        }

        Ok(())
    }

    pub fn validate(&self) -> DbResult<()> {
        if self.registries_table.is_none() {
            return Err(DbError::ConfigError(
                "missing registries table setting".to_string(),
            ));
        }

        Ok(())
    }

    pub fn build(self) -> DbResult<DbConfig> {
        self.validate()?;

        Ok(DbConfig {
            registries_table: self.registries_table.unwrap(),
        })
    }
}

pub fn get_env_var(name: &str) -> DbResult<String> {
    std::env::var(name).map_err(|e| DbError::ConfigError(format!("config key {}: {}", name, e)))
}

pub fn maybe_get_env_var(name: &str) -> DbResult<Option<String>> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(e) => Err(DbError::ConfigError(format!("config key {}: {}", name, e))),
    }
}
