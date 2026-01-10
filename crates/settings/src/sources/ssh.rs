use crate::error::SettingsError;
use ssh2_config::{ParseRule, SshConfig};
use std::path::PathBuf;

/// Returns the default SSH config file path (~/.ssh/config).
fn ssh_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".ssh").join("config"))
}

/// Parses SSH config file and returns host names.
///
/// Returns an empty list if the SSH config file doesn't exist.
/// Returns an error if the file exists but cannot be parsed (user should fix it).
///
/// # Errors
///
/// Returns `SettingsError::SshParse` if the SSH config file exists but is malformed.
pub fn parse_ssh_config() -> Result<Vec<String>, SettingsError> {
    let Some(path) = ssh_config_path() else {
        return Err(SettingsError::NoHomeDir);
    };

    // If the file doesn't exist, return empty list (not an error)
    if !path.exists() {
        return Ok(Vec::new());
    }

    // File exists, so parse errors are fatal
    let config = SshConfig::parse_default_file(ParseRule::ALLOW_UNKNOWN_FIELDS)
        .map_err(|e| SettingsError::SshParse(e.to_string()))?;

    let mut hosts = Vec::new();

    for host in config.get_hosts() {
        for clause in &host.pattern {
            let name = clause.pattern.as_str();

            // Skip wildcards and patterns
            if name.contains('*') || name.contains('?') || name == "!" {
                continue;
            }

            // Skip negated patterns
            if clause.negated {
                continue;
            }

            hosts.push(name.to_string());
        }
    }

    hosts.sort_by_key(|a| a.to_lowercase());
    Ok(hosts)
}
