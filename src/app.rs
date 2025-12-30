use std::collections::HashMap;

use image::load_from_memory;
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::ssh::parse_ssh_config;
use crate::terminal::launch_in_terminal;

const MENU_ID_QUIT: &str = "quit";
const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");

#[derive(Default)]
pub struct App {
    icon: Option<TrayIcon>,
    commands: HashMap<String, String>,
}

impl App {
    pub fn init(&mut self) {
        let hosts = parse_ssh_config();
        let (menu, commands) = build_menu(&hosts);
        self.commands = commands;

        let img = load_from_memory(ICON_BYTES)
            .expect("Failed to load icon")
            .into_rgba8();
        let (width, height) = img.dimensions();
        let icon = Icon::from_rgba(img.into_raw(), width, height).expect("Failed to create icon");

        self.icon = Some(
            TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("xshuttle")
                .with_icon(icon)
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

        if let Some(command) = self.commands.get(menu_id)
            && let Err(e) = launch_in_terminal(command)
        {
            eprintln!("Error: {}", e);
        }

        false
    }
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
