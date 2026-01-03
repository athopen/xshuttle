use std::collections::HashMap;

use config::Entry;
use image::load_from_memory;
use muda::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub const MENU_ID_CONFIGURE: &str = "configure";
pub const MENU_ID_RELOAD: &str = "reload";
pub const MENU_ID_QUIT: &str = "quit";

const ICON_BYTES: &[u8] = include_bytes!("../../../assets/icon.png");

pub struct Tray {
    icon: Option<TrayIcon>,
}

impl Default for Tray {
    fn default() -> Self {
        Self::new()
    }
}

impl Tray {
    pub fn new() -> Self {
        Self { icon: None }
    }

    pub fn init(&mut self, menu: Menu) {
        self.icon = Some(
            TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("xshuttle")
                .with_icon(load_icon())
                .build()
                .expect("Failed to create tray icon"),
        );
    }

    pub fn set_menu(&self, menu: Menu) {
        if let Some(icon) = &self.icon {
            icon.set_menu(Some(Box::new(menu)));
        }
    }

    pub fn destroy(&mut self) {
        self.icon.take();
    }
}

fn load_icon() -> Icon {
    let img = load_from_memory(ICON_BYTES)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = img.dimensions();
    Icon::from_rgba(img.into_raw(), width, height).expect("Failed to create icon")
}

pub fn build_menu(entries: &[Entry], hosts: &[String]) -> (Menu, HashMap<String, String>) {
    let menu = Menu::new();
    let mut menu_id_map = HashMap::new();
    let mut id_counter = 0usize;

    build_entries(&menu, entries, &mut menu_id_map, &mut id_counter);

    if !entries.is_empty() && !hosts.is_empty() {
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
            Entry::Action(cmd) => {
                let menu_id = format!("action_{}", *id_counter);
                *id_counter += 1;
                menu_id_map.insert(menu_id.clone(), cmd.cmd.clone());
                let menu_item = MenuItem::with_id(menu_id, &cmd.name, true, None);
                menu.append(&menu_item).expect("Failed to append menu item");
            }
            Entry::Group(group) => {
                let submenu = Submenu::new(&group.name, true);
                build_submenu_entries(&submenu, &group.entries, menu_id_map, id_counter);
                menu.append(&submenu).expect("Failed to append submenu");
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
            Entry::Action(cmd) => {
                let menu_id = format!("action_{}", *id_counter);
                *id_counter += 1;
                menu_id_map.insert(menu_id.clone(), cmd.cmd.clone());
                let menu_item = MenuItem::with_id(menu_id, &cmd.name, true, None);
                submenu
                    .append(&menu_item)
                    .expect("Failed to append menu item");
            }
            Entry::Group(group) => {
                let nested = Submenu::new(&group.name, true);
                build_submenu_entries(&nested, &group.entries, menu_id_map, id_counter);
                submenu.append(&nested).expect("Failed to append submenu");
            }
        }
    }
}
