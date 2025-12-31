use serde::Deserialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const DEFAULT_CONFIG: &str = include_str!("../assets/default.json");

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_terminal")]
    pub terminal: String,

    #[serde(default = "default_editor")]
    pub editor: String,
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
}
