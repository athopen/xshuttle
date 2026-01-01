use std::collections::HashMap;

use image::load_from_memory;
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::config::{Config, Entry};
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
    menu_id_map: HashMap<String, String>,
}

impl App {
    pub fn init(&mut self) {
        if let Err(e) = Config::ensure_config_exists() {
            eprintln!("Warning: Could not ensure config exists: {}", e);
        }
        let config = Config::load();

        let hosts = parse_ssh_config();
        let (menu, actions) = build_menu(&config.actions, &hosts);
        self.menu_id_map = actions;
        self.config = Some(config);

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
        let config = Config::load();

        let hosts = parse_ssh_config();
        let (menu, menu_id_map) = build_menu(&config.actions, &hosts);
        self.menu_id_map = menu_id_map;
        self.config = Some(config);

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

fn build_menu(actions: &[Entry], hosts: &[String]) -> (Menu, HashMap<String, String>) {
    let menu = Menu::new();
    let mut menu_id_map = HashMap::new();
    let mut id_counter = 0usize;

    build_entries(&menu, actions, &mut menu_id_map, &mut id_counter);

    if !actions.is_empty() && !hosts.is_empty() {
        menu.append(&PredefinedMenuItem::separator()).unwrap();
    }

    for (index, host) in hosts.iter().enumerate() {
        let menu_id = format!("ssh_{}", index);
        menu_id_map.insert(menu_id.clone(), format!("ssh {}", host));
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

    (menu, menu_id_map)
}

fn build_entries(
    menu: &Menu,
    entries: &[Entry],
    menu_id_map: &mut HashMap<String, String>,
    id_counter: &mut usize,
) {
    for entry in entries {
        match entry {
            Entry::Action(action) => {
                let menu_id = format!("action_{}", *id_counter);
                *id_counter += 1;
                menu_id_map.insert(menu_id.clone(), action.cmd.clone());
                let item = MenuItem::with_id(menu_id, &action.name, true, None);
                menu.append(&item).expect("Failed to append menu item");
            }
            Entry::Submenu(submenus) => {
                for (name, children) in submenus {
                    let submenu = Submenu::new(name, true);
                    build_submenu_entries(&submenu, children, menu_id_map, id_counter);
                    menu.append(&submenu).expect("Failed to append submenu");
                }
            }
        }
    }
}

fn build_submenu_entries(
    submenu: &Submenu,
    entries: &[Entry],
    menu_id_map: &mut HashMap<String, String>,
    id_counter: &mut usize,
) {
    for entry in entries {
        match entry {
            Entry::Action(action) => {
                let menu_id = format!("action_{}", *id_counter);
                *id_counter += 1;
                menu_id_map.insert(menu_id.clone(), action.cmd.clone());
                let item = MenuItem::with_id(menu_id, &action.name, true, None);
                submenu.append(&item).expect("Failed to append menu item");
            }
            Entry::Submenu(submenus) => {
                for (name, children) in submenus {
                    let nested = Submenu::new(name, true);
                    build_submenu_entries(&nested, children, menu_id_map, id_counter);
                    submenu.append(&nested).expect("Failed to append submenu");
                }
            }
        }
    }
}

fn is_terminal_editor(editor: &str) -> bool {
    matches!(
        editor,
        "nano" | "vim" | "vi" | "nvim" | "emacs" | "micro" | "ne" | "joe" | "pico" | "ed"
    )
}
