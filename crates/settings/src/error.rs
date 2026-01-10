use std::fmt;
use std::io;
use thiserror::Error;

// ============================================================================
// Validation Types
// ============================================================================

/// A validation error with path and message.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// JSON path to the error location.
    pub path: String,
    /// Human-readable error description.
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
    /// Validation passed with no errors.
    Valid,
    /// Validation failed with one or more errors.
    Invalid(Vec<ValidationError>),
}

// ============================================================================
// Settings Error
// ============================================================================

/// Error type for settings loading operations.
#[derive(Error, Debug)]
pub enum SettingsError {
    /// Home directory not found.
    #[error("could not determine home directory")]
    NoHomeDir,

    /// Config file I/O error.
    #[error("failed to read config: {0}")]
    ConfigIo(#[from] io::Error),

    /// Config JSON parse error (not valid JSON at all).
    #[error("invalid JSON: {0}")]
    ConfigParse(#[from] serde_json::Error),

    /// Config schema validation error.
    #[error("config validation failed: {}", format_validation_errors(.0))]
    ConfigValidation(Vec<ValidationError>),

    /// SSH config parse error (fatal - user should fix their SSH config).
    #[error("failed to parse SSH config: {0}")]
    SshParse(String),
}

fn format_validation_errors(errors: &[ValidationError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}
