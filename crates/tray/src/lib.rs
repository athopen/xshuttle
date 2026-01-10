use std::fmt;

use image::load_from_memory;
use settings::{Action, Node, Nodes, Settings};
use tray_icon::menu::{MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub use tray_icon::menu::{Menu, MenuEvent, MenuId};

pub const MENU_ID_CONFIGURE: &str = "configure";
pub const MENU_ID_RELOAD: &str = "reload";
pub const MENU_ID_QUIT: &str = "quit";
pub const MENU_ID_ACTION_PREFIX: &str = "action_";
pub const MENU_ID_HOST_PREFIX: &str = "host_";

const ICON_BYTES: &[u8] = include_bytes!("../../../assets/icon.png");

pub struct Tray {
    icon: Option<TrayIcon>,
}

impl fmt::Debug for Tray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tray")
            .field("icon", &self.icon.as_ref().map(|_| "TrayIcon"))
            .finish()
    }
}

impl Default for Tray {
    fn default() -> Self {
        Self::new()
    }
}

impl Tray {
    /// Creates a new tray icon instance (not yet initialized).
    pub fn new() -> Self {
        Self { icon: None }
    }

    /// Initializes the tray icon with the given menu.
    ///
    /// # Panics
    ///
    /// Panics if the tray icon or embedded icon cannot be created.
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

/// Builds a menu from settings.
///
/// Uses the indexed `Nodes<T>` containers for O(1) lookup.
/// Menu item IDs are formatted as `node_{index}` for dynamic entries.
///
/// # Panics
///
/// Panics if menu items cannot be appended to the menu.
pub fn build_menu(settings: &Settings) -> Menu {
    let menu = Menu::new();

    // Build action entries (with submenus)
    build_action_nodes(&menu, settings.actions.nodes(), &settings.actions);

    // Add separator if both sections have items
    if !settings.actions.is_empty() && !settings.hosts.is_empty() {
        menu.append(&PredefinedMenuItem::separator()).unwrap();
    }

    // Build host entries (flat)
    for node in settings.hosts.nodes() {
        if let Node::Leaf { id, .. } = node
            && let Some(host) = settings.hosts.get(*id)
        {
            let menu_id = format!("{}{}", MENU_ID_HOST_PREFIX, id.index());
            let item = MenuItem::with_id(menu_id, &host.hostname, true, None);
            menu.append(&item).expect("Failed to append menu item");
        }
    }

    // Add separator before static items
    if !settings.hosts.is_empty() || !settings.actions.is_empty() {
        menu.append(&PredefinedMenuItem::separator()).unwrap();
    }

    // Add static menu items
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

    menu
}

fn build_action_nodes(menu: &Menu, tree: &[Node<Action>], actions: &Nodes<Action>) {
    for node in tree {
        match node {
            Node::Leaf { id, .. } => {
                if let Some(action) = actions.get(*id) {
                    let menu_id = format!("{}{}", MENU_ID_ACTION_PREFIX, id.index());
                    let menu_item = MenuItem::with_id(menu_id, &action.name, true, None);
                    menu.append(&menu_item).expect("Failed to append menu item");
                }
            }
            Node::Group { name, children } => {
                let submenu = Submenu::new(name, true);
                build_action_submenu(&submenu, children, actions);
                menu.append(&submenu).expect("Failed to append submenu");
            }
        }
    }
}

fn build_action_submenu(submenu: &Submenu, tree: &[Node<Action>], actions: &Nodes<Action>) {
    for node in tree {
        match node {
            Node::Leaf { id, .. } => {
                if let Some(action) = actions.get(*id) {
                    let menu_id = format!("{}{}", MENU_ID_ACTION_PREFIX, id.index());
                    let menu_item = MenuItem::with_id(menu_id, &action.name, true, None);
                    submenu
                        .append(&menu_item)
                        .expect("Failed to append menu item");
                }
            }
            Node::Group { name, children } => {
                let nested = Submenu::new(name, true);
                build_action_submenu(&nested, children, actions);
                submenu.append(&nested).expect("Failed to append submenu");
            }
        }
    }
}
