//! Application configuration structures

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, warn};

use crate::cli::CliArgs;

use super::{KeyBindings, Theme};

/// Non-hidden directories to exclude from scanning
/// We ignore hidden directories (starting with .) by default in the scanner
const EXCLUDE_DIR_PATTERN: &[&str] = &[
    "node_modules",
    "target",
    "venv",
    "build",
    "site",
    "out",
    "dist",
    "bin",
    "obj",
    "Debug",
    "Release",
    "cache",
    "tmp",
    "temp",
    "log",
    "logs",
    "*log",
    "*logs",
    // MacOS specific
    "Library",
    "Applications",
    // Windows specific
    "AppData",
];

/// Application configuration (all settings needed at runtime)
#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    pub main: MainConfig,
    pub ui: UIConfig,
    pub internal: InternalConfig,
}

/// Main section of the configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainConfig {
    /// Directories to scan for repositories
    pub scan_dirs: Vec<String>,
    /// Maximum scan depth (0 means unlimited)
    pub max_depth: usize,
}

/// UI section of the configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UIConfig {
    /// TUI theme
    pub theme: Theme,
    /// Key bindings
    #[serde(default)]
    pub keybindings: KeyBindings,
}

/// Internal configuration (not user-configurable)
#[derive(Debug, Clone)]
pub struct InternalConfig {
    /// Directories to exclude from scanning
    pub exclude_dirs: Vec<String>,
    /// UI refresh interval in milliseconds
    pub refresh_interval: u64,
    /// Path to file where current working directory should be written on exit
    pub cwd_file: Option<String>,
}

impl Default for MainConfig {
    fn default() -> Self {
        Self {
            scan_dirs: vec![dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| {
                    debug!("Could not determine home directory, using current directory as default scan directory");
                    ".".to_string()
                })],
            max_depth: 5,
        }
    }
}

