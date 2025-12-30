use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

/// Terminal app with its bundle name and launch method
struct Terminal {
    /// Application name for `open -a`
    app: &'static str,
    /// Custom launch command (None = use standard `open -a`)
    custom_launch: Option<&'static [&'static str]>,
}

/// Known macOS terminal emulators in preference order
const TERMINALS: &[Terminal] = &[
    Terminal {
        app: "iTerm",
        custom_launch: None,
    },
    Terminal {
        app: "Terminal",
        custom_launch: None,
    },
    Terminal {
        app: "Warp",
        custom_launch: None,
    },
    Terminal {
        app: "kitty",
        custom_launch: None,
    },
    Terminal {
        app: "Ghostty",
        custom_launch: None,
    },
    Terminal {
        app: "WezTerm",
        custom_launch: Some(&["open", "-na", "wezterm", "--args", "start", "--"]),
    },
];

/// Check if a macOS app is installed
fn is_app_installed(app: &str) -> bool {
    Command::new("mdfind")
        .args([
            "kMDItemKind == 'Application'",
            &format!("kMDItemDisplayName == '{}'", app),
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

/// Detect available terminal emulator
fn detect_terminal() -> Option<&'static Terminal> {
    TERMINALS.iter().find(|t| is_app_installed(t.app))
}

/// Create a temporary script with the command
fn create_script(command: &str) -> Result<String, String> {
    let script_path = "/tmp/xshuttle-run.sh";

    let script_content = format!("#!/bin/zsh -il\n{}\nexec $SHELL", command);

    let mut file =
        File::create(script_path).map_err(|e| format!("Failed to create script: {}", e))?;

    file.write_all(script_content.as_bytes())
        .map_err(|e| format!("Failed to write script: {}", e))?;

    fs::set_permissions(script_path, fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to set permissions: {}", e))?;

    Ok(script_path.to_string())
}

/// Launch a command in the system's terminal emulator
pub fn launch_in_terminal(command: &str) -> Result<(), String> {
    let terminal = detect_terminal().ok_or_else(|| {
        "No terminal emulator found. Install Terminal.app, iTerm, or Warp.".to_string()
    })?;

    let script_path = create_script(command)?;

    match terminal.custom_launch {
        Some(args) => {
            let mut cmd_args: Vec<&str> = args.to_vec();
            cmd_args.push(&script_path);

            Command::new(cmd_args[0])
                .args(&cmd_args[1..])
                .spawn()
                .map_err(|e| format!("Failed to launch {}: {}", terminal.app, e))?;
        }
        None => {
            Command::new("open")
                .args(["-a", terminal.app, &script_path])
                .spawn()
                .map_err(|e| format!("Failed to launch {}: {}", terminal.app, e))?;
        }
    }

    Ok(())
}
