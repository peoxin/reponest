use std::sync::Arc;
use std::time::Duration;
use tracing::error;

use crate::core::{self, RepoInfoWorker};
use crate::tui::state::AppState;

/// Spawn background task for repository scanning and info retrieval
pub fn spawn_scan_repo_and_get_info_task(state: &AppState) {
    let repos = state.repos.clone();
    let config = state.config.clone();

    tokio::spawn(async move {
        // Create a new worker for this scan operation
        let git_worker = Arc::new(RepoInfoWorker::for_repo_info());

        // Fast async directory scan to find all Git repositories
        match core::scan_directories(&config.main.scan_dirs, &config).await {
            Ok(repo_paths) => {
                // Submit all paths for background Git processing
                git_worker.submit_repos(&repo_paths);
            }
            Err(e) => {
                error!("Error scanning directories: {}", e);
            }
        }

        // Poll for results periodically and update state
        loop {
            tokio::time::sleep(Duration::from_millis(config.internal.refresh_interval)).await;

            let results = git_worker.poll_results();
            if results.is_empty() {
                // Check if all tasks are complete
                if git_worker.is_complete() {
                    break; // Worker finished all tasks
                }
                continue;
            }

            let mut repos_lock = repos.lock().await;
            for result in results {
                match result {
                    Ok(repo_info) => {
                        // Avoid duplicates
                        if !repos_lock
                            .iter()
                            .any(|r| r.basic.path == repo_info.basic.path)
                        {
                            repos_lock.push(repo_info);
                        }
                    }
                    Err(e) => {
                        error!("Error processing repo: {}", e);
                    }
                }
            }
        }
    });
}
