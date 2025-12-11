use anyhow::{Context, Result};
use crossterm::{
    ExecutableCommand, execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::config::AppConfig;
use crate::tui::input;
use crate::tui::state::AppState;
use crate::tui::task;
use crate::tui::ui;

/// Initialize terminal for TUI mode
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    terminal::enable_raw_mode().context("Failed to enable terminal raw mode")?;
    let mut stdout = std::io::stdout();
    stdout
        .execute(EnterAlternateScreen)
        .context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).context("Failed to create terminal")
}

/// Restore terminal to normal mode
fn cleanup_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    terminal::disable_raw_mode().context("Failed to disable terminal raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;
    Ok(())
}

/// Run the TUI application
pub async fn run_tui_app(cfg: AppConfig) -> Result<()> {
    let mut terminal = setup_terminal()?;

    let app_state = AppState::new(cfg.clone());
    task::spawn_scan_repo_and_get_info_task(&app_state);
    let res = run_event_loop(&mut terminal, app_state).await;

    cleanup_terminal(&mut terminal)?;
    Ok(res?)
}

/// Main event loop for UI rendering and input handling
async fn run_event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: AppState,
) -> io::Result<()> {
    loop {
        // Render the UI
        terminal.draw(|f| {
            ui::render_ui(f, &state);
        })?;

        // Handle input events
        if input::handle_input_events(&state).await? {
            return Ok(()); // exit requested
        }
    }
}
