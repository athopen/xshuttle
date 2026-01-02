use ssh2_config::{ParseRule, SshConfig};

/// Parse SSH config file and return a list of host names
pub fn parse_ssh_config() -> Vec<String> {
    let config = match SshConfig::parse_default_file(ParseRule::ALLOW_UNKNOWN_FIELDS) {
        Ok(config) => config,
        Err(_) => return Vec::new(),
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
