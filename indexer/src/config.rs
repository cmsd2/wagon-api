use crate::error::IndexerError;
use crate::result::IndexerResult;
use rusoto_core::Region;
use std::path::PathBuf;

pub const DEFAULT_WORK_DIR: &str = "/tmp";

pub struct ConfigBuilder {
    pub region: Region,
    pub index_git_url: Option<String>,
    pub work_dir: PathBuf,
    pub remote_name: String,
    pub remote_branch: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub persist_checkout: bool,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        ConfigBuilder {
            region: Region::default(),
            index_git_url: None,
            work_dir: PathBuf::from(DEFAULT_WORK_DIR),
            remote_name: "origin".to_string(),
            remote_branch: "master".to_string(),
            username: None,
            password: None,
            persist_checkout: true,
        }
    }
}

pub struct Config {
    pub region: Region,
    pub index_git_url: String,
    pub work_dir: PathBuf,
    pub remote_name: String,
    pub remote_branch: String,
    pub username: String,
    pub password: String,
    pub persist_checkout: bool,
}

impl Config {
    pub fn load() -> IndexerResult<Config> {
        let mut config = ConfigBuilder::default();
        config.load()?;
        config.build()
    }
}

impl ConfigBuilder {
    pub fn load(&mut self) -> IndexerResult<()> {
        if let Some(git_url) = maybe_get_env_var("INDEXER_GIT_URL")? {
            self.index_git_url = Some(git_url);
        }

        if let Some(work_dir) = maybe_get_env_var("INDEXER_WORK_DIR")? {
            self.work_dir = PathBuf::from(work_dir);
        }

        if let Some(username) = maybe_get_env_var("INDEXER_GITHUB_USER")? {
            self.username = Some(username);
        }

        if let Some(password) = maybe_get_env_var("INDEXER_GITHUB_TOKEN")? {
            self.password = Some(password);
        }

        if let Some(branch) = maybe_get_env_var("INDEXER_GITHUB_BRANCH")? {
            self.remote_branch = branch;
        }

        Ok(())
    }

    pub fn validate(&self) -> IndexerResult<()> {
        if self.index_git_url.is_none() {
            return Err(IndexerError::ConfigError(format!(
                "missing git index url config setting"
            )));
        }

        if self.username.is_none() {
            return Err(IndexerError::ConfigError(format!(
                "missing git username config setting"
            )));
        }

        if self.password.is_none() {
            return Err(IndexerError::ConfigError(format!(
                "missing git password config setting"
            )));
        }

        Ok(())
    }

    pub fn build(self) -> IndexerResult<Config> {
        self.validate()?;

        Ok(Config {
            region: self.region,
            index_git_url: self.index_git_url.unwrap(),
            work_dir: self.work_dir,
            username: self.username.unwrap(),
            password: self.password.unwrap(),
            remote_name: self.remote_name,
            remote_branch: self.remote_branch,
            persist_checkout: self.persist_checkout,
        })
    }
}

pub fn get_env_var(name: &str) -> IndexerResult<String> {
    std::env::var(name)
        .map_err(|e| IndexerError::ConfigError(format!("config key {}: {}", name, e)))
}

pub fn maybe_get_env_var(name: &str) -> IndexerResult<Option<String>> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(e) => Err(IndexerError::ConfigError(format!(
            "config key {}: {}",
            name, e
        ))),
    }
}
