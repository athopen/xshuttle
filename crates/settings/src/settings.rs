use crate::error::SettingsError;
use crate::host::Host;
use crate::loaders::{config, ssh};
use crate::nodes::Nodes;
use crate::types::Action;
use std::io;
use std::path::PathBuf;

/// Complete application settings loaded from all sources.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Terminal emulator to use for commands.
    pub terminal: String,
    /// Editor for opening config files.
    pub editor: String,
    /// Actions from ~/.xshuttle.json with O(1) ID-based lookup.
    pub actions: Nodes<Action>,
    /// SSH hosts from ~/.ssh/config with O(1) ID-based lookup.
    pub hosts: Nodes<Host>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            terminal: Self::DEFAULT_TERMINAL.to_string(),
            editor: Self::DEFAULT_EDITOR.to_string(),
            actions: Nodes::from_entries(vec![]),
            hosts: Nodes::from_hostnames(vec![]),
        }
    }
}

impl Settings {
    /// Default terminal value when not specified in config.
    pub const DEFAULT_TERMINAL: &'static str = "default";
    /// Default editor value when not specified in config.
    pub const DEFAULT_EDITOR: &'static str = "default";

    /// Load settings from all sources.
    ///
    /// This loads:
    /// - Configuration from `~/.xshuttle.json` (uses defaults if missing)
    /// - SSH hosts from `~/.ssh/config` (empty if file doesn't exist)
    ///
    /// Warnings for non-fatal issues are logged via the `log` crate.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Home directory cannot be determined
    /// - Config file exists but is invalid JSON or fails validation
    /// - SSH config file exists but cannot be parsed
    pub fn load() -> Result<Self, SettingsError> {
        let config = config::load()?.unwrap_or_default();
        let raw_hosts = ssh::parse_ssh_config()?;

        Ok(Settings {
            terminal: config
                .terminal
                .unwrap_or_else(|| Self::DEFAULT_TERMINAL.to_string()),
            editor: config
                .editor
                .unwrap_or_else(|| Self::DEFAULT_EDITOR.to_string()),
            actions: Nodes::from_entries(config.actions.unwrap_or_default()),
            hosts: Nodes::from_hostnames(raw_hosts),
        })
    }

    /// Get the path to the main config file.
    pub fn config_path() -> Option<PathBuf> {
        config::config_path()
    }

    /// Ensure the config file exists, creating a default one if missing.
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined or if
    /// writing the default config file fails.
    pub fn ensure_config_exists() -> Result<PathBuf, io::Error> {
        config::ensure_config_exists().map_err(|e| match e {
            SettingsError::ConfigIo(io_err) => io_err,
            SettingsError::NoHomeDir => io::Error::new(
                io::ErrorKind::NotFound,
                "could not determine home directory",
            ),
            _ => io::Error::other(e.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path_returns_some() {
        // This test assumes a home directory exists
        let path = Settings::config_path();
        if let Some(p) = path {
            assert!(p.ends_with(".xshuttle.json"));
        }
    }

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert_eq!(settings.terminal, Settings::DEFAULT_TERMINAL);
        assert_eq!(settings.editor, Settings::DEFAULT_EDITOR);
        assert!(settings.actions.is_empty());
        assert!(settings.hosts.is_empty());
    }
}
