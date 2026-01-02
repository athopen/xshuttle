use std::process::Command;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Terminal {
    #[default]
    Default,
    Gnome,
    Konsole,
    Xfce4,
    Alacritty,
    Kitty,
    Ghostty,
    Wezterm,
    Tilix,
    Terminator,
    XTerminalEmulator,
    Xterm,
}

impl Terminal {
    fn bin(&self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::Gnome => Some("gnome-terminal"),
            Self::Konsole => Some("konsole"),
            Self::Xfce4 => Some("xfce4-terminal"),
            Self::Alacritty => Some("alacritty"),
            Self::Kitty => Some("kitty"),
            Self::Ghostty => Some("ghostty"),
            Self::Wezterm => Some("wezterm"),
            Self::Tilix => Some("tilix"),
            Self::Terminator => Some("terminator"),
            Self::XTerminalEmulator => Some("x-terminal-emulator"),
            Self::Xterm => Some("xterm"),
        }
    }

    fn args(&self) -> &'static [&'static str] {
        match self {
            Self::Default => &[],
            Self::Gnome => &["--", "sh", "-c", "{}; exec bash"],
            Self::Konsole => &["-e", "sh", "-c", "{}; exec bash"],
            Self::Xfce4 => &["-e", "sh -c '{}; exec bash'"],
            Self::Alacritty => &["-e", "sh", "-c", "{}; exec bash"],
            Self::Kitty => &["sh", "-c", "{}; exec bash"],
            Self::Ghostty => &["-e", "sh", "-c", "{}; exec bash"],
            Self::Wezterm => &["start", "--", "sh", "-c", "{}; exec bash"],
            Self::Tilix => &["-e", "sh -c '{}; exec bash'"],
            Self::Terminator => &["-e", "sh -c '{}; exec bash'"],
            Self::XTerminalEmulator => &["-e", "sh", "-c", "{}; exec bash"],
            Self::Xterm => &["-e", "sh", "-c", "{}; exec bash"],
        }
    }

    fn is_available(&self) -> bool {
        self.bin().is_some_and(|bin| which::which(bin).is_ok())
    }

    fn all() -> &'static [Terminal] {
        &[
            Self::Gnome,
            Self::Konsole,
            Self::Xfce4,
            Self::Alacritty,
            Self::Kitty,
            Self::Ghostty,
            Self::Wezterm,
            Self::Tilix,
            Self::Terminator,
            Self::XTerminalEmulator,
            Self::Xterm,
        ]
    }

    fn detect(&self) -> Option<Terminal> {
        // Prefer requested terminal if available
        if *self != Self::Default && self.is_available() {
            return Some(*self);
        }

        // Check $TERMINAL env var
        if let Ok(term_env) = std::env::var("TERMINAL") {
            for &t in Self::all() {
                if t.bin().is_some_and(|b| term_env.contains(b)) && t.is_available() {
                    return Some(t);
                }
            }
        }

        // First available
        Self::all().iter().copied().find(|t| t.is_available())
    }

    pub fn launch(&self, command: &str) -> Result<(), String> {
        let terminal = self
            .detect()
            .ok_or("No terminal found. Install gnome-terminal, konsole, alacritty, or xterm.")?;

        let bin = terminal.bin().expect("detect() never returns Default");
        let args: Vec<String> = terminal
            .args()
            .iter()
            .map(|a| a.replace("{}", command))
            .collect();

        Command::new(bin)
            .args(&args)
            .spawn()
            .map_err(|e| format!("Failed to launch {}: {}", bin, e))?;

        Ok(())
    }
}

impl From<&str> for Terminal {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "gnome-terminal" => Self::Gnome,
            "konsole" => Self::Konsole,
            "xfce4-terminal" => Self::Xfce4,
            "alacritty" => Self::Alacritty,
            "kitty" => Self::Kitty,
            "ghostty" => Self::Ghostty,
            "wezterm" => Self::Wezterm,
            "tilix" => Self::Tilix,
            "terminator" => Self::Terminator,
            "x-terminal-emulator" => Self::XTerminalEmulator,
            "xterm" => Self::Xterm,
            _ => Self::Default,
        }
    }
}
