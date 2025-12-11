use ratatui::{
    Frame,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::config::ColorScheme;
use crate::core::repo_info::{
    FileChangeStatus, RepoBasicInfo, RepoCommitInfo, RepoFileChanges, RepoInfo, RepoRemoteInfo,
    RepoStashInfo, RepoSyncStatus, RepoWorkingStatus,
};
use crate::tui::state::RenderSnapshot;

/// Render the repository details section
pub fn render_repository_details(
    f: &mut Frame,
    snapshot: &RenderSnapshot,
    content_chunks: &[ratatui::layout::Rect],
    colors: &ColorScheme,
) {
    let detail_text = match snapshot.repos.get(snapshot.selected_index) {
        Some(repo) => build_repo_detail_lines(repo, snapshot.is_detail_view, colors),
        None => vec![
            Line::from(""),
            Line::from(Span::styled(
                "No repository selected",
                Style::default().fg(colors.text_muted),
            )),
        ],
    };

    let detail_title = match snapshot.is_detail_view {
        true => "Repo Details (ESC to exit)",
        false => "Repo Info",
    };
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .title(detail_title)
        .border_style(Style::default().fg(colors.border));

    let detail_paragraph = Paragraph::new(detail_text).block(detail_block);
    let detail_chunk_idx = if snapshot.is_detail_view { 0 } else { 1 };
    f.render_widget(detail_paragraph, content_chunks[detail_chunk_idx]);
}

/// Build detailed information lines for a repository in TUI
fn build_repo_detail_lines<'a>(
    repo: &'a RepoInfo,
    is_detail_view: bool,
    colors: &'a ColorScheme,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    lines.extend(repo.basic.render_lines(colors));
    lines.extend(repo.sync.render_lines(colors));
    lines.extend(repo.working.render_lines(colors));
    lines.extend(repo.stash.render_lines(colors));
    lines.extend(repo.remote.render_lines(colors));
    lines.extend(repo.commit.render_lines(colors));

    if is_detail_view {
        lines.extend(repo.files.render_lines(colors));
    }

    lines
}

/// Trait for rendering detail sections in TUI
trait RenderDetail {
    fn render_lines(&self, colors: &ColorScheme) -> Vec<Line<'_>>;
}

impl RenderDetail for RepoBasicInfo {
    fn render_lines(&self, colors: &ColorScheme) -> Vec<Line<'_>> {
        vec![
            Line::from(vec![Span::styled(
                self.name.clone(),
                Style::default()
                    .fg(colors.repo_name)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                self.path.display().to_string(),
                Style::default().fg(colors.text_muted),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Branch: ", Style::default().fg(colors.text_secondary)),
                Span::styled(self.branch.clone(), Style::default().fg(colors.branch_name)),
            ]),
        ]
    }
}

impl RenderDetail for RepoSyncStatus {
    fn render_lines(&self, colors: &ColorScheme) -> Vec<Line<'_>> {
        if self.ahead == 0 && self.behind == 0 {
            return vec![];
        }

        let mut sync_spans = vec![Span::styled(
            "Sync: ",
            Style::default().fg(colors.text_secondary),
        )];

