use crate::schema::{ConfigError, ValidationResult, validate};
use crate::types::Config;
use serde_json::Value;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

const DEFAULT_JSON: &str = include_str!("../../../assets/xshuttle.default.json");

/// Error type for config loading operations.
#[derive(Debug)]
pub enum LoadError {
    NoHomeDir,
    Io(io::Error),
    Config(ConfigError),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::NoHomeDir => write!(f, "could not determine home directory"),
            LoadError::Io(e) => write!(f, "failed to read config: {e}"),
            LoadError::Config(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::Io(e) => Some(e),
            LoadError::Config(e) => Some(e),
            LoadError::NoHomeDir => None,
        }
    }
}

impl From<io::Error> for LoadError {
    fn from(e: io::Error) -> Self {
        LoadError::Io(e)
    }
}

impl From<ConfigError> for LoadError {
    fn from(e: ConfigError) -> Self {
        LoadError::Config(e)
    }
}

/// Returns the default config file path (~/.xshuttle.json).
pub fn config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".xshuttle.json"))
}

/// Ensures the config file exists, creating a default one if missing.
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined or if
/// writing the default config file fails.
pub fn ensure_config_exists() -> Result<PathBuf, LoadError> {
    let path = config_path().ok_or(LoadError::NoHomeDir)?;

    if !path.exists() {
        fs::write(&path, DEFAULT_JSON)?;
        eprintln!("Created default config at {}", path.display());
    }

    Ok(path)
}

/// Loads config from a string.
///
/// # Errors
///
/// Returns an error if the JSON is invalid or fails schema validation.
pub fn load_from_str(s: &str) -> Result<Config, ConfigError> {
    s.parse()
}

/// Loads config from a specific path.
///
/// # Errors
///
/// Returns an error if the file cannot be read or the config is invalid.
pub fn load_from_path(path: &Path) -> Result<Config, LoadError> {
    let contents = fs::read_to_string(path)?;
    Ok(load_from_str(&contents)?)
}

/// Loads config from the default path (~/.xshuttle.json).
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined or the config is invalid.
pub fn load() -> Result<Config, LoadError> {
    let path = config_path().ok_or(LoadError::NoHomeDir)?;

    if !path.exists() {
        return Ok(Config::default());
    }

    load_from_path(&path)
}

impl FromStr for Config {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: Value = serde_json::from_str(s).map_err(ConfigError::InvalidJson)?;

        if let ValidationResult::Invalid(errors) = validate(&value) {
            return Err(ConfigError::ValidationFailed(errors));
        }

        serde_json::from_value(value).map_err(ConfigError::InvalidJson)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Entry;

    #[test]
    fn test_load_from_str_empty() {
        let config = load_from_str("{}").unwrap();
        assert_eq!(config.terminal, "default");
    }

    #[test]
    fn test_load_from_str_with_terminal() {
        let config = load_from_str(r#"{"terminal": "kitty"}"#).unwrap();
        assert_eq!(config.terminal, "kitty");
    }

    #[test]
    fn test_load_from_str_rejects_unknown_fields() {
        let result = load_from_str(r#"{"terminal": "alacritty", "unknown": true}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_str_invalid_json() {
        let result = load_from_str("not valid json");
        assert!(matches!(result, Err(ConfigError::InvalidJson(_))));
    }

    #[test]
    fn test_load_from_path_missing_file() {
        let result = load_from_path(Path::new("/nonexistent/path/config.json"));
        assert!(matches!(result, Err(LoadError::Io(_))));
    }

    #[test]
    fn test_flat_actions() {
        let config =
            load_from_str(r#"{"actions": [{"name": "Test", "cmd": "echo hello"}]}"#).unwrap();
        assert_eq!(config.entries.len(), 1);
        match &config.entries[0] {
            Entry::Action(c) => {
                assert_eq!(c.name, "Test");
                assert_eq!(c.cmd, "echo hello");
            }
            Entry::Group(_) => panic!("Expected Action"),
        }
    }

    #[test]
    fn test_nested_actions() {
        let config = load_from_str(
            r#"{
                "actions": [
                    {"name": "Top Level", "cmd": "echo top"},
                    {"Production": [
                        {"name": "Server 1", "cmd": "ssh server1"}
                    ]}
                ]
            }"#,
        )
        .unwrap();
        assert_eq!(config.entries.len(), 2);
        match &config.entries[0] {
            Entry::Action(c) => assert_eq!(c.name, "Top Level"),
            Entry::Group(_) => panic!("Expected Action"),
        }
        match &config.entries[1] {
            Entry::Group(group) => {
                assert_eq!(group.name, "Production");
                assert_eq!(group.entries.len(), 1);
            }
            Entry::Action(_) => panic!("Expected Group"),
        }
    }

    #[test]
    fn test_deeply_nested_actions() {
        let config = load_from_str(
            r#"{
                "actions": [
                    {"Level1": [
                        {"Level2": [
                            {"name": "Deep", "cmd": "echo deep"}
                        ]}
                    ]}
                ]
            }"#,
        )
        .unwrap();
        assert_eq!(config.entries.len(), 1);
        match &config.entries[0] {
            Entry::Group(l1) => {
                assert_eq!(l1.name, "Level1");
                match &l1.entries[0] {
                    Entry::Group(l2) => {
                        assert_eq!(l2.name, "Level2");
                        match &l2.entries[0] {
                            Entry::Action(c) => assert_eq!(c.name, "Deep"),
                            Entry::Group(_) => panic!("Expected Action at level 3"),
                        }
                    }
                    Entry::Action(_) => panic!("Expected Group at level 2"),
                }
            }
            Entry::Action(_) => panic!("Expected Group at level 1"),
        }
    }

    #[test]
    fn test_group_with_multiple_children() {
        let config = load_from_str(
            r#"{
                "actions": [
                    {"Servers": [
                        {"name": "Server 1", "cmd": "ssh server1"},
                        {"name": "Server 2", "cmd": "ssh server2"},
                        {"name": "Server 3", "cmd": "ssh server3"}
                    ]}
                ]
            }"#,
        )
        .unwrap();
        assert_eq!(config.entries.len(), 1);
        match &config.entries[0] {
            Entry::Group(group) => {
                assert_eq!(group.name, "Servers");
                assert_eq!(group.entries.len(), 3);
            }
            Entry::Action(_) => panic!("Expected Group"),
        }
    }

    #[test]
    fn test_group_serialization_roundtrip() {
        let original = r#"{"terminal":"default","editor":"default","actions":[{"Production":[{"name":"Server","cmd":"ssh prod"}]}]}"#;
        let config: Config = serde_json::from_str(original).unwrap();
        let serialized = serde_json::to_string(&config).unwrap();

        // Parse both and compare structure
        let orig_value: Value = serde_json::from_str(original).unwrap();
        let ser_value: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(orig_value, ser_value);
    }

    #[test]
    fn test_from_str_valid() {
        let config: Config = r#"{"terminal": "kitty"}"#.parse().unwrap();
        assert_eq!(config.terminal, "kitty");
    }

    #[test]
    fn test_from_str_invalid_json() {
        let result: Result<Config, _> = "not valid json".parse();
        assert!(matches!(result, Err(ConfigError::InvalidJson(_))));
    }

    #[test]
    fn test_from_str_validation_failed() {
        let result: Result<Config, _> = r#"{"unknown": true}"#.parse();
        assert!(matches!(result, Err(ConfigError::ValidationFailed(_))));
    }
}