impl Default for InternalConfig {
    fn default() -> Self {
        Self {
            exclude_dirs: EXCLUDE_DIR_PATTERN.iter().map(|s| s.to_string()).collect(),
            refresh_interval: 100,
            cwd_file: None,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct AppConfigUserFields {
    main: MainConfig,
    ui: UIConfig,
}

impl AppConfig {
    /// Create app configuration with layered priority system:
    /// CLI args (highest) -> Config file -> Default values (lowest)
    pub fn from_layers(cli_args: &CliArgs) -> Self {
        let mut config = Self::default();
        if let Some(file_config) = Self::load_from_file(cli_args.config.as_deref()) {
            config.merge_file_config(file_config);
        }
        config.apply_cli_overrides(cli_args);

        debug!("Final scan directories: {:?}", config.main.scan_dirs);

        config
    }

    /// Get list of paths to search for configuration file (in priority order)
    ///
    /// Search order:
    /// 1. CLI --config argument (highest priority)
    /// 2. $REPONEST_CONFIG (environment variable)
    /// - Linux:
    ///   3. $XDG_CONFIG_HOME/reponest/config.toml
    ///   4. ~/.config/reponest/config.toml
    /// - macOS:
    ///   3. ~/Library/Application Support/reponest/config.toml
    ///   4. ~/.config/reponest/config.toml
    /// - Windows:
    ///   3. %APPDATA%\reponest\config.toml
    ///   4. ~/.config/reponest/config.toml
    fn get_search_paths(cli_config_path: Option<&str>) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Check for CLI --config argument first (highest priority)
        if let Some(config_path) = cli_config_path {
            let expanded_path = PathBuf::from(expand_tilde_in_path(config_path));
            debug!("Using config path from CLI argument: {:?}", expanded_path);
            paths.push(expanded_path);
        }

        // Check for REPONEST_CONFIG environment variable
        if let Ok(config_path) = std::env::var("REPONEST_CONFIG") {
            let expanded_path = PathBuf::from(expand_tilde_in_path(&config_path));
            debug!(
                "Using config path from REPONEST_CONFIG: {:?}",
                expanded_path
            );
            paths.push(expanded_path);
        }

        if let Some(dir) = dirs::config_dir() {
            paths.push(dir.join("reponest").join("config.toml"));
        }

        if let Some(dir) = dirs::home_dir() {
            let fallback = dir.join(".config").join("reponest").join("config.toml");
            if !paths.contains(&fallback) {
                paths.push(fallback);
            }
        }

        paths
    }

    /// Load user configuration from file, return None if file does not exist
    fn load_from_file(cli_config_path: Option<&str>) -> Option<AppConfigUserFields> {
        let config_paths = Self::get_search_paths(cli_config_path);
        debug!("Searching for config file in paths: {:?}", config_paths);

        for config_path in &config_paths {
            if config_path.exists() {
                debug!("Loading config from: {:?}", config_path);
                match fs::read_to_string(config_path) {
                    Ok(content) => match toml::from_str::<AppConfigUserFields>(&content) {
                        Ok(config) => {
                            debug!("Successfully loaded config from file");
                            return Some(config);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to parse config file at {:?}: {}. Using defaults",
                                config_path, e
                            );
                            return None;
                        }
                    },
                    Err(e) => {
                        warn!(
                            "Failed to read config file at {:?}: {}. Using defaults",
                            config_path, e
                        );
                        return None;
                    }
                }
            }
        }

        debug!("No config file found in search paths: {:?}", config_paths);
        None
    }

    /// Merge user configuration loaded from file
    fn merge_file_config(&mut self, mut file_config: AppConfigUserFields) {
        // Expand ~ in scan_dirs paths
        file_config.main.scan_dirs = file_config
            .main
            .scan_dirs
            .iter()
            .map(|p| expand_tilde_in_path(p))
            .collect();

        self.main = file_config.main;
        self.ui = file_config.ui;
    }

    /// Apply CLI argument overrides to configuration
    fn apply_cli_overrides(&mut self, args: &CliArgs) {
        if let Some(ref path) = args.path {
            debug!("CLI override: scan_dirs = [{}]", path);
            self.main.scan_dirs = vec![path.clone()];
        }

        if let Some(depth) = args.max_depth {
            debug!("CLI override: max_depth = {}", depth);
            self.main.max_depth = depth;
        }

        if let Some(ref theme_str) = args.theme {
            match theme_str.parse::<Theme>() {
                Ok(theme) => {
                    debug!("CLI override: theme = {}", theme);
                    self.ui.theme = theme;
                }
                Err(e) => {
                    warn!("Invalid theme '{}': {}. Using default theme.", theme_str, e);
                }
            }
        }

        if let Some(ref cwd_file) = args.cwd_file {
            debug!("CLI override: cwd_file = {}", cwd_file);
            self.internal.cwd_file = Some(cwd_file.clone());
        }
    }

    /// Print user-configurable fields in JSON format
    pub fn print(&self) {
        let user_fields = AppConfigUserFields {
            main: self.main.clone(),
            ui: self.ui.clone(),
        };
        match serde_json::to_string_pretty(&user_fields) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Failed to serialize configuration: {}", e),
        }
    }
}

