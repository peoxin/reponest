use anyhow::{Context, Result};
use crossterm::style::{Color, Stylize};
use std::time::Instant;
use tracing::{debug, info};

use crate::config::AppConfig;
use crate::core::{
    self,
    repo_info::{
        FileChangeStatus, RepoBasicInfo, RepoCommitInfo, RepoFileChanges, RepoInfo, RepoRemoteInfo,
        RepoStashInfo, RepoSyncStatus, RepoWorkingStatus,
    },
};

/// List repositories in the specified path
pub async fn list_repos(
    config: AppConfig,
    detail: bool,
    json: bool,
    dirty_filter: bool,
    conflict_filter: bool,
) -> Result<()> {
    let start = Instant::now();

    // Scan directories asynchronously to find Git repositories
    let repo_paths = core::scan_directories(&config.main.scan_dirs, &config)
        .await
        .context("Failed to scan directories")?;

    let scan_elapsed = start.elapsed();
    debug!(
        paths_found = repo_paths.len(),
        elapsed = ?scan_elapsed,
        "Async directory scan finished"
    );

    // Process repositories in parallel to gather Git information
    let repos = core::get_repos_info_parallel(&repo_paths);

    info!(
        repo_count = repos.len(),
        total_elapsed = ?start.elapsed(),
        git_elapsed = ?(start.elapsed() - scan_elapsed),
        "Repository processing finished"
    );

    let filtered_repos: Vec<&RepoInfo> = repos
        .iter()
        .filter(|r| !dirty_filter || r.working.is_dirty)
        .filter(|r| !conflict_filter || r.working.conflicts > 0)
        .collect();

    if json {
        print_repos_json(&filtered_repos)?;
    } else if detail {
        print_repos_detail(&filtered_repos);
    } else {
        print_repos_list(&filtered_repos);
    }

    Ok(())
}

/// Print repositories in JSON format
fn print_repos_json(repos: &[&RepoInfo]) -> Result<()> {
    let json =
        serde_json::to_string_pretty(&repos).context("Failed to serialize repositories to JSON")?;
    println!("{}", json);
    Ok(())
}

/// Print repositories in simple list format
fn print_repos_list(repos: &[&RepoInfo]) {
    if repos.is_empty() {
        info!("No repositories found");
        return;
    }

    info!(count = repos.len(), "Listing repositories");

    let views: Vec<CompactRepoView> = repos.iter().map(|repo| repo.to_compact_view()).collect();

    // Calculate column widths
    let max_name = views.iter().map(|v| v.name.len()).max().unwrap_or(0);
    let max_status = views.iter().map(|v| v.status.len()).max().unwrap_or(0);
    let max_branch = views.iter().map(|v| v.branch.len()).max().unwrap_or(0);

    // Print each repository
    for view in &views {
        let name_pad = max_name.saturating_sub(view.name.len());
        let status_pad = max_status.saturating_sub(view.status.len());
        let branch_pad = max_branch.saturating_sub(view.branch.len());

        println!(
            "{}{}  {}{}  {}{}  {}",
            view.name.as_str().with(Color::Cyan).bold(),
            " ".repeat(name_pad),
            view.status.as_str().with(view.status_color()).bold(),
            " ".repeat(status_pad),
            &view.branch,
            " ".repeat(branch_pad),
            view.path.as_str().with(Color::DarkGrey)
        );
    }
}

/// Print repositories in detailed format
fn print_repos_detail(repos: &[&RepoInfo]) {
    if repos.is_empty() {
        info!("No repositories found");
        return;
    }

    info!(
        count = repos.len(),
        "Displaying detailed repository information"
    );
    println!("Found {} repos:\n", repos.len());

    for (idx, repo) in repos.iter().enumerate() {
        if idx > 0 {
            println!();
        }

        println!("{}", "─".repeat(70).with(Color::DarkGrey));
        println!("{}", repo.basic.name.as_str().with(Color::Cyan).bold());

        for line in repo.to_detail_lines() {
            println!("  {}", line);
        }
    }

    println!("\n{}", "─".repeat(70).with(Color::DarkGrey));
}

/// Trait for RepoInfo formatting
trait RepoDisplay {
    fn to_compact_view(&self) -> CompactRepoView;
    fn to_detail_lines(&self) -> Vec<String>;
}

impl RepoDisplay for RepoInfo {
    fn to_compact_view(&self) -> CompactRepoView {
        CompactRepoView::from_repo(self)
    }

    fn to_detail_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();

        lines.extend(self.basic.format_for_detail());
        lines.extend(self.working.format_for_detail());

        if self.sync.has_content() {
            lines.extend(self.sync.format_for_detail());
        }
        if self.stash.has_content() {
            lines.extend(self.stash.format_for_detail());
        }
        if self.remote.has_content() {
            lines.extend(self.remote.format_for_detail());
        }
        if self.commit.has_content() {
            lines.extend(self.commit.format_for_detail());
        }

        lines
    }
}

/// Compact display data for list view
struct CompactRepoView {
    name: String,
    branch: String,
    status: String,
    path: String,
}

