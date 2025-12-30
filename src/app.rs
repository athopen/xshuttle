use crate::asset::load_icon;
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

#[derive(Default)]
pub struct App {
    icon: Option<TrayIcon>,
}

impl App {
    pub fn init(&mut self) {
        let menu = Menu::new();
        menu.append(&MenuItem::with_id("dummy1", "dummy1", true, None))
            .unwrap();
        menu.append(&MenuItem::with_id("dummy2", "dummy2", true, None))
            .unwrap();
        menu.append(&PredefinedMenuItem::separator()).unwrap();
        menu.append(&MenuItem::with_id("configure", "Configure", true, None))
            .unwrap();
        menu.append(&MenuItem::with_id("quit", "Quit", true, None))
            .unwrap();

        let img = load_icon();
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

    /// Handle a menu event. Returns true if the app should quit.
    pub fn handle_menu_event(&mut self, event: MenuEvent) -> bool {
        println!("{}", event.id.0);
        if event.id.0 == "quit" {
            self.icon.take();
            return true;
        }
        false
    }
}
