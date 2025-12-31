use std::collections::HashMap;

use image::load_from_memory;
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::config::Config;
use crate::ssh::parse_ssh_config;
use crate::terminal::Terminal;

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

    menu.append(&MenuItem::with_id(MENU_ID_QUIT, "Quit", true, None))
        .unwrap();

    (menu, commands)
}
