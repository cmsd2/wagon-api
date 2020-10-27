use crate::config::Config;
use crate::error::IndexerError;
use crate::result::IndexerResult;
use crate::work_dir::WorkDir;
use git2::build::RepoBuilder;
use git2::{Cred, Direction, FetchOptions, RemoteCallbacks, Repository};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use tempdir::TempDir;

pub fn ls_remote(config: &Config) -> IndexerResult<()> {
    let repo = Repository::open(config.work_dir.clone())?;
    let remote_name = &config.remote_name;
    let mut remote = repo
        .find_remote(remote_name)
        .or_else(|_| repo.remote_anonymous(remote_name))?;

    let connection = remote.connect_auth(Direction::Fetch, None, None)?;

    for head in connection.list()?.iter() {
        println!("{}\t{}", head.oid(), head.name());
    }
    Ok(())
}

pub enum RepoState {
    Open(Rc<Repository>),
    None,
}

pub struct Repo {
    pub dir: WorkDir,
    pub path: PathBuf,
    pub remote_url: String,
    pub remote_name: String,
    pub remote_branch: String,
    pub username: String,
    pub password: String,
    state: RefCell<RepoState>,
}

impl Repo {
    pub fn new(config: &Config) -> IndexerResult<Self> {
        let dir = if config.persist_checkout {
            WorkDir::Persistent(config.work_dir.join("index"))
        } else {
            WorkDir::Temporary(TempDir::new_in(&config.work_dir, "index")?)
        };
        Ok(Repo {
            path: dir.path().to_path_buf(),
            dir: dir,
            remote_url: config.index_git_url.clone(),
            remote_name: config.remote_name.clone(),
            remote_branch: config.remote_branch.clone(),
            username: config.username.clone(),
            password: config.password.clone(),
            state: RefCell::new(RepoState::None),
        })
    }

    pub fn head_commit_id(&self) -> IndexerResult<String> {
        let git_repo = self.open()?;
        Ok(git_repo.refname_to_id("HEAD")?.to_string())
    }

    pub fn open(&self) -> IndexerResult<Rc<git2::Repository>> {
        let mut state = self.state.borrow_mut();

        match &*state {
            RepoState::None => {
                log::info!("opening repo at {:?}", self.path);
                let git_repo = Rc::new(Repository::open(&self.path)?);
                *state = RepoState::Open(git_repo.clone());
                Ok(git_repo)
            }
            RepoState::Open(git_repo) => Ok(git_repo.clone()),
        }
    }

    pub fn checkout(&self) -> IndexerResult<()> {
        if !self.dot_git_exists() {
            self.clone()?;
        } else {
            let id = self.fetch()?;
            self.reset_head(id)?;
        }
        log::info!("checkout {} in {:?}", self.remote_url, self.dir);
        Ok(())
    }

    pub fn collect_files(&self) -> IndexerResult<HashSet<PathBuf>> {
        let git_repo = self.open()?;

        let commit_id = git_repo.refname_to_id("HEAD")?;
        let commit = git_repo.find_commit(commit_id)?;

        let mut result = HashSet::new();
        commit
            .tree()?
            .walk(git2::TreeWalkMode::PreOrder, |root, entry| {
                if let Some(name) = entry.name() {
                    let mut path = PathBuf::from(root);
                    path.push(name);
                    result.insert(path);
                } else {
                    log::debug!("no path for {}", entry.id());
                }
                git2::TreeWalkResult::Ok
            })?;
        Ok(result)
    }

    pub fn collect_changed_files(
        &self,
        leaf: Option<git2::Oid>,
        root: Option<git2::Oid>,
    ) -> IndexerResult<HashSet<PathBuf>> {
        let results = self.revwalk(
            leaf,
            root,
            |commit_id, results| {
                self.commit_deltas(
                    commit_id,
                    |delta, mut results| {
                        if let Some(path) = delta.new_file().path() {
                            results.insert(path.to_owned());
                        }
                        Ok(results)
                    },
                    results,
                )
            },
            HashSet::new(),
        )?;

        Ok(results)
    }

