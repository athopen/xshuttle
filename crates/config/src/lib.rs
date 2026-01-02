use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const DEFAULT_CONFIG: &str = include_str!("../../../assets/default.json");

#[derive(Debug, Clone, Deserialize)]
pub struct Action {
    pub name: String,
    pub cmd: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Entry {
    Action(Action),
    Submenu(HashMap<String, Vec<Entry>>),
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_terminal")]
    pub terminal: String,

    #[serde(default = "default_editor")]
    pub editor: String,

    #[serde(default)]
    pub actions: Vec<Entry>,
}

fn default_terminal() -> String {
    "default".to_string()
}

fn default_editor() -> String {
    "default".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            terminal: default_terminal(),
            editor: default_editor(),
            actions: Vec::new(),
        }
    }
}

impl Config {
    /// Returns the default config file path (~/.xshuttle.json)
    pub fn config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".xshuttle.json"))
    }

    /// Ensures the config file exists, creating a default one if missing.
    /// Returns the path to the config file.
    pub fn ensure_config_exists() -> io::Result<PathBuf> {
        let path = Self::config_path().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Could not determine home directory",
            )
        })?;

        if !path.exists() {
            fs::write(&path, DEFAULT_CONFIG)?;
            eprintln!("Created default config at {}", path.display());
        }

        Ok(path)
    }

    /// Load config from a string (useful for testing)
    pub fn load_from_str(s: &str) -> Self {
        serde_json::from_str(s).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to parse config: {}", e);
            Self::default()
        })
    }

    /// Load config from a specific path (useful for testing with tempfile)
    pub fn load_from_path(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(path) {
            Ok(contents) => Self::load_from_str(&contents),
            Err(e) => {
                eprintln!("Warning: Failed to read {}: {}", path.display(), e);
                Self::default()
            }
        }
    }

    /// Load config from the default path (~/.xshuttle.json)
    pub fn load() -> Self {
        match Self::config_path() {
            Some(path) => Self::load_from_path(&path),
            None => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.terminal, "default");
    }

    #[test]
    fn test_load_from_str_empty() {
        let config = Config::load_from_str("{}");
        assert_eq!(config.terminal, "default");
    }

    #[test]
    fn test_load_from_str_with_terminal() {
        let config = Config::load_from_str(r#"{"terminal": "kitty"}"#);
        assert_eq!(config.terminal, "kitty");
    }

    #[test]
    fn test_load_from_str_ignores_unknown_fields() {
        let config = Config::load_from_str(r#"{"terminal": "alacritty", "unknown": true}"#);
        assert_eq!(config.terminal, "alacritty");
    }

    #[test]
    fn test_load_from_str_invalid_json_returns_default() {
        let config = Config::load_from_str("not valid json");
        assert_eq!(config.terminal, "default");
    }

    #[test]
    fn test_load_from_path_missing_file_returns_default() {
        let config = Config::load_from_path(Path::new("/nonexistent/path/config.json"));
        assert_eq!(config.terminal, "default");
    }

    #[test]
    fn test_flat_actions() {
        let config =
            Config::load_from_str(r#"{"actions": [{"name": "Test", "cmd": "echo hello"}]}"#);
        assert_eq!(config.actions.len(), 1);
        match &config.actions[0] {
            Entry::Action(a) => {
                assert_eq!(a.name, "Test");
                assert_eq!(a.cmd, "echo hello");
            }
            _ => panic!("Expected Action"),
        }
    }

    #[test]
    fn test_nested_actions() {
        let config = Config::load_from_str(
            r#"{
                "actions": [
                    {"name": "Top Level", "cmd": "echo top"},
                    {"Production": [
                        {"name": "Server 1", "cmd": "ssh server1"}
                    ]}
                ]
            }"#,
        );
        assert_eq!(config.actions.len(), 2);
        match &config.actions[0] {
            Entry::Action(a) => assert_eq!(a.name, "Top Level"),
            _ => panic!("Expected Action"),
        }
        match &config.actions[1] {
            Entry::Submenu(map) => {
                assert!(map.contains_key("Production"));
                assert_eq!(map["Production"].len(), 1);
            }
            _ => panic!("Expected Submenu"),
        }
    }

    #[test]
    fn test_deeply_nested_actions() {
        let config = Config::load_from_str(
            r#"{
                "actions": [
                    {"Level1": [
                        {"Level2": [
                            {"name": "Deep", "cmd": "echo deep"}
                        ]}
                    ]}
                ]
            }"#,
        );
        assert_eq!(config.actions.len(), 1);
        match &config.actions[0] {
            Entry::Submenu(l1) => match &l1["Level1"][0] {
                Entry::Submenu(l2) => match &l2["Level2"][0] {
                    Entry::Action(a) => assert_eq!(a.name, "Deep"),
                    _ => panic!("Expected Action at level 3"),
                },
                _ => panic!("Expected Submenu at level 2"),
            },
            _ => panic!("Expected Submenu at level 1"),
        }
    }
}
