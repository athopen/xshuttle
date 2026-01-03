use jsonschema::Validator;
use serde_json::Value;
use std::fmt;

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

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::InvalidJson(e) => Some(e),
            ConfigError::ValidationFailed(_) => None,
        }
    }
}

/// Returns the embedded JSON schema as a string.
pub fn schema() -> &'static str {
    SCHEMA_JSON
}

/// Validate a JSON value against the config schema.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_is_valid_json() {
        let schema: Value = serde_json::from_str(schema()).unwrap();
        assert!(schema.is_object());
        assert!(schema.get("$schema").is_some());
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
}
