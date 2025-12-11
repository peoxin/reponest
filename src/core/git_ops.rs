//! This module provides Git-specific operations using both synchronous
//! and asynchronous patterns.
//!
//! Inspired by GitUI's async git operations:
//! https://github.com/gitui-org/gitui/tree/master/asyncgit

use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

use super::repo_info::RepoInfo;
use super::worker::Worker;

/// Gather repository information in parallel using rayon
///
/// This is the fastest way to process repositories synchronously.
pub fn get_repos_info_parallel(paths: &[PathBuf]) -> Vec<RepoInfo> {
    paths
        .par_iter()
        .filter_map(|path| RepoInfo::from_path(path.clone()).ok())
        .collect()
}

/// Worker for extracting repository information
pub type RepoInfoWorker = Worker<PathBuf, RepoInfo>;

impl RepoInfoWorker {
    /// Create a new repository information worker
    pub fn for_repo_info() -> Self {
        Self::new(RepoInfo::from_path)
    }

    /// Submit multiple repository paths to the worker
    ///
    /// This is a non-blocking batch operation. All paths are queued immediately,
    /// and results can be polled later using `poll_results()`.
    pub fn submit_repos(self: &Arc<Self>, paths: &[PathBuf]) {
        for path in paths {
            let _ = self.submit(path.clone());
        }
        self.finish_submitting();
    }
}