/// Expand ~ in path to home directory
fn expand_tilde_in_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return path.replacen("~", &home.to_string_lossy(), 1);
        }
    } else if path == "~"
        && let Some(home) = dirs::home_dir()
    {
        return home.to_string_lossy().to_string();
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_expand_tilde_in_path() {
        let home = dirs::home_dir().unwrap();
        let home_str = home.to_string_lossy();

        // Test ~/path
        let result = expand_tilde_in_path("~/test/path");
        assert!(result.starts_with(&*home_str));
        assert!(result.ends_with("test/path"));

        // Test ~
        let result = expand_tilde_in_path("~");
        assert_eq!(result, home_str);

        // Test no tilde
        let result = expand_tilde_in_path("/absolute/path");
        assert_eq!(result, "/absolute/path");
    }

    #[test]
    fn test_cli_config_priority() {
        // Use platform-appropriate paths for testing
        #[cfg(target_os = "windows")]
        let (custom_path, env_path, cli_path) = (
            "C:\\custom\\config.toml",
            "C:\\env\\config.toml",
            "C:\\cli\\config.toml",
        );
        #[cfg(not(target_os = "windows"))]
        let (custom_path, env_path, cli_path) = (
            "/custom/config.toml",
            "/env/config.toml",
            "/cli/config.toml",
        );

        // Test that CLI --config has highest priority
        let paths = AppConfig::get_search_paths(Some(custom_path));
        assert_eq!(paths[0], PathBuf::from(custom_path));

        // Test with tilde expansion in CLI config (Unix/macOS only)
        #[cfg(not(target_os = "windows"))]
        {
            let paths = AppConfig::get_search_paths(Some("~/my-config.toml"));
            assert!(!paths[0].to_string_lossy().contains('~'));
            assert!(paths[0].to_string_lossy().contains("my-config.toml"));
        }

        // Test that CLI config has highest priority
        // We verify CLI is at index 0, and path list contains multiple entries
        let original = env::var("REPONEST_CONFIG").ok();
        // SAFETY: Safe in tests as we restore the value and tests run isolated
        unsafe {
            env::set_var("REPONEST_CONFIG", env_path);
        }

        let paths = AppConfig::get_search_paths(Some(cli_path));

        // CLI path should be first
        assert_eq!(paths[0], PathBuf::from(cli_path));

        // Should have at least 2 paths (CLI + env or system paths)
        assert!(
            paths.len() >= 2,
            "Expected at least 2 paths (CLI + env/system), got {} paths: {:?}",
            paths.len(),
            paths
        );

        // Environment path should be second (if no system defaults interfere)
        // or at least present in the list
        if paths.len() >= 2 {
            // Check if second path is env path OR env path exists somewhere in list
            let env_pathbuf = PathBuf::from(env_path);
            assert!(
                paths[1] == env_pathbuf || paths.contains(&env_pathbuf),
                "Expected env path {:?} at index 1 or in list, got paths: {:?}",
                env_pathbuf,
                paths
            );
        }

        // Restore original value
        // SAFETY: Safe in tests as this restores the original state
        unsafe {
            match original {
                Some(val) => env::set_var("REPONEST_CONFIG", val),
                None => env::remove_var("REPONEST_CONFIG"),
            }
        }
    }

    #[test]
    fn test_config_env_var() {
        // Save original value
        let original = env::var("REPONEST_CONFIG").ok();

        // Use platform-appropriate path for testing
        #[cfg(target_os = "windows")]
        let test_path = "C:\\tmp\\test-config.toml";
        #[cfg(not(target_os = "windows"))]
        let test_path = "/tmp/test-config.toml";

        // Test with custom config path
        // SAFETY: Safe in tests as we restore the value and tests run isolated
        unsafe {
            env::set_var("REPONEST_CONFIG", test_path);
        }
        let paths = AppConfig::get_search_paths(None);
        assert_eq!(paths[0], PathBuf::from(test_path));

        // Test with tilde expansion (Unix/macOS only)
        #[cfg(not(target_os = "windows"))]
        {
            unsafe {
                env::set_var("REPONEST_CONFIG", "~/my-config.toml");
            }
            let paths = AppConfig::get_search_paths(None);
            assert!(!paths[0].to_string_lossy().contains('~'));
            assert!(paths[0].to_string_lossy().contains("my-config.toml"));
        }

        // Restore original value
        // SAFETY: Safe in tests as this restores the original state
        unsafe {
            match original {
                Some(val) => env::set_var("REPONEST_CONFIG", val),
                None => env::remove_var("REPONEST_CONFIG"),
            }
        }
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(!config.main.scan_dirs.is_empty());
        assert_eq!(config.main.max_depth, 5);
        assert!(!config.internal.exclude_dirs.is_empty());
    }
}
