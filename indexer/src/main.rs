pub mod config;
pub mod error;
pub mod repo;
pub mod result;
pub mod work_dir;

use config::Config;
use repo::Repo;
use result::IndexerResult;

fn main() {
    env_logger::init();
    let config = Config::load().expect("config");

    index(&config).expect("index");
}

pub fn index(config: &Config) -> IndexerResult<()> {
    let repo = Repo::new(config)?;
    log::info!("indexing {:?}", config.index_git_url);
    repo.checkout()?;
    Ok(())
}
