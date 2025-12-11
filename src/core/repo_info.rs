//! This module contains all data structures for representing Git repository information.

use git2::{Repository, StatusOptions};
use serde::Serialize;
use std::path::PathBuf;

/// Basic repository identification
#[derive(Debug, Clone, Serialize)]
pub struct RepoBasicInfo {
    pub path: PathBuf,
    pub name: String,
    pub branch: String,
}

/// Repository sync status with remote
#[derive(Debug, Clone, Default, Serialize)]
pub struct RepoSyncStatus {
    pub ahead: usize,
    pub behind: usize,
}

/// Repository working directory status
#[derive(Debug, Clone, Serialize)]
pub struct RepoWorkingStatus {
    pub is_dirty: bool,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    pub conflicts: usize,
}

/// Repository remote information
#[derive(Debug, Clone, Default, Serialize)]
pub struct RepoRemoteInfo {
    pub url: Option<String>,
}

/// Repository commit information
#[derive(Debug, Clone, Default, Serialize)]
pub struct RepoCommitInfo {
    pub message: Option<String>,
    pub author: Option<String>,
}

/// Repository stash information
#[derive(Debug, Clone, Default, Serialize)]
pub struct RepoStashInfo {
    pub count: usize,
}

/// File changes in the repository
#[derive(Debug, Clone, Default, Serialize)]
pub struct RepoFileChanges {
    pub changes: Vec<FileChange>,
}

/// Represents a change in a file within the repository
#[derive(Debug, Clone, Serialize)]
pub struct FileChange {
    pub path: String,
    pub status: FileChangeStatus,
}

/// Enum for the status of a file change
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FileChangeStatus {
    Staged,
    Modified,
    Untracked,
    Conflicted,
}

/// Information about a Git repository
#[derive(Debug, Clone, Serialize)]
pub struct RepoInfo {
    pub basic: RepoBasicInfo,
    pub sync: RepoSyncStatus,
    pub working: RepoWorkingStatus,
    pub remote: RepoRemoteInfo,
    pub commit: RepoCommitInfo,
    pub stash: RepoStashInfo,
    pub files: RepoFileChanges,
}

/// Statistics about file changes in the repository
struct FileChangeStatistic {
    working: RepoWorkingStatus,
    files: RepoFileChanges,
}

impl RepoInfo {
    /// Create a RepoInfo from a repository path
    pub fn from_path(path: PathBuf) -> Result<Self, String> {
        let mut repo = Repository::open(&path)
            .map_err(|e| format!("Failed to open repo at {:?}: {}", path, e))?;

        let basic = Self::get_basic_info(&repo, path)?;
        let sync = Self::get_sync_status(&repo);
        let change_stat = Self::get_file_changes(&repo)?;
        let remote = Self::get_remote_info(&repo);
        let commit = Self::get_commit_info(&repo);
        let stash = Self::get_stash_info(&mut repo);

        Ok(Self {
            basic,
            sync,
            working: change_stat.working,
            remote,
            commit,
            stash,
            files: change_stat.files,
        })
    }

    /// Get basic repository information
    fn get_basic_info(repo: &Repository, path: PathBuf) -> Result<RepoBasicInfo, String> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        let branch = match repo.head() {
            Ok(head) => head.shorthand().unwrap_or("?").to_string(),
            Err(_) => "?".to_string(),
        };

