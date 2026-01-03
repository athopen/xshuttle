use std::collections::HashMap;

use config::{Config, config_path, ensure_config_exists, load};
use muda::MenuEvent;
use ssh::parse_ssh_config;
use terminal::Terminal;
use tray::{MENU_ID_CONFIGURE, MENU_ID_QUIT, MENU_ID_RELOAD, Tray, build_menu};

#[derive(Default)]
pub struct App {
    config: Option<Config>,
    tray: Tray,
    menu_id_map: HashMap<String, String>,
}

impl App {
    pub fn init(&mut self) {
        if let Err(e) = ensure_config_exists() {
            eprintln!("Warning: Could not ensure config exists: {}", e);
        }

        let config = load().unwrap_or_else(|e| {
            eprintln!("Warning: {}", e);
            Config::default()
        });

        let hosts = parse_ssh_config();
        let (menu, actions) = build_menu(&config.entries, &hosts);
        self.menu_id_map = actions;
        self.config = Some(config);

        self.tray.init(menu);
    }

    pub fn handle_menu_event(&mut self, event: MenuEvent) -> bool {
        let menu_id = &event.id.0;

        if menu_id == MENU_ID_QUIT {
            self.tray.destroy();
            return true;
        }

        if menu_id == MENU_ID_CONFIGURE {
            self.configure();
            return false;
        }

        if menu_id == MENU_ID_RELOAD {
            self.reload();
            return false;
        }

        if let Some(command) = self.menu_id_map.get(menu_id) {
            let terminal = self
                .config
                .as_ref()
                .map(|c| Terminal::from(c.terminal.as_str()))
                .unwrap_or_default();

            if let Err(e) = terminal.launch(command) {
                eprintln!("Error: {}", e);
            }
        }

        false
    }

    fn configure(&self) {
        let Some(path) = config_path() else {
            eprintln!("Error: Could not determine config path");
            return;
        };

        let editor = self
            .config
            .as_ref()
            .map(|c| c.editor.as_str())
            .unwrap_or("default");

        let result = match editor {
            "default" => open::that(&path),
            editor if is_terminal_editor(editor) => {
                let cmd = format!("{} {}", editor, path.display());
                let terminal = self
                    .config
                    .as_ref()
                    .map(|c| Terminal::from(c.terminal.as_str()))
                    .unwrap_or_default();
                terminal.launch(&cmd).map_err(std::io::Error::other)
            }
            editor => open::with(&path, editor),
        };

        if let Err(e) = result {
            eprintln!("Error opening config: {}", e);
        }
    }

    fn reload(&mut self) {
        let config = load().unwrap_or_else(|e| {
            eprintln!("Warning: {}", e);
            Config::default()
        });

        let hosts = parse_ssh_config();
        let (menu, menu_id_map) = build_menu(&config.entries, &hosts);
        self.menu_id_map = menu_id_map;
        self.config = Some(config);

        self.tray.set_menu(menu);
    }
}

fn is_terminal_editor(editor: &str) -> bool {
    matches!(
        editor,
        "nano" | "vim" | "vi" | "nvim" | "emacs" | "micro" | "ne" | "joe" | "pico" | "ed"
    )
}