    pub fn commit_deltas<R, F: Fn(git2::DiffDelta, R) -> IndexerResult<R>>(
        &self,
        commit_id: git2::Oid,
        callback: F,
        mut accumulator: R,
    ) -> IndexerResult<R> {
        let git_repo = self.open()?;
        let commit = git_repo.find_commit(commit_id)?;
        let commit_tree = commit.tree()?;
        let mut diff_options = git2::DiffOptions::new();

        for parent in commit.parents() {
            let parent_tree = parent.tree()?;
            let diff = git_repo.diff_tree_to_tree(
                Some(&parent_tree),
                Some(&commit_tree),
                Some(&mut diff_options),
            )?;
            for delta in diff.deltas() {
                accumulator = callback(delta, accumulator)?;
            }
        }
        Ok(accumulator)
    }

    pub fn revwalk<R, F: Fn(git2::Oid, R) -> IndexerResult<R>>(
        &self,
        leaf: Option<git2::Oid>,
        root: Option<git2::Oid>,
        callback: F,
        mut accumulator: R,
    ) -> IndexerResult<R> {
        let git_repo = self.open()?;
        let mut revwalk = git_repo.revwalk()?;
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;

        if let Some(leaf) = leaf {
            revwalk.push(leaf)?;
        } else {
            revwalk.push_head()?;
        }

        if let Some(root) = root {
            revwalk.hide(root)?;
        }

        for oid in revwalk {
            accumulator = callback(oid?, accumulator)?;
        }

        Ok(accumulator)
    }

    pub fn path_exists<P>(path: P) -> bool
    where
        P: AsRef<Path> + fmt::Debug,
    {
        match fs::metadata(&path) {
            Ok(_) => true,
            Err(_) => {
                log::debug!("path doesn't exist at {:?}", path);
                false
            }
        }
    }

    pub fn clone_dir_exists(&self) -> bool {
        Self::path_exists(&self.path)
    }

    pub fn dot_git_path(&self) -> PathBuf {
        self.path.join(".git")
    }

    pub fn dot_git_exists(&self) -> bool {
        Self::path_exists(&self.dot_git_path())
    }

    pub fn clone(&self) -> IndexerResult<git2::Oid> {
        if self.dot_git_exists() {
            return Err(IndexerError::CloneDirectoryAlreadyExists);
        }

        log::info!("cloning repo {} into {:?}", self.remote_url, self.path);

        let git_repo = Rc::new(self.repo_builder().clone(&self.remote_url, &self.path)?);

        *self.state.borrow_mut() = RepoState::Open(git_repo.clone());

        Ok(git_repo.refname_to_id("HEAD")?)
    }

    pub fn fetch(&self) -> IndexerResult<git2::Oid> {
        //let git_repo = Repository::open(&self.path)?;
        let remote_name = self.remote_name.clone();

        let git_repo = self.open()?;

        let mut remote = git_repo.find_remote(&remote_name)?;

        let mut fo = self.fetch_options();
        log::info!(
            "fetching remote branch {} from {}",
            self.remote_branch,
            self.remote_url,
        );
        remote.fetch(&[&self.remote_branch], Some(&mut fo), None)?;

        Ok(git_repo.refname_to_id("FETCH_HEAD")?)
    }

    pub fn reset_head(&self, oid: git2::Oid) -> IndexerResult<()> {
        let git_repo = self.open()?;

        log::info!("resetting head to {}", oid);
        let refname = format!("refs/heads/{}", self.remote_branch);
        git_repo.reference(
            &refname,
            oid,
            true,
            &format!("Setting {} to {}", self.remote_branch, oid),
        )?;
        git_repo.set_head(&refname)?;
        git_repo.checkout_head(Some(
            git2::build::CheckoutBuilder::default()
                .allow_conflicts(true)
                .conflict_style_merge(true)
                .force(),
        ))?;

        Ok(())
    }

    pub fn fetch_options<'a>(&'a self) -> FetchOptions<'a> {
        let mut callbacks = RemoteCallbacks::new();

        callbacks.transfer_progress(|stats| {
            if stats.received_objects() == stats.total_objects() {
                log::debug!(
                    "Resolving deltas {}/{}\r",
                    stats.indexed_deltas(),
                    stats.total_deltas()
                );
            } else if stats.total_objects() > 0 {
                log::debug!(
                    "Received {}/{} objects ({}) in {} bytes\r",
                    stats.received_objects(),
                    stats.total_objects(),
                    stats.indexed_objects(),
                    stats.received_bytes()
                );
            }
            true
        });

        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext(&self.username, &self.password)
        });
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);

        fo
    }

    pub fn repo_builder<'a>(&'a self) -> RepoBuilder<'a> {
        let fo = self.fetch_options();
        let mut rb = RepoBuilder::new();
        rb.fetch_options(fo);
        rb
    }
}
