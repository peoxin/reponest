use ratatui::{
    Frame,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::config::ColorScheme;
use crate::core::RepoInfo;
use crate::tui::state::RenderSnapshot;

/// Render the repository list on the left side
pub fn render_repository_list(
    f: &mut Frame,
    snapshot: &RenderSnapshot,
    content_chunks: &[ratatui::layout::Rect],
    colors: &ColorScheme,
) {
    let items: Vec<ListItem> = snapshot
        .repos
        .iter()
        .enumerate()
        .map(|(idx, repo)| create_repo_list_item(repo, idx, snapshot.selected_index, colors))
        .collect();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Repos ({} found)", snapshot.repos.len()))
        .border_style(Style::default().fg(colors.border));

    let list = List::new(items)
        .block(list_block)
        .highlight_style(Style::default().bg(colors.highlight_bg))
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    if !snapshot.repos.is_empty() {
        list_state.select(Some(snapshot.selected_index));
    }

    f.render_stateful_widget(list, content_chunks[0], &mut list_state);
}

/// Create a single list item for a repository
fn create_repo_list_item<'a>(
    repo: &'a RepoInfo,
    idx: usize,
    current_selected: usize,
    colors: &'a ColorScheme,
) -> ListItem<'a> {
    // Determine repo name color based on repo status
    let color = if repo.working.conflicts > 0 {
        colors.status_conflict
    } else if repo.working.is_dirty {
        colors.status_dirty
    } else if repo.sync.ahead > 0 || repo.sync.behind > 0 {
        colors.status_sync
    } else {
        colors.status_clean
    };

    // Apply bold modifier if selected
    let style = if idx == current_selected {
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(color)
    };

    let repo_name = repo.basic.name.clone();

    ListItem::new(repo_name).style(style)
}
