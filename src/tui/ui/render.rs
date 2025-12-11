use ratatui::Frame;

use crate::tui::state::AppState;
use crate::tui::ui::keyhint_bar::render_keyhint_bar;
use crate::tui::ui::layout::create_layout;
use crate::tui::ui::repo_detail::render_repository_details;
use crate::tui::ui::repo_list::render_repository_list;

/// Render the TUI interface frame
pub fn render_ui(f: &mut Frame, state: &AppState) {
    let snapshot = state.get_render_snapshot();
    let colors = &state.colors;

    let (main_chunks, content_chunks) = create_layout(f, snapshot.is_detail_view);
    if !snapshot.is_detail_view {
        render_repository_list(f, &snapshot, &content_chunks, colors);
    }
    render_repository_details(f, &snapshot, &content_chunks, colors);
    render_keyhint_bar(f, &snapshot, colors, &main_chunks);
}