        if self.ahead > 0 && self.behind > 0 {
            sync_spans.push(Span::styled(
                format!("↑{} ", self.ahead),
                Style::default()
                    .fg(colors.commit_ahead)
                    .add_modifier(Modifier::BOLD),
            ));
            sync_spans.push(Span::styled(
                format!("↓{}", self.behind),
                Style::default()
                    .fg(colors.commit_behind)
                    .add_modifier(Modifier::BOLD),
            ));
        } else if self.ahead > 0 {
            sync_spans.push(Span::styled(
                format!("↑{} ahead", self.ahead),
                Style::default()
                    .fg(colors.commit_ahead)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            sync_spans.push(Span::styled(
                format!("↓{} behind", self.behind),
                Style::default()
                    .fg(colors.commit_behind)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        vec![Line::from(sync_spans)]
    }
}

impl RenderDetail for RepoWorkingStatus {
    fn render_lines(&self, colors: &ColorScheme) -> Vec<Line<'_>> {
        let mut lines = vec![Line::from("")];

        let (prefix, status_text, color) = if self.conflicts > 0 {
            ("[!] ", "CONFLICT", colors.status_conflict)
        } else if self.is_dirty {
            ("[~] ", "DIRTY", colors.status_dirty)
        } else {
            ("[✓] ", "CLEAN", colors.status_clean)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(color)),
            Span::styled(
                status_text.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]));

        // Add change statistics
        if self.staged > 0 {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled("● ", Style::default().fg(colors.status_clean)),
                Span::styled(
                    format!("{} staged", self.staged),
                    Style::default().fg(colors.status_clean),
                ),
            ]));
        }
        if self.modified > 0 {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled("● ", Style::default().fg(colors.status_dirty)),
                Span::styled(
                    format!("{} modified", self.modified),
                    Style::default().fg(colors.status_dirty),
                ),
            ]));
        }
        if self.untracked > 0 {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled("● ", Style::default().fg(colors.status_sync)),
                Span::styled(
                    format!("{} untracked", self.untracked),
                    Style::default().fg(colors.status_sync),
                ),
            ]));
        }
        if self.conflicts > 0 {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled("● ", Style::default().fg(colors.status_conflict)),
                Span::styled(
                    format!("{} conflicts", self.conflicts),
                    Style::default()
                        .fg(colors.status_conflict)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        lines
    }
}

impl RenderDetail for RepoStashInfo {
    fn render_lines(&self, colors: &ColorScheme) -> Vec<Line<'_>> {
        if self.count == 0 {
            return vec![];
        }

        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Stashes: ", Style::default().fg(colors.section_stash)),
                Span::styled(
                    self.count.to_string(),
                    Style::default().fg(colors.section_stash),
                ),
            ]),
        ]
    }
}

impl RenderDetail for RepoRemoteInfo {
    fn render_lines(&self, colors: &ColorScheme) -> Vec<Line<'_>> {
        let Some(ref url) = self.url else {
            return vec![];
        };

        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Remote:",
                Style::default().fg(colors.section_remote),
            )]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(url.clone(), Style::default().fg(colors.text_secondary)),
            ]),
        ]
    }
}

impl RenderDetail for RepoCommitInfo {
    fn render_lines(&self, colors: &ColorScheme) -> Vec<Line<'_>> {
        let Some(ref message) = self.message else {
            return vec![];
        };

        let mut lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Last Commit:",
                Style::default().fg(colors.section_commit),
            )]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(message.clone(), Style::default().fg(colors.text_primary)),
            ]),
        ];

        if let Some(ref author) = self.author {
            lines.push(Line::from(vec![
                Span::raw("  by "),
                Span::styled(author.clone(), Style::default().fg(colors.text_secondary)),
            ]));
        }

        lines
    }
}

impl RenderDetail for RepoFileChanges {
    fn render_lines(&self, colors: &ColorScheme) -> Vec<Line<'_>> {
        if self.changes.is_empty() {
            return vec![];
        }

        let mut lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "File Changes:",
                Style::default()
                    .fg(colors.status_dirty)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];

        for change in &self.changes {
            let (symbol, color) = match change.status {
                FileChangeStatus::Staged => ("● ", colors.status_clean),
                FileChangeStatus::Modified => ("● ", colors.status_dirty),
                FileChangeStatus::Untracked => ("● ", colors.status_sync),
                FileChangeStatus::Conflicted => ("● ", colors.status_conflict),
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(symbol, Style::default().fg(color)),
                Span::styled(
                    change.path.clone(),
                    Style::default().fg(colors.text_primary),
                ),
            ]));
        }

        lines
    }
}
