use std::process::Command;

/// Terminal emulator with its launch arguments
struct Terminal {
    /// Binary name
    bin: &'static str,
    /// Arguments to execute a command
    /// Use {} as placeholder for the command
    args: &'static [&'static str],
}

/// Known Linux terminal emulators in preference order
const TERMINALS: &[Terminal] = &[
    // Modern terminals
    Terminal {
        bin: "gnome-terminal",
        args: &["--", "sh", "-c", "{}; exec bash"],
    },
    Terminal {
        bin: "konsole",
        args: &["-e", "sh", "-c", "{}; exec bash"],
    },
    Terminal {
        bin: "xfce4-terminal",
        args: &["-e", "sh -c '{}; exec bash'"],
    },
    Terminal {
        bin: "alacritty",
        args: &["-e", "sh", "-c", "{}; exec bash"],
    },
    Terminal {
        bin: "kitty",
        args: &["sh", "-c", "{}; exec bash"],
    },
    Terminal {
        bin: "wezterm",
        args: &["start", "--", "sh", "-c", "{}; exec bash"],
    },
    Terminal {
        bin: "tilix",
        args: &["-e", "sh -c '{}; exec bash'"],
    },
    Terminal {
        bin: "terminator",
        args: &["-e", "sh -c '{}; exec bash'"],
    },
    // Fallbacks
    Terminal {
        bin: "x-terminal-emulator",
        args: &["-e", "sh", "-c", "{}; exec bash"],
    },
    Terminal {
        bin: "xterm",
        args: &["-e", "sh", "-c", "{}; exec bash"],
    },
];

/// Detect available terminal emulator
fn detect_terminal() -> Option<&'static Terminal> {
    // Check $TERMINAL environment variable first
    if let Ok(term_env) = std::env::var("TERMINAL") {
        for terminal in TERMINALS {
            if term_env.contains(terminal.bin) && which::which(terminal.bin).is_ok() {
                return Some(terminal);
            }
        }
    }

    // Fall back to checking known terminals
    TERMINALS.iter().find(|t| which::which(t.bin).is_ok())
}

/// Launch a command in the system's terminal emulator
pub fn launch_in_terminal(command: &str) -> Result<(), String> {
    let terminal = detect_terminal().ok_or_else(|| {
        "No terminal emulator found. Install gnome-terminal, konsole, alacritty, or xterm."
            .to_string()
    })?;

    let args: Vec<String> = terminal
        .args
        .iter()
        .map(|arg| arg.replace("{}", command))
        .collect();

    Command::new(terminal.bin)
        .args(&args)
        .spawn()
        .map_err(|e| format!("Failed to launch {}: {}", terminal.bin, e))?;

    Ok(())
}
