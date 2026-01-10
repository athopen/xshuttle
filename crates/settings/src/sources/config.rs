use crate::error::SettingsError;
use crate::types::Entry;
use jsonschema::Validator;
use serde::Deserialize;
use serde_json::Value;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_JSON: &str = include_str!("../../../../assets/xshuttle.default.json");
const SCHEMA_JSON: &str = include_str!("../../../../assets/xshuttle.schema.json");

/// Internal configuration structure for JSON deserialization.
/// All fields are optional - defaults are applied by the Settings struct.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ConfigContent {
    pub terminal: Option<String>,
    pub editor: Option<String>,
    pub actions: Option<Vec<Entry>>,
}

// ============================================================================
// Schema Validation
// ============================================================================

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

/// Returns the embedded JSON schema as a string.
pub fn schema() -> &'static str {
    SCHEMA_JSON
}

/// Validates a JSON value against the config schema.
///
/// # Panics
///
/// Panics if the embedded schema is invalid JSON or not a valid JSON Schema.
/// This should never happen as the schema is compile-time embedded.
pub fn validate(value: &Value) -> ValidationResult {
    let schema: Value =
        serde_json::from_str(SCHEMA_JSON).expect("embedded schema should be valid JSON");

    let validator = Validator::new(&schema).expect("embedded schema should be a valid JSON Schema");

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

// ============================================================================
// Config Loading
// ============================================================================

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
pub fn ensure_config_exists() -> Result<PathBuf, SettingsError> {
    let path = config_path().ok_or(SettingsError::NoHomeDir)?;

    if !path.exists() {
        fs::write(&path, DEFAULT_JSON)?;
        eprintln!("Created default config at {}", path.display());
    }

    Ok(path)
}

/// Loads config content from a string.
///
/// Validates against the schema first, then deserializes.
///
/// # Errors
///
/// Returns an error if the JSON is invalid or fails schema validation.
pub fn load_from_str(s: &str) -> Result<ConfigContent, SettingsError> {
    let value: Value = serde_json::from_str(s).map_err(SettingsError::ConfigParse)?;

    // Validate against schema
    if let ValidationResult::Invalid(errors) = validate(&value) {
        return Err(SettingsError::ConfigValidation(errors));
    }

    // Schema is valid, deserialize
    serde_json::from_value(value).map_err(SettingsError::ConfigParse)
}

/// Loads config content from a specific path.
///
/// # Errors
///
/// Returns an error if the file cannot be read or the JSON is unparseable.
pub fn load_from_path(path: &Path) -> Result<ConfigContent, SettingsError> {
    let contents = fs::read_to_string(path)?;
    load_from_str(&contents)
}

/// Loads config content from the default path (~/.xshuttle.json).
///
/// Returns `None` if the config file doesn't exist (caller should use defaults).
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined or the JSON is unparseable.
pub fn load() -> Result<Option<ConfigContent>, SettingsError> {
    let path = config_path().ok_or(SettingsError::NoHomeDir)?;

    if !path.exists() {
        return Ok(None);
    }

    load_from_path(&path).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Schema validation tests
    #[test]
    fn test_schema_is_valid_json() {
        let schema_value: Value = serde_json::from_str(schema()).unwrap();
        assert!(schema_value.is_object());
        assert!(schema_value.get("$schema").is_some());
    }

    #[test]
    fn test_validate_valid_config() {
        let config = r#"{"terminal": "kitty", "actions": []}"#;
        let value: Value = serde_json::from_str(config).unwrap();
        assert!(matches!(validate(&value), ValidationResult::Valid));
    }

    #[test]
    fn test_validate_invalid_action_missing_cmd() {
        let config = r#"{"actions": [{"name": "Test"}]}"#;
        let value: Value = serde_json::from_str(config).unwrap();
        match validate(&value) {
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
        assert!(matches!(validate(&value), ValidationResult::Invalid(_)));
    }

    #[test]
    fn test_validate_unknown_field_rejected() {
        let config = r#"{"unknown_field": "value"}"#;
        let value: Value = serde_json::from_str(config).unwrap();
        assert!(matches!(validate(&value), ValidationResult::Invalid(_)));
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
        assert!(matches!(validate(&value), ValidationResult::Valid));
    }

    // Config loading tests
    #[test]
    fn test_load_from_str_empty() {
        let content = load_from_str("{}").unwrap();
        assert!(content.terminal.is_none());
        assert!(content.editor.is_none());
        assert!(content.actions.is_none());
    }

    #[test]
    fn test_load_from_str_with_terminal() {
        let content = load_from_str(r#"{"terminal": "kitty"}"#).unwrap();
        assert_eq!(content.terminal.as_deref(), Some("kitty"));
    }

    #[test]
    fn test_invalid_unknown_fields() {
        // Unknown fields cause validation error
        let result = load_from_str(r#"{"terminal": "alacritty", "unknown": true}"#);
        assert!(matches!(result, Err(SettingsError::ConfigValidation(_))));
    }

    #[test]
    fn test_invalid_terminal_type() {
        // Invalid type for terminal causes validation error
        let result = load_from_str(r#"{"terminal": 123}"#);
        assert!(matches!(result, Err(SettingsError::ConfigValidation(_))));
    }

    #[test]
    fn test_invalid_editor_type() {
        // Invalid type for editor causes validation error
        let result = load_from_str(r#"{"editor": ["vim", "nano"]}"#);
        assert!(matches!(result, Err(SettingsError::ConfigValidation(_))));
    }

    #[test]
    fn test_invalid_actions_type() {
        // Invalid type for actions causes validation error
        let result = load_from_str(r#"{"actions": "not an array"}"#);
        assert!(matches!(result, Err(SettingsError::ConfigValidation(_))));
    }

    #[test]
    fn test_invalid_action_entry() {
        // Invalid action entries cause validation error
        let result = load_from_str(
            r#"{"actions": [
                {"name": "Valid", "cmd": "echo valid"},
                {"name": "Missing cmd"}
            ]}"#,
        );
        assert!(matches!(result, Err(SettingsError::ConfigValidation(_))));
    }

    #[test]
    fn test_load_from_str_invalid_json() {
        let result = load_from_str("not valid json");
        assert!(matches!(result, Err(SettingsError::ConfigParse(_))));
    }

    #[test]
    fn test_flat_actions() {
        let content =
            load_from_str(r#"{"actions": [{"name": "Test", "cmd": "echo hello"}]}"#).unwrap();
        let entries = content.actions.unwrap();
        assert_eq!(entries.len(), 1);
        match &entries[0] {
            Entry::Action(c) => {
                assert_eq!(c.name, "Test");
                assert_eq!(c.cmd, "echo hello");
            }
            Entry::Group(_) => panic!("Expected Action"),
        }
    }

    #[test]
    fn test_nested_actions() {
        let content = load_from_str(
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
        let entries = content.actions.unwrap();
        assert_eq!(entries.len(), 2);
        match &entries[0] {
            Entry::Action(c) => assert_eq!(c.name, "Top Level"),
            Entry::Group(_) => panic!("Expected Action"),
        }
        match &entries[1] {
            Entry::Group(group) => {
                assert_eq!(group.name, "Production");
                assert_eq!(group.entries.len(), 1);
            }
            Entry::Action(_) => panic!("Expected Group"),
        }
    }

    #[test]
    fn test_deeply_nested_actions() {
        let content = load_from_str(
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
        let entries = content.actions.unwrap();
        assert_eq!(entries.len(), 1);
        match &entries[0] {
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
}
