use std::path::{Path, PathBuf};
use tempdir::TempDir;

#[derive(Debug)]
pub enum WorkDir {
    Temporary(TempDir),
    Persistent(PathBuf),
}

impl WorkDir {
    pub fn path(&self) -> &Path {
        match self {
            WorkDir::Temporary(t) => t.path(),
            WorkDir::Persistent(p) => p.as_path(),
        }
    }
}
