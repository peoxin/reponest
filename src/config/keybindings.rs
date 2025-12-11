//! Key bindings configuration

use serde::{Deserialize, Serialize};

/// Key bindings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyBindings {
    pub quit: Vec<String>,
    pub move_up: Vec<String>,
    pub move_down: Vec<String>,
    pub move_left: Vec<String>,
    pub move_right: Vec<String>,
    pub details: Vec<String>,
    pub back: Vec<String>,
    pub cd: Vec<String>,
    pub open: Vec<String>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: vec!["q".to_string()],
            move_up: vec!["k".to_string(), "Up".to_string()],
            move_down: vec!["j".to_string(), "Down".to_string()],
            move_left: vec!["h".to_string(), "Left".to_string()],
            move_right: vec!["l".to_string(), "Right".to_string()],
            details: vec!["l".to_string(), "Right".to_string()],
            back: vec!["Esc".to_string()],
            cd: vec!["o".to_string()],
            open: vec!["O".to_string(), "Enter".to_string()],
        }
    }
}

impl KeyBindings {
    /// Check if a key matches any binding for the given action
    pub fn matches(&self, action: &str, key: &str) -> bool {
        let bindings = match action {
            "quit" => &self.quit,
            "move_up" => &self.move_up,
            "move_down" => &self.move_down,
            "move_left" => &self.move_left,
            "move_right" => &self.move_right,
            "details" => &self.details,
            "back" => &self.back,
            "cd" => &self.cd,
            "open" => &self.open,
            _ => return false,
        };
        bindings.iter().any(|b| b == key)
    }
}
