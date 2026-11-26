//! Repository metadata utilities for Warp.
//!
//! Repository detection, Git path filtering, and filesystem watching for terminal metadata.
use std::{
    borrow::Borrow,
    path::{Path, PathBuf},
};

use thiserror::Error;
use warp_util::standardized_path::StandardizedPath;
use warpui::SingletonEntity;

/// Errors that can occur when working with repository metadata.
#[derive(Error, Debug)]
pub enum RepoMetadataError {
    #[error("Repository not found: {0}")]
    RepoNotFound(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Path encoding does not match local OS: {0}")]
    PathEncodingMismatch(StandardizedPath),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to start watcher: {0}")]
    WatcherError(#[from] anyhow::Error),
}
mod git_paths;
pub mod repositories;
pub mod repository;
pub mod watcher;

pub use repository::Repository;
pub use watcher::{DirectoryWatcher, RepositoryUpdate, TargetFile};

pub fn is_in_repo(path: &str, app: &warpui::AppContext) -> bool {
    use crate::repositories::DetectedRepositories;

    DetectedRepositories::as_ref(app)
        .get_root_for_path(std::path::Path::new(path))
        .is_some()
}

/// A wrapper around PathBuf that ensures the path is canonicalized.
/// This helps avoid issues with symbolic links, relative paths, and different path representations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonicalizedPath(PathBuf);

impl std::fmt::Display for CanonicalizedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.display().fmt(f)
    }
}

impl CanonicalizedPath {
    pub fn as_path_buf(&self) -> &PathBuf {
        &self.0
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// See [`PathBuf::pop`]
    pub fn pop(&mut self) -> bool {
        self.0.pop()
    }

    pub fn starts_with(&self, path: &CanonicalizedPath) -> bool {
        self.0.starts_with(&path.0)
    }
}

impl TryFrom<PathBuf> for CanonicalizedPath {
    type Error = std::io::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let canonical = dunce::canonicalize(&value)?;
        Ok(CanonicalizedPath(canonical))
    }
}

impl TryFrom<&Path> for CanonicalizedPath {
    type Error = std::io::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let canonical = dunce::canonicalize(value)?;
        Ok(CanonicalizedPath(canonical))
    }
}

impl TryFrom<&PathBuf> for CanonicalizedPath {
    type Error = std::io::Error;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let canonical = dunce::canonicalize(value)?;
        Ok(CanonicalizedPath(canonical))
    }
}

impl TryFrom<&str> for CanonicalizedPath {
    type Error = std::io::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let path = PathBuf::from(value);
        let canonical = dunce::canonicalize(&path)?;
        Ok(CanonicalizedPath(canonical))
    }
}

impl From<CanonicalizedPath> for PathBuf {
    fn from(canonical: CanonicalizedPath) -> Self {
        canonical.0
    }
}

impl From<&CanonicalizedPath> for PathBuf {
    fn from(canonical: &CanonicalizedPath) -> Self {
        canonical.0.clone()
    }
}

// We can implement Borrow<Path> and Borrow<PathBuf> because CanonicalizedPath uses identical trait
// implementations to those types (particularly for Hash, Ord, and Eq). If the implementations of
// those traits differ, the Borrow implementations will no longer be sound.

impl Borrow<Path> for CanonicalizedPath {
    fn borrow(&self) -> &Path {
        self.0.borrow()
    }
}

impl Borrow<PathBuf> for CanonicalizedPath {
    fn borrow(&self) -> &PathBuf {
        &self.0
    }
}

impl From<CanonicalizedPath> for StandardizedPath {
    fn from(canonical: CanonicalizedPath) -> Self {
        // CanonicalizedPath is always absolute and local, so try_from_local will not fail.
        StandardizedPath::try_from_local(canonical.as_path())
            .expect("CanonicalizedPath is always absolute")
    }
}
