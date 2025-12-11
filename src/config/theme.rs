//! Theme system for TUI color schemes

use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Available themes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Theme {
    #[default]
    Default,
    Dark,
    Light,
}

impl Theme {
    /// Get the color scheme for this theme
    pub fn colors(&self) -> ColorScheme {
        match self {
            Self::Default => ColorScheme::default(),
            Self::Dark => ColorScheme::dark(),
            Self::Light => ColorScheme::light(),
        }
    }
}

impl FromStr for Theme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(Self::Default),
            "dark" => Ok(Self::Dark),
            "light" => Ok(Self::Light),
            _ => Err(format!(
                "Invalid theme '{}'. Valid options: default, dark, light",
                s
            )),
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Dark => write!(f, "dark"),
            Self::Light => write!(f, "light"),
        }
    }
}

/// Color scheme for the TUI
#[derive(Debug, Clone, Copy)]
pub struct ColorScheme {
    // General UI
    pub border: Color,
    pub highlight_bg: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,

    // Status colors
    pub status_clean: Color,
    pub status_dirty: Color,
    pub status_conflict: Color,
    pub status_sync: Color,

    // Key hints
    pub key_action: Color,
    pub key_warning: Color,
    pub key_danger: Color,

    // Repository info
    pub repo_name: Color,
    pub branch_name: Color,
    pub commit_ahead: Color,
    pub commit_behind: Color,

    // Section headers
    pub section_remote: Color,
    pub section_commit: Color,
    pub section_stash: Color,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            // General UI
            border: Color::White,
            highlight_bg: Color::DarkGray,
            text_primary: Color::White,
            text_secondary: Color::Gray,
            text_muted: Color::DarkGray,

            // Status colors
            status_clean: Color::Green,
            status_dirty: Color::Yellow,
            status_conflict: Color::Red,
            status_sync: Color::Cyan,

            // Key hints
            key_action: Color::Green,
            key_warning: Color::Yellow,
            key_danger: Color::Red,

            // Repository info
            repo_name: Color::Cyan,
            branch_name: Color::Green,
            commit_ahead: Color::Cyan,
            commit_behind: Color::Yellow,

            // Section headers
            section_remote: Color::Blue,
            section_commit: Color::Magenta,
            section_stash: Color::Magenta,
        }
    }
}

impl ColorScheme {
    /// Dark theme
    pub fn dark() -> Self {
        Self {
            // General UI
            border: Color::Rgb(80, 80, 80),
            highlight_bg: Color::Rgb(40, 45, 50),
            text_primary: Color::Rgb(220, 225, 230),
            text_secondary: Color::Rgb(150, 155, 160),
            text_muted: Color::Rgb(90, 95, 100),

            // Status colors
            status_clean: Color::Rgb(80, 200, 120), // Soft green
            status_dirty: Color::Rgb(230, 190, 90), // Warm yellow
            status_conflict: Color::Rgb(240, 90, 90), // Bright red
            status_sync: Color::Rgb(90, 180, 230),  // Sky blue

            // Key hints
            key_action: Color::Rgb(100, 220, 150), // Bright green
            key_warning: Color::Rgb(250, 200, 100), // Bright yellow
            key_danger: Color::Rgb(250, 100, 100), // Bright red

            // Repository info
            repo_name: Color::Rgb(100, 200, 240), // Bright cyan
            branch_name: Color::Rgb(120, 230, 150), // Bright green
            commit_ahead: Color::Rgb(110, 200, 240), // Cyan
            commit_behind: Color::Rgb(240, 190, 100), // Amber

            // Section headers
            section_remote: Color::Rgb(130, 170, 240), // Light blue
            section_commit: Color::Rgb(230, 150, 230), // Pink/magenta
            section_stash: Color::Rgb(210, 140, 230),  // Purple
        }
    }

    /// Light theme
    pub fn light() -> Self {
        Self {
            // General UI
            border: Color::Rgb(180, 185, 190),
            highlight_bg: Color::Rgb(235, 240, 245), // Very light blue-gray
            text_primary: Color::Rgb(20, 20, 25),    // Near black
            text_secondary: Color::Rgb(70, 75, 80),  // Dark gray
            text_muted: Color::Rgb(140, 145, 150),   // Medium gray

            // Status colors
            status_clean: Color::Rgb(0, 130, 50),  // Rich green
            status_dirty: Color::Rgb(200, 120, 0), // Deep orange
            status_conflict: Color::Rgb(200, 20, 20), // Strong red
            status_sync: Color::Rgb(0, 100, 180),  // Deep blue

            // Key hints
            key_action: Color::Rgb(0, 140, 70),   // Rich green
            key_warning: Color::Rgb(210, 130, 0), // Deep amber
            key_danger: Color::Rgb(220, 20, 20),  // Bold red

            // Repository info
            repo_name: Color::Rgb(0, 90, 180),      // Deep blue
            branch_name: Color::Rgb(0, 130, 60),    // Forest green
            commit_ahead: Color::Rgb(0, 110, 200),  // Bright blue
            commit_behind: Color::Rgb(190, 100, 0), // Orange-brown

            // Section headers
            section_remote: Color::Rgb(30, 80, 200), // Royal blue
            section_commit: Color::Rgb(170, 50, 170), // Rich magenta
            section_stash: Color::Rgb(150, 50, 180), // Rich purple
        }
    }
}