        Ok(RepoBasicInfo { path, name, branch })
    }

    /// Get repository sync status with remote
    fn get_sync_status(repo: &Repository) -> RepoSyncStatus {
        let (ahead, behind) = Self::get_ahead_behind(repo).unwrap_or((0, 0));
        RepoSyncStatus { ahead, behind }
    }

    /// Get ahead/behind counts with respect to the upstream
    fn get_ahead_behind(repo: &Repository) -> Result<(usize, usize), git2::Error> {
        let head = repo.head()?;
        let local_oid = head
            .target()
            .ok_or_else(|| git2::Error::from_str("HEAD has no target"))?;

        let branch_name = head
            .shorthand()
            .ok_or_else(|| git2::Error::from_str("No branch name"))?;

        let upstream_name = format!("refs/remotes/origin/{}", branch_name);
        let upstream = match repo.find_reference(&upstream_name) {
            Ok(r) => r,
            Err(_) => return Ok((0, 0)),
        };

        let upstream_oid = upstream
            .target()
            .ok_or_else(|| git2::Error::from_str("Upstream has no target"))?;

        let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid)?;
        Ok((ahead, behind))
    }

    /// Get file change statistics for the repository
    fn get_file_changes(repo: &Repository) -> Result<FileChangeStatistic, String> {
        let mut status_opts = StatusOptions::new();
        status_opts
            .show(git2::StatusShow::IndexAndWorkdir)
            .include_untracked(true);

        let statuses = repo
            .statuses(Some(&mut status_opts))
            .map_err(|e| format!("Failed to get statuses: {}", e))?;

        let is_dirty = statuses.iter().any(|s| s.status() != git2::Status::CURRENT);

        let mut staged = 0;
        let mut modified = 0;
        let mut untracked = 0;
        let mut conflicts = 0;
        let mut file_changes = Vec::new();

        for entry in statuses.iter() {
            let status = entry.status();
            let file_path = entry.path().unwrap_or("?").to_string();

            if status.is_conflicted() {
                conflicts += 1;
                file_changes.push(FileChange {
                    path: file_path,
                    status: FileChangeStatus::Conflicted,
                });
            } else if status.is_index_new()
                || status.is_index_modified()
                || status.is_index_deleted()
            {
                staged += 1;
                file_changes.push(FileChange {
                    path: file_path,
                    status: FileChangeStatus::Staged,
                });
            } else if status.is_wt_modified() || status.is_wt_deleted() {
                modified += 1;
                file_changes.push(FileChange {
                    path: file_path,
                    status: FileChangeStatus::Modified,
                });
            } else if status.is_wt_new() {
                untracked += 1;
                file_changes.push(FileChange {
                    path: file_path,
                    status: FileChangeStatus::Untracked,
                });
            }
        }

        Ok(FileChangeStatistic {
            working: RepoWorkingStatus {
                is_dirty,
                staged,
                modified,
                untracked,
                conflicts,
            },
            files: RepoFileChanges {
                changes: file_changes,
            },
        })
    }

    /// Get remote repository information
    fn get_remote_info(repo: &Repository) -> RepoRemoteInfo {
        // Try to get remote from current branch's upstream
        let remote_name = repo
            .head()
            .ok()
            .and_then(|head| {
                let branch_name = head.shorthand()?;
                let branch = repo
                    .find_branch(branch_name, git2::BranchType::Local)
                    .ok()?;
                branch.upstream().ok()?.name().ok()?.map(|s| s.to_string())
            })
            .and_then(|upstream_name| {
                // Extract remote name from upstream (e.g., "origin/main" -> "origin")
                upstream_name.split('/').next().map(|s| s.to_string())
            });

        // If we found a remote from upstream, use it
        if let Some(name) = remote_name
            && let Ok(remote) = repo.find_remote(&name)
            && let Some(url) = remote.url()
        {
            return RepoRemoteInfo {
                url: Some(url.to_string()),
            };
        }

        // Fallback to "origin"
        if let Ok(remote) = repo.find_remote("origin")
            && let Some(url) = remote.url()
        {
            return RepoRemoteInfo {
                url: Some(url.to_string()),
            };
        }

        // Fallback to first available remote
        if let Ok(remotes) = repo.remotes() {
            for remote_name in remotes.iter() {
                if let Some(name) = remote_name
                    && let Ok(remote) = repo.find_remote(name)
                    && let Some(url) = remote.url()
                {
                    return RepoRemoteInfo {
                        url: Some(url.to_string()),
                    };
                }
            }
        }

        RepoRemoteInfo { url: None }
    }

    /// Get the last commit information
    fn get_commit_info(repo: &Repository) -> RepoCommitInfo {
        match repo.head() {
            Ok(head) => {
                if let Ok(commit) = head.peel_to_commit() {
                    let message = commit
                        .message()
                        .map(|m| m.lines().next().unwrap_or("").to_string());
                    let author = Some(commit.author().name().unwrap_or("Unknown").to_string());

                    RepoCommitInfo { message, author }
                } else {
                    RepoCommitInfo::default()
                }
            }
            Err(_) => RepoCommitInfo::default(),
        }
    }

    /// Get the stash information
    fn get_stash_info(repo: &mut Repository) -> RepoStashInfo {
        let mut count = 0;
        let _ = repo.stash_foreach(|_index, _name, _oid| {
            count += 1;
            true
        });

        RepoStashInfo { count }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Repository, Signature};
    use std::fs;
    use std::path::Path;

    /// Helper function to create a test repository with initial commit
    fn create_test_repo(path: &Path) -> Repository {
        fs::create_dir_all(path).unwrap();
        let repo = Repository::init(path).unwrap();

        // Configure test user
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();

        // Create initial commit
        let sig = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        {
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
                .unwrap();
        }

        repo
    }

    /// Helper to create a file in the repository
    fn create_file(repo_path: &Path, filename: &str, content: &str) {
        let file_path = repo_path.join(filename);
        fs::write(file_path, content).unwrap();
    }

    #[test]
    fn test_repo_info_clean_repo() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let _repo = create_test_repo(repo_path);

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Test basic info
        assert_eq!(info.basic.branch, "main");
        assert!(!info.basic.name.is_empty());

        // Test working status - should be clean
        assert!(!info.working.is_dirty);
        assert_eq!(info.working.staged, 0);
        assert_eq!(info.working.modified, 0);
        assert_eq!(info.working.untracked, 0);
        assert_eq!(info.working.conflicts, 0);

        // Test file changes - should be empty
        assert_eq!(info.files.changes.len(), 0);

        // Test stash - should be empty
        assert_eq!(info.stash.count, 0);
    }

    #[test]
    fn test_repo_info_with_untracked_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let _repo = create_test_repo(repo_path);

        // Create untracked files
        create_file(repo_path, "untracked1.txt", "content1");
        create_file(repo_path, "untracked2.txt", "content2");

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Repo should be dirty with untracked files
        assert!(info.working.is_dirty);
        assert_eq!(info.working.untracked, 2);
        assert_eq!(info.working.staged, 0);
        assert_eq!(info.working.modified, 0);

        // Check file changes
        assert_eq!(info.files.changes.len(), 2);
        assert!(
            info.files
                .changes
                .iter()
                .all(|c| c.status == FileChangeStatus::Untracked)
        );
    }

    #[test]
    fn test_repo_info_with_staged_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let repo = create_test_repo(repo_path);

        // Create and stage a file
        create_file(repo_path, "staged.txt", "staged content");
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("staged.txt")).unwrap();
        index.write().unwrap();

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Repo should be dirty with staged file
        assert!(info.working.is_dirty);
        assert_eq!(info.working.staged, 1);
        assert_eq!(info.working.untracked, 0);
        assert_eq!(info.working.modified, 0);

        // Check file changes
        assert_eq!(info.files.changes.len(), 1);
        assert_eq!(info.files.changes[0].status, FileChangeStatus::Staged);
        assert_eq!(info.files.changes[0].path, "staged.txt");
    }

    #[test]
    fn test_repo_info_with_modified_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let repo = create_test_repo(repo_path);

        // Create, stage and commit a file
        create_file(repo_path, "modified.txt", "original content");
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("modified.txt")).unwrap();
        index.write().unwrap();

        let sig = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Add file", &tree, &[&parent])
            .unwrap();

        // Modify the file
        create_file(repo_path, "modified.txt", "modified content");

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Repo should be dirty with modified file
        assert!(info.working.is_dirty);
        assert_eq!(info.working.modified, 1);
        assert_eq!(info.working.staged, 0);
        assert_eq!(info.working.untracked, 0);

        // Check file changes
        assert_eq!(info.files.changes.len(), 1);
        assert_eq!(info.files.changes[0].status, FileChangeStatus::Modified);
    }

    #[test]
    fn test_repo_info_with_mixed_changes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let repo = create_test_repo(repo_path);

        // Create committed file
        create_file(repo_path, "committed.txt", "original");
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("committed.txt")).unwrap();
        index.write().unwrap();

        // Commit the file
        let sig = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Add file", &tree, &[&parent])
            .unwrap();

        // Create different types of changes
        create_file(repo_path, "committed.txt", "modified");
        create_file(repo_path, "untracked.txt", "new file");
        create_file(repo_path, "staged.txt", "staged file");

        let mut index = repo.index().unwrap();
        index.add_path(Path::new("staged.txt")).unwrap();
        index.write().unwrap();

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Repo should be dirty with all types of changes
        assert!(info.working.is_dirty);
        assert_eq!(info.working.modified, 1);
        assert_eq!(info.working.staged, 1);
        assert_eq!(info.working.untracked, 1);
        assert_eq!(info.working.conflicts, 0);

        // Check total file changes
        assert_eq!(info.files.changes.len(), 3);
    }

    #[test]
    fn test_repo_basic_info() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let _repo = create_test_repo(repo_path);

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Test path
        assert_eq!(info.basic.path, repo_path);

        // Test name extraction
        assert!(!info.basic.name.is_empty());

        // Test branch name - Git's default branch can be 'main' or 'master'
        assert!(
            info.basic.branch == "main" || info.basic.branch == "master",
            "Expected branch to be 'main' or 'master', got '{}'",
            info.basic.branch
        );
    }

    #[test]
    fn test_commit_info() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let _repo = create_test_repo(repo_path);

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Should have initial commit info
        assert_eq!(info.commit.message, Some("Initial commit".to_string()));
        assert_eq!(info.commit.author, Some("Test User".to_string()));
    }

    #[test]
    fn test_invalid_repo_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let invalid_path = temp_dir.path().join("nonexistent");

        let result = RepoInfo::from_path(invalid_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_remote_info() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let repo = create_test_repo(repo_path);

        // Add a remote
        repo.remote("origin", "https://github.com/test/repo.git")
            .unwrap();

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Should have remote info
        assert_eq!(
            info.remote.url,
            Some("https://github.com/test/repo.git".to_string())
        );
    }

    #[test]
    fn test_remote_info_no_remote() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let _repo = create_test_repo(repo_path);

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Should have no remote info
        assert_eq!(info.remote.url, None);
    }

    #[test]
    fn test_remote_info_multiple_remotes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let repo = create_test_repo(repo_path);

        // Add multiple remotes
        repo.remote("upstream", "https://github.com/upstream/repo.git")
            .unwrap();
        repo.remote("origin", "https://github.com/origin/repo.git")
            .unwrap();
        repo.remote("fork", "https://github.com/fork/repo.git")
            .unwrap();

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Should prefer origin when no upstream is set
        assert_eq!(
            info.remote.url,
            Some("https://github.com/origin/repo.git".to_string())
        );
    }

    #[test]
    fn test_remote_info_non_origin_remote() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let repo = create_test_repo(repo_path);

        // Add only a non-origin remote
        repo.remote("upstream", "https://github.com/upstream/repo.git")
            .unwrap();

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Should use the first available remote
        assert_eq!(
            info.remote.url,
            Some("https://github.com/upstream/repo.git".to_string())
        );
    }

    #[test]
    fn test_remote_info_with_upstream() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let repo = create_test_repo(repo_path);

        // Add multiple remotes
        repo.remote("origin", "https://github.com/origin/repo.git")
            .unwrap();
        repo.remote("upstream", "https://github.com/upstream/repo.git")
            .unwrap();

        // Get the current commit
        let head = repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();

        // Create upstream branch and set it as tracking
        repo.reference(
            "refs/remotes/upstream/main",
            commit.id(),
            false,
            "create upstream branch",
        )
        .unwrap();

        let mut branch = repo.find_branch("main", git2::BranchType::Local).unwrap();
        branch.set_upstream(Some("upstream/main")).unwrap();

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Should prefer upstream remote from branch tracking
        assert_eq!(
            info.remote.url,
            Some("https://github.com/upstream/repo.git".to_string())
        );
    }

    #[test]
    fn test_sync_status_up_to_date() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let repo = create_test_repo(repo_path);

        // Create a "remote" reference at the same commit
        let head = repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();
        repo.reference(
            "refs/remotes/origin/main",
            commit.id(),
            false,
            "create remote tracking branch",
        )
        .unwrap();

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Should be in sync
        assert_eq!(info.sync.ahead, 0);
        assert_eq!(info.sync.behind, 0);
    }

    #[test]
    fn test_sync_status_ahead() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let repo = create_test_repo(repo_path);

        // Get initial commit
        let head = repo.head().unwrap();
        let initial_commit = head.peel_to_commit().unwrap();

        // Create "remote" reference at initial commit
        repo.reference(
            "refs/remotes/origin/main",
            initial_commit.id(),
            false,
            "create remote tracking branch",
        )
        .unwrap();

        // Make a new local commit (ahead of remote)
        create_file(repo_path, "new_file.txt", "content");
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("new_file.txt")).unwrap();
        index.write().unwrap();

        let sig = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "New commit",
            &tree,
            &[&initial_commit],
        )
        .unwrap();

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Should be ahead by 1
        assert_eq!(info.sync.ahead, 1);
        assert_eq!(info.sync.behind, 0);
    }

    #[test]
    fn test_sync_status_behind() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let repo = create_test_repo(repo_path);

        // Get initial commit
        let head = repo.head().unwrap();
        let initial_commit = head.peel_to_commit().unwrap();

        // Make a commit that will be "remote"
        create_file(repo_path, "remote_file.txt", "remote content");
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("remote_file.txt")).unwrap();
        index.write().unwrap();

        let sig = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let remote_commit = repo
            .commit(
                None, // Don't update HEAD
                &sig,
                &sig,
                "Remote commit",
                &tree,
                &[&initial_commit],
            )
            .unwrap();

        // Create "remote" reference at the new commit
        repo.reference(
            "refs/remotes/origin/main",
            remote_commit,
            false,
            "create remote tracking branch",
        )
        .unwrap();

        // Reset HEAD to initial commit (behind remote)
        repo.reset(initial_commit.as_object(), git2::ResetType::Hard, None)
            .unwrap();

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Should be behind by 1
        assert_eq!(info.sync.ahead, 0);
        assert_eq!(info.sync.behind, 1);
    }

    #[test]
    fn test_sync_status_diverged() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let repo = create_test_repo(repo_path);

        // Get initial commit
        let head = repo.head().unwrap();
        let initial_commit = head.peel_to_commit().unwrap();

        // Create a "remote" commit
        create_file(repo_path, "remote_file.txt", "remote");
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("remote_file.txt")).unwrap();
        index.write().unwrap();

        let sig = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let remote_commit = repo
            .commit(None, &sig, &sig, "Remote commit", &tree, &[&initial_commit])
            .unwrap();

        // Create "remote" reference
        repo.reference(
            "refs/remotes/origin/main",
            remote_commit,
            false,
            "create remote tracking branch",
        )
        .unwrap();

        // Reset to initial and create different local commit
        repo.reset(initial_commit.as_object(), git2::ResetType::Hard, None)
            .unwrap();

        create_file(repo_path, "local_file.txt", "local");
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("local_file.txt")).unwrap();
        index.write().unwrap();

        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Local commit",
            &tree,
            &[&initial_commit],
        )
        .unwrap();

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Should be diverged (both ahead and behind)
        assert_eq!(info.sync.ahead, 1);
        assert_eq!(info.sync.behind, 1);
    }

    #[test]
    fn test_stash_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let _repo = create_test_repo(repo_path);

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Should have no stashes
        assert_eq!(info.stash.count, 0);
    }

    #[test]
    fn test_stash_with_entries() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let mut repo = create_test_repo(repo_path);

        // Create some changes
        create_file(repo_path, "stashed.txt", "stashed content");
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("stashed.txt")).unwrap();
        index.write().unwrap();

        // Create a stash
        let sig = Signature::now("Test User", "test@example.com").unwrap();
        repo.stash_save(&sig, "Test stash 1", None).unwrap();

        // Create more changes and another stash
        create_file(repo_path, "stashed2.txt", "stashed content 2");
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("stashed2.txt")).unwrap();
        index.write().unwrap();

        repo.stash_save(&sig, "Test stash 2", None).unwrap();

        let info = RepoInfo::from_path(repo_path.to_path_buf()).unwrap();

        // Should have 2 stashes
        assert_eq!(info.stash.count, 2);
    }
}
