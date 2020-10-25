use git2;
use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum IndexerError {
    ConfigError(String),
    IoError(io::Error),
    GitError(git2::Error),
    CloneDirectoryAlreadyExists,
}

impl Error for IndexerError {}

impl fmt::Display for IndexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::env::VarError> for IndexerError {
    fn from(e: std::env::VarError) -> Self {
        IndexerError::ConfigError(format!("env var error: {}", e))
    }
}

impl From<git2::Error> for IndexerError {
    fn from(e: git2::Error) -> Self {
        IndexerError::GitError(e)
    }
}

impl From<io::Error> for IndexerError {
    fn from(e: io::Error) -> Self {
        IndexerError::IoError(e)
    }
}
