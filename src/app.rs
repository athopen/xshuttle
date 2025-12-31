use std::collections::HashMap;

use image::load_from_memory;
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::config::Config;
use crate::ssh::parse_ssh_config;
use crate::terminal::Terminal;

const MENU_ID_CONFIGURE: &str = "configure";
const MENU_ID_RELOAD: &str = "reload";
const MENU_ID_QUIT: &str = "quit";
const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");

#[derive(Default)]
pub struct App {
    config: Option<Config>,
    icon: Option<TrayIcon>,
    commands: HashMap<String, String>,
}

impl App {
    pub fn init(&mut self) {
        if let Err(e) = Config::ensure_config_exists() {
            eprintln!("Warning: Could not ensure config exists: {}", e);
        }
        self.config = Some(Config::load());

        let hosts = parse_ssh_config();
        let (menu, commands) = build_menu(&hosts);
        self.commands = commands;

        self.icon = Some(
            TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("xshuttle")
                .with_icon(load_icon())
                .build()
                .expect("Failed to create tray icon"),
        );
    }

    pub fn handle_menu_event(&mut self, event: MenuEvent) -> bool {
        let menu_id = &event.id.0;

        if menu_id == MENU_ID_QUIT {
            self.icon.take();
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

        if let Some(command) = self.commands.get(menu_id) {
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
        let Some(config_path) = Config::config_path() else {
            eprintln!("Error: Could not determine config path");
            return;
        };

        let editor = self
            .config
            .as_ref()
            .map(|c| c.editor.as_str())
            .unwrap_or("default");

        let result = match editor {
            "default" => open::that(&config_path),
            editor if is_terminal_editor(editor) => {
                let cmd = format!("{} {}", editor, config_path.display());
                let terminal = self
                    .config
                    .as_ref()
                    .map(|c| Terminal::from(c.terminal.as_str()))
                    .unwrap_or_default();
                terminal.launch(&cmd).map_err(std::io::Error::other)
            }
            editor => open::with(&config_path, editor),
        };

        if let Err(e) = result {
            eprintln!("Error opening config: {}", e);
        }
    }

    fn reload(&mut self) {
        self.config = Some(Config::load());

        let hosts = parse_ssh_config();
        let (menu, commands) = build_menu(&hosts);
        self.commands = commands;

        if let Some(icon) = &self.icon {
            icon.set_menu(Some(Box::new(menu)));
        }
    }
}

fn load_icon() -> Icon {
    let img = load_from_memory(ICON_BYTES)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = img.dimensions();
    Icon::from_rgba(img.into_raw(), width, height).expect("Failed to create icon")
}

fn build_menu(hosts: &[String]) -> (Menu, HashMap<String, String>) {
    let menu = Menu::new();
    let mut commands = HashMap::new();

    for (index, host) in hosts.iter().enumerate() {
        let menu_id = format!("ssh_{}", index);
        commands.insert(menu_id.clone(), format!("ssh {}", host));
        let item = MenuItem::with_id(menu_id, host, true, None);
        menu.append(&item).expect("Failed to append menu item");
    }

    if !hosts.is_empty() {
        menu.append(&PredefinedMenuItem::separator()).unwrap();
    }

    menu.append(&MenuItem::with_id(
        MENU_ID_CONFIGURE,
        "Configure",
        true,
        None,
    ))
    .unwrap();
    menu.append(&MenuItem::with_id(MENU_ID_RELOAD, "Reload", true, None))
        .unwrap();
    menu.append(&MenuItem::with_id(MENU_ID_QUIT, "Quit", true, None))
        .unwrap();

    (menu, commands)
}

fn is_terminal_editor(editor: &str) -> bool {
    matches!(
        editor,
        "nano" | "vim" | "vi" | "nvim" | "emacs" | "micro" | "ne" | "joe" | "pico" | "ed"
    )
}
