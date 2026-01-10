use crate::sources::config::ValidationError;
use std::io;
use thiserror::Error;

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
