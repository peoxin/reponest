use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

/// Create the layout for the TUI
pub fn create_layout(
    f: &Frame,
    is_detail_view: bool,
) -> (
    std::rc::Rc<[ratatui::layout::Rect]>,
    std::rc::Rc<[ratatui::layout::Rect]>,
) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // main content area
            Constraint::Length(1), // bottom keyhint bar
        ])
        .split(f.area());

    let content_chunks = if is_detail_view {
        // Detailed view uses full width
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(main_chunks[0])
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(45), // left side repo list
                Constraint::Percentage(55), // right side details
            ])
            .split(main_chunks[0])
    };

    (main_chunks, content_chunks)
}
