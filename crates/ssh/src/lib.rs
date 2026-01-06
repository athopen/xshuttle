use ssh2_config::{ParseRule, SshConfig};

/// Parses SSH config file and returns host names.
pub fn parse_ssh_config() -> Vec<String> {
    let Ok(config) = SshConfig::parse_default_file(ParseRule::ALLOW_UNKNOWN_FIELDS) else {
        return Vec::new();
    };

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
    hosts
}
