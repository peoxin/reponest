use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io;
use std::process::Command;
use std::time::Duration;

use crate::tui::state::AppState;

/// Handle input events with polling, returns true if should exit
pub async fn handle_input_events(state: &AppState) -> io::Result<bool> {
    // Poll for input events with refresh interval timeout
    if event::poll(Duration::from_millis(
        state.config.internal.refresh_interval,
    ))? && let Event::Key(key) = event::read()?
        && key.kind == KeyEventKind::Press
    {
        return handle_key_event(key.code, state).await;
    }
    Ok(false) // continue running
}

/// Convert KeyCode to string for matching
fn keycode_to_string(key: KeyCode) -> String {
    match key {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        _ => String::new(),
    }
}

/// Handle keyboard input events, returns true if should exit
async fn handle_key_event(key_code: KeyCode, state: &AppState) -> io::Result<bool> {
    let key_str = keycode_to_string(key_code);
    if key_str.is_empty() {
        return Ok(false);
    }

    let kb = &state.config.ui.keybindings;

    if kb.matches("quit", &key_str) {
        return Ok(true);
    }

    if kb.matches("cd", &key_str) {
        return handle_cd_to_repo(state).await;
    }

    if kb.matches("back", &key_str) {
        handle_escape(state).await;
    } else if kb.matches("details", &key_str) {
        handle_enter(state).await;
    } else if kb.matches("move_down", &key_str) {
        handle_move_down(state).await;
    } else if kb.matches("move_up", &key_str) {
        handle_move_up(state).await;
    } else if kb.matches("open", &key_str) {
        handle_open_in_file_manager(state).await;
    }

    Ok(false)
}

/// Handle escape action
async fn handle_escape(state: &AppState) {
    let is_detail = state.is_detail_view().await;
    if is_detail {
        state.set_detail_view(false).await;
    }
}

/// Handle enter action
async fn handle_enter(state: &AppState) {
    let is_detail = state.is_detail_view().await;
    if !is_detail && !state.is_repos_empty().await {
        state.set_detail_view(true).await;
    }
}

/// Handle moving down action
async fn handle_move_down(state: &AppState) {
    let is_detail = state.is_detail_view().await;
    if !is_detail {
        state.move_selection_down().await;
    }
}

/// Handle moving up action
async fn handle_move_up(state: &AppState) {
    let is_detail = state.is_detail_view().await;
    if !is_detail {
        state.move_selection_up().await;
    }
}

/// Handle opening the selected repository path in file manager
async fn handle_open_in_file_manager(state: &AppState) {
    let is_detail = state.is_detail_view().await;
    if !is_detail && let Some(path) = state.get_selected_repo_path().await {
        #[cfg(target_os = "macos")]
        let _ = Command::new("open").arg(&path).spawn();

        #[cfg(target_os = "linux")]
        let _ = Command::new("xdg-open").arg(&path).spawn();

        #[cfg(target_os = "windows")]
        let _ = Command::new("explorer").arg(&path).spawn();
    }
}

/// Handle changing directory to the selected repository (exits TUI)
async fn handle_cd_to_repo(state: &AppState) -> io::Result<bool> {
    let is_detail = state.is_detail_view().await;
    if !is_detail && let Some(path) = state.get_selected_repo_path().await {
        // Write the path to cwd_file if specified
        if let Some(cwd_file) = &state.config.internal.cwd_file {
            std::fs::write(cwd_file, path.to_string_lossy().as_bytes())?;
        }
        return Ok(true); // Exit the application
    }
    Ok(false)
}
