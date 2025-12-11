use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::{AppConfig, ColorScheme};
use crate::core::RepoInfo;

/// Shared application state
/// We place app config within the state as it may be modified during runtime.
/// The app config is session specific and should be part of the state.
pub struct AppState {
    pub repos: Arc<Mutex<Vec<RepoInfo>>>,  // list of repos
    pub selected_index: Arc<Mutex<usize>>, // current selected repo index
    pub detail_view: Arc<Mutex<bool>>,     // whether in detail view
    pub config: Arc<AppConfig>,            // app config in current session
    pub colors: ColorScheme,               // color scheme from theme
}

/// Snapshot of UI state for rendering
#[derive(Clone)]
pub struct RenderSnapshot {
    pub repos: Vec<RepoInfo>,
    pub selected_index: usize,
    pub is_detail_view: bool,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let colors = config.ui.theme.colors();
        Self {
            repos: Arc::new(Mutex::new(Vec::new())),
            selected_index: Arc::new(Mutex::new(0)),
            detail_view: Arc::new(Mutex::new(false)),
            config: Arc::new(config),
            colors,
        }
    }

    /// Check if repository list is empty
    pub async fn is_repos_empty(&self) -> bool {
        let repos_lock = self.repos.lock().await;
        repos_lock.is_empty()
    }

    /// Move selection up
    pub async fn move_selection_up(&self) {
        let mut selected = self.selected_index.lock().await;
        if *selected > 0 {
            *selected -= 1;
        }
    }

    /// Move selection down
    pub async fn move_selection_down(&self) {
        let mut selected = self.selected_index.lock().await;
        let repos = self.repos.lock().await;
        *selected = (*selected + 1).min(repos.len().saturating_sub(1));
    }

    /// Get detail view status
    pub async fn is_detail_view(&self) -> bool {
        *self.detail_view.lock().await
    }

    /// Set detail view status
    pub async fn set_detail_view(&self, enabled: bool) {
        let mut detail = self.detail_view.lock().await;
        *detail = enabled;
    }

    /// Get a snapshot of state for rendering (using try_lock for sync context)
    /// Returns default values if locks are unavailable
    pub fn get_render_snapshot(&self) -> RenderSnapshot {
        RenderSnapshot {
            repos: self
                .repos
                .try_lock()
                .ok()
                .map(|r| r.clone())
                .unwrap_or_default(),
            selected_index: self
                .selected_index
                .try_lock()
                .ok()
                .map(|s| *s)
                .unwrap_or_default(),
            is_detail_view: self
                .detail_view
                .try_lock()
                .ok()
                .map(|d| *d)
                .unwrap_or_default(),
        }
    }

    /// Get the path of the currently selected repository
    pub async fn get_selected_repo_path(&self) -> Option<std::path::PathBuf> {
        let repos = self.repos.lock().await;
        let selected = self.selected_index.lock().await;
        repos.get(*selected).map(|repo| repo.basic.path.clone())
    }
}
