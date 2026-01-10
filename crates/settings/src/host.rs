//! SSH host entry type.

/// An SSH host entry.
///
/// Wraps a hostname string for type safety and future extensibility.
#[derive(Debug, Clone)]
pub struct Host {
    /// The SSH hostname to connect to.
    pub hostname: String,
}

impl Host {
    /// Returns the command to execute: `ssh {hostname}`.
    #[must_use]
    pub fn command(&self) -> String {
        format!("ssh {}", self.hostname)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_command() {
        let host = Host {
            hostname: "prod-server".into(),
        };
        assert_eq!(host.command(), "ssh prod-server");
    }

    #[test]
    fn test_host_clone() {
        let host = Host {
            hostname: "staging".into(),
        };
        let cloned = host.clone();
        assert_eq!(host.hostname, cloned.hostname);
    }
}
