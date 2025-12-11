use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::config::ColorScheme;
use crate::tui::state::RenderSnapshot;

/// Render keyhint bar at the bottom
pub fn render_keyhint_bar(
    f: &mut Frame,
    snapshot: &RenderSnapshot,
    colors: &ColorScheme,
    main_chunks: &[ratatui::layout::Rect],
) {
    let keyhints = if snapshot.is_detail_view {
        get_detail_keyhints(colors)
    } else {
        get_main_keyhints(colors)
    };

    let keyhint_line = build_keyhint_line(&keyhints);
    let paragraph = Paragraph::new(vec![keyhint_line]);
    f.render_widget(paragraph, main_chunks[1]);
}

/// Get keyhints for main view
fn get_main_keyhints(colors: &ColorScheme) -> Vec<KeyHint> {
    vec![
        KeyHint::new("↓/j", "Down", colors.key_action),
        KeyHint::new("↑/k", "Up", colors.key_action),
        KeyHint::new("→/l", "Details", colors.key_action),
        KeyHint::new("o", "CD", colors.key_action),
        KeyHint::new("O/Enter", "Open", colors.key_action),
        KeyHint::new("q", "Quit", colors.key_danger),
    ]
}

/// Get keyhints for detail view
fn get_detail_keyhints(colors: &ColorScheme) -> Vec<KeyHint> {
    vec![
        KeyHint::new("ESC", "Back", colors.key_warning),
        KeyHint::new("q", "Quit", colors.key_danger),
    ]
}

/// Build a single line from a list of keyhints
fn build_keyhint_line(keyhints: &[KeyHint]) -> Line<'_> {
    let mut spans = Vec::new();
    for keyhint in keyhints {
        spans.extend(keyhint.to_spans());
    }
    Line::from(spans)
}

/// Represents a single hotkey with its display and description
struct KeyHint {
    keys: &'static str,
    description: &'static str,
    color: Color,
}

impl KeyHint {
    fn new(keys: &'static str, description: &'static str, color: Color) -> Self {
        Self {
            keys,
            description,
            color,
        }
    }

    fn to_spans(&self) -> Vec<Span<'_>> {
        vec![
            Span::styled(
                format!(" {} ", self.keys),
                Style::default().fg(self.color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}  ", self.description)),
        ]
    }
}
