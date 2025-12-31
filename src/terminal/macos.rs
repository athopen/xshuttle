use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum Terminal {
    #[default]
    Auto,
    ITerm,
    Terminal,
    Warp,
    Kitty,
    Ghostty,
    Wezterm,
}

impl Terminal {
    fn app(&self) -> Option<&'static str> {
        match self {
            Self::Auto => None,
            Self::ITerm => Some("iTerm"),
            Self::Terminal => Some("Terminal"),
            Self::Warp => Some("Warp"),
            Self::Kitty => Some("kitty"),
            Self::Ghostty => Some("Ghostty"),
            Self::Wezterm => Some("WezTerm"),
        }
    }

    fn custom_launch(&self) -> Option<&'static [&'static str]> {
        match self {
            Self::Wezterm => Some(&["open", "-na", "wezterm", "--args", "start", "--"]),
            _ => None,
        }
    }

    fn is_available(&self) -> bool {
        self.app().is_some_and(is_app_installed)
    }

    fn all() -> &'static [Terminal] {
        &[
            Self::ITerm,
            Self::Terminal,
            Self::Warp,
            Self::Kitty,
            Self::Ghostty,
            Self::Wezterm,
        ]
    }

    fn detect(&self) -> Option<Terminal> {
        // Prefer requested terminal if available
        if *self != Self::Auto && self.is_available() {
            return Some(*self);
        }

        // First available
        Self::all().iter().copied().find(|t| t.is_available())
    }

    pub fn launch(&self, command: &str) -> Result<(), String> {
        let terminal = self
            .detect()
            .ok_or("No terminal found. Install Terminal.app, iTerm, or Warp.")?;

        let app = terminal.app().expect("detect() never returns Auto");
        let script_path = create_script(command)?;

        match terminal.custom_launch() {
            Some(args) => {
                let mut cmd_args: Vec<&str> = args.to_vec();
                cmd_args.push(&script_path);

                Command::new(cmd_args[0])
                    .args(&cmd_args[1..])
                    .spawn()
                    .map_err(|e| format!("Failed to launch {}: {}", app, e))?;
            }
            None => {
                Command::new("open")
                    .args(["-a", app, &script_path])
                    .spawn()
                    .map_err(|e| format!("Failed to launch {}: {}", app, e))?;
            }
        }

        Ok(())
    }
}

impl From<&str> for Terminal {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "iterm" => Self::ITerm,
            "terminal" | "terminal.app" => Self::Terminal,
            "warp" => Self::Warp,
            "kitty" => Self::Kitty,
            "ghostty" => Self::Ghostty,
            "wezterm" => Self::Wezterm,
            _ => Self::Auto,
        }
    }
}

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