impl CompactRepoView {
    fn from_repo(repo: &RepoInfo) -> Self {
        let name = repo.basic.name.clone();
        let branch = repo.basic.branch.clone();
        let path = repo.basic.path.display().to_string();

        let status = if repo.working.conflicts > 0 {
            "conflict".to_string()
        } else if repo.working.is_dirty {
            "dirty".to_string()
        } else if repo.sync.ahead > 0 {
            "unpushed".to_string()
        } else if repo.sync.behind > 0 {
            "unpulled".to_string()
        } else {
            "clean".to_string()
        };

        Self {
            name,
            branch,
            status,
            path,
        }
    }

    fn status_color(&self) -> Color {
        if self.status.contains("conflict") {
            Color::Red
        } else if self.status.contains("dirty") {
            Color::Yellow
        } else if self.status.contains("unpushed") || self.status.contains("unpulled") {
            Color::Cyan
        } else {
            Color::Green
        }
    }
}

/// Format repository component for detailed view
trait DetailViewFormat {
    fn format_for_detail(&self) -> Vec<String>;

    /// Check if this component has content to display
    fn has_content(&self) -> bool {
        true
    }
}

impl DetailViewFormat for RepoBasicInfo {
    fn format_for_detail(&self) -> Vec<String> {
        vec![
            format!(
                "{}{}",
                "Path: ".with(Color::DarkGrey),
                self.path.display().to_string().with(Color::White)
            ),
            format!(
                "{}{}",
                "Branch: ".with(Color::DarkGrey),
                self.branch.as_str().with(Color::Green)
            ),
        ]
    }
}

impl DetailViewFormat for RepoSyncStatus {
    fn format_for_detail(&self) -> Vec<String> {
        if self.ahead == 0 && self.behind == 0 {
            return vec![];
        }

        let sync_info = if self.ahead > 0 && self.behind > 0 {
            format!(
                "{}{} ahead, {} behind",
                "Sync: ".with(Color::DarkGrey),
                format!("↑{}", self.ahead).with(Color::Cyan),
                format!("↓{}", self.behind).with(Color::Yellow)
            )
        } else if self.ahead > 0 {
            format!(
                "{}{} ahead",
                "Sync: ".with(Color::DarkGrey),
                format!("↑{}", self.ahead).with(Color::Cyan)
            )
        } else {
            format!(
                "{}{} behind",
                "Sync: ".with(Color::DarkGrey),
                format!("↓{}", self.behind).with(Color::Yellow)
            )
        };

        vec![sync_info]
    }

    fn has_content(&self) -> bool {
        self.ahead > 0 || self.behind > 0
    }
}

impl DetailViewFormat for RepoWorkingStatus {
    fn format_for_detail(&self) -> Vec<String> {
        let label = "Status: ".with(Color::DarkGrey);

        let status_text = if self.conflicts > 0 {
            let content = format!("CONFLICT (conflicts: {})", self.conflicts)
                .with(Color::Red)
                .bold();
            format!("{}{}", label, content)
        } else if self.is_dirty {
            let content = format!(
                "DIRTY (staged: {}, modified: {}, untracked: {})",
                self.staged, self.modified, self.untracked
            )
            .with(Color::Yellow)
            .bold();
            format!("{}{}", label, content)
        } else {
            let content = "CLEAN".with(Color::Green).bold();
            format!("{}{}", label, content)
        };

        vec![status_text]
    }
}

impl DetailViewFormat for RepoRemoteInfo {
    fn format_for_detail(&self) -> Vec<String> {
        if let Some(ref url) = self.url {
            vec![format!(
                "{}{}",
                "Remote: ".with(Color::DarkGrey),
                url.as_str().with(Color::Blue)
            )]
        } else {
            vec![]
        }
    }

    fn has_content(&self) -> bool {
        self.url.is_some()
    }
}

impl DetailViewFormat for RepoCommitInfo {
    fn format_for_detail(&self) -> Vec<String> {
        let mut lines = Vec::new();

        if let Some(ref msg) = self.message {
            lines.push(format!(
                "{}{}",
                "Commit: ".with(Color::DarkGrey),
                msg.as_str().with(Color::White)
            ));
            if let Some(ref author) = self.author {
                lines.push(format!(
                    "{}{}",
                    "Author: ".with(Color::DarkGrey),
                    author.as_str().with(Color::White)
                ));
            }
        }

        lines
    }

    fn has_content(&self) -> bool {
        self.message.is_some()
    }
}

impl DetailViewFormat for RepoStashInfo {
    fn format_for_detail(&self) -> Vec<String> {
        if self.count > 0 {
            vec![format!(
                "{}{}",
                "Stashes: ".with(Color::DarkGrey),
                self.count.to_string().with(Color::Magenta)
            )]
        } else {
            vec![]
        }
    }

    fn has_content(&self) -> bool {
        self.count > 0
    }
}

impl DetailViewFormat for RepoFileChanges {
    fn format_for_detail(&self) -> Vec<String> {
        if self.changes.is_empty() {
            return vec![];
        }

        let mut lines = vec!["Files:".with(Color::DarkGrey).to_string()];
        for change in &self.changes {
            let (marker, color) = match change.status {
                FileChangeStatus::Staged => ("[S]", Color::Green),
                FileChangeStatus::Modified => ("[M]", Color::Yellow),
                FileChangeStatus::Untracked => ("[U]", Color::Cyan),
                FileChangeStatus::Conflicted => ("[C]", Color::Red),
            };
            lines.push(format!("  {} {}", marker.with(color).bold(), change.path));
        }

        lines
    }

    fn has_content(&self) -> bool {
        !self.changes.is_empty()
    }
}
