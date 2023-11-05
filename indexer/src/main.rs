#[macro_use]
extern crate maplit;

pub mod config;
pub mod db;
pub mod error;
pub mod repo;
pub mod result;
pub mod work_dir;

use config::Config;
use db::registries::RegistryBuilder;
use db::DbConfig;
use repo::Repo;
use result::IndexerResult;
use rusoto_dynamodb::DynamoDbClient;

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = Config::load().expect("config");

    index(&config).await.expect("index");
}

pub async fn index(config: &Config) -> IndexerResult<()> {
    let client = DynamoDbClient::new(config.region.clone());
    let db_config = DbConfig::load()?;
    let repo = Repo::new(config)?;
    log::info!("indexing {:?}", config.index_git_url);
    let maybe_current_registry =
        db::registries::get_registry(&repo.remote_url, &client, &db_config).await?;
    let maybe_current_commit_id = maybe_current_registry
        .as_ref()
        .and_then(|r| r.head_commit_id.as_ref())
        .map(|s| git2::Oid::from_str(s))
        .map_or(Ok(None), |v| v.map(Some))?;
    repo.checkout()?;
    log::info!(
        "collecting changes between {:?} and {}",
        maybe_current_commit_id,
        repo.head_commit_id()?
    );
    let changed_files = repo.collect_changed_files(None, maybe_current_commit_id)?;
    for f in changed_files.iter() {
        log::debug!("found commit for {:?}", f);
    }
    log::info!("collecting files at HEAD");
    for f in repo.collect_files()? {
        if changed_files.contains(&f) {
            log::debug!("changed {:?}", f);
        } else {
            log::debug!("unchanged {:?}", f);
        }
    }
    let mut new_registry = new_registry_for_repo(&repo)?;
    if let Some(current_registry) = maybe_current_registry {
        new_registry.version = current_registry.version;
        if new_registry != current_registry {
            db::registries::update_registry(new_registry, &client, &db_config).await?;
        }
    } else {
        db::registries::put_registry(new_registry, &client, &db_config).await?;
    }

    Ok(())
}

pub fn new_registry_for_repo(repo: &Repo) -> IndexerResult<db::registries::Registry> {
    let mut registry = RegistryBuilder::default();
    registry.url = Some(repo.remote_url.clone());
    registry.head = Some(repo.remote_branch.clone());
    registry.head_commit_id = Some(repo.head_commit_id()?);
    registry.build().map_err(|e| e.into())
}
pub async fn save_registry(
    client: &DynamoDbClient,
    repo: &Repo,
    db_config: &DbConfig,
) -> IndexerResult<()> {
    let registry = new_registry_for_repo(repo)?;
    db::registries::upsert_registry(registry, client, db_config).await?;
    Ok(())
}
