use jsonschema::Validator;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

const DEFAULT_JSON: &str = include_str!("../../../assets/xshuttle.default.json");
const SCHEMA_JSON: &str = include_str!("../../../assets/xshuttle.schema.json");

/// A validation error with path and message.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "{}: {}", self.path, self.message)
        }
    }
}

/// Result of config validation.
#[derive(Debug)]
pub enum ValidationResult {
    Valid,
    Invalid(Vec<ValidationError>),
}

/// Error type for config parsing.
#[derive(Debug)]
pub enum ConfigError {
    InvalidJson(serde_json::Error),
    ValidationFailed(Vec<ValidationError>),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidJson(e) => write!(f, "invalid JSON: {e}"),
            ConfigError::ValidationFailed(errors) => {
                writeln!(f, "validation failed:")?;
                for error in errors {
                    writeln!(f, "  - {error}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Action {
    pub name: String,
    pub cmd: String,
}

/// A named group containing nested entries.
/// Serializes to/from JSON as `{"GroupName": [...]}`
#[derive(Debug, Clone)]
pub struct Group {
    pub name: String,
    pub entries: Vec<Entry>,
}

impl Serialize for Group {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&self.name, &self.entries)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Group {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct GroupVisitor;

        impl<'de> Visitor<'de> for GroupVisitor {
            type Value = Group;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with a single key (group name) and array value")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let (name, entries): (String, Vec<Entry>) = access
                    .next_entry()?
                    .ok_or_else(|| de::Error::custom("expected non-empty map for group"))?;

                // Ensure no extra keys
                if access.next_key::<String>()?.is_some() {
                    return Err(de::Error::custom(
                        "group must have exactly one key (the group name)",
                    ));
                }

                Ok(Group { name, entries })
            }
        }

        deserializer.deserialize_map(GroupVisitor)
    }
}

/// An entry in the menu - either a action or a group.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Entry {
    Action(Action),
    Group(Group),
}

#[derive(Debug, Deserialize, Serialize)]
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
    /// Returns the embedded JSON schema as a string.
    pub fn schema() -> &'static str {
        SCHEMA_JSON
    }

    /// Validate a JSON value against the config schema.
    pub fn validate_json(value: &Value) -> ValidationResult {
        let schema: Value =
            serde_json::from_str(SCHEMA_JSON).expect("embedded schema should be valid JSON");

        let validator =
            Validator::new(&schema).expect("embedded schema should be a valid JSON Schema");

        let errors: Vec<ValidationError> = validator
            .iter_errors(value)
            .map(|e| ValidationError {
                path: e.instance_path.to_string(),
                message: e.to_string(),
            })
            .collect();

        if errors.is_empty() {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid(errors)
        }
    }

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
            fs::write(&path, DEFAULT_JSON)?;
            eprintln!("Created default config at {}", path.display());
        }

        Ok(path)
    }

    /// Load config from a string (useful for testing)
    pub fn load_from_str(s: &str) -> Self {
        s.parse::<Self>().unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
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

impl FromStr for Config {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: Value = serde_json::from_str(s).map_err(ConfigError::InvalidJson)?;

        if let ValidationResult::Invalid(errors) = Self::validate_json(&value) {
            return Err(ConfigError::ValidationFailed(errors));
        }

        serde_json::from_value(value).map_err(ConfigError::InvalidJson)
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
    fn test_load_from_str_rejects_unknown_fields() {
        let config = Config::load_from_str(r#"{"terminal": "alacritty", "unknown": true}"#);
        // Unknown fields cause validation failure, returning default
        assert_eq!(config.terminal, "default");
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
            Entry::Action(c) => {
                assert_eq!(c.name, "Test");
                assert_eq!(c.cmd, "echo hello");
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
            Entry::Action(c) => assert_eq!(c.name, "Top Level"),
            _ => panic!("Expected Action"),
        }
        match &config.actions[1] {
            Entry::Group(group) => {
                assert_eq!(group.name, "Production");
                assert_eq!(group.entries.len(), 1);
            }
            _ => panic!("Expected Group"),
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
            Entry::Group(l1) => {
                assert_eq!(l1.name, "Level1");
                match &l1.entries[0] {
                    Entry::Group(l2) => {
                        assert_eq!(l2.name, "Level2");
                        match &l2.entries[0] {
                            Entry::Action(c) => assert_eq!(c.name, "Deep"),
                            _ => panic!("Expected Action at level 3"),
                        }
                    }
                    _ => panic!("Expected Group at level 2"),
                }
            }
            _ => panic!("Expected Group at level 1"),
        }
    }

    #[test]
    fn test_group_with_multiple_children() {
        let config = Config::load_from_str(
            r#"{
                "actions": [
                    {"Servers": [
                        {"name": "Server 1", "cmd": "ssh server1"},
                        {"name": "Server 2", "cmd": "ssh server2"},
                        {"name": "Server 3", "cmd": "ssh server3"}
                    ]}
                ]
            }"#,
        );
        assert_eq!(config.actions.len(), 1);
        match &config.actions[0] {
            Entry::Group(group) => {
                assert_eq!(group.name, "Servers");
                assert_eq!(group.entries.len(), 3);
            }
            _ => panic!("Expected Group"),
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

    // Schema validation tests

    #[test]
    fn test_schema_is_valid_json() {
        let schema: Value = serde_json::from_str(Config::schema()).unwrap();
        assert!(schema.is_object());
        assert!(schema.get("$schema").is_some());
    }

    #[test]
    fn test_validate_valid_config() {
        let config = r#"{"terminal": "kitty", "actions": []}"#;
        let value: Value = serde_json::from_str(config).unwrap();
        assert!(matches!(
            Config::validate_json(&value),
            ValidationResult::Valid
        ));
    }

    #[test]
    fn test_validate_invalid_action_missing_cmd() {
        let config = r#"{"actions": [{"name": "Test"}]}"#;
        let value: Value = serde_json::from_str(config).unwrap();
        match Config::validate_json(&value) {
            ValidationResult::Invalid(errors) => {
                assert!(!errors.is_empty());
            }
            ValidationResult::Valid => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_validate_invalid_type() {
        let config = r#"{"terminal": 123}"#;
        let value: Value = serde_json::from_str(config).unwrap();
        assert!(matches!(
            Config::validate_json(&value),
            ValidationResult::Invalid(_)
        ));
    }

    #[test]
    fn test_validate_unknown_field_rejected() {
        let config = r#"{"unknown_field": "value"}"#;
        let value: Value = serde_json::from_str(config).unwrap();
        assert!(matches!(
            Config::validate_json(&value),
            ValidationResult::Invalid(_)
        ));
    }

    #[test]
    fn test_validate_nested_group() {
        let config = r#"{
            "actions": [
                {"Production": [
                    {"name": "Server", "cmd": "ssh server"}
                ]}
            ]
        }"#;
        let value: Value = serde_json::from_str(config).unwrap();
        assert!(matches!(
            Config::validate_json(&value),
            ValidationResult::Valid
        ));
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
