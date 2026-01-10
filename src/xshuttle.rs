use settings::{NodeId, Settings};
use terminal::Terminal;
use tray::{
    MENU_ID_ACTION_PREFIX, MENU_ID_CONFIGURE, MENU_ID_HOST_PREFIX, MENU_ID_QUIT, MENU_ID_RELOAD,
    Menu, MenuEvent, Tray, build_menu,
};

#[derive(Debug)]
pub enum UserEvent {
    MenuEvent(MenuEvent),
}

#[derive(Default)]
pub struct Application {
    settings: Option<Settings>,
    tray: Tray,
}

impl Application {
    pub fn init(&mut self) {
        if let Err(e) = Settings::ensure_config_exists() {
            eprintln!("Warning: Could not ensure config exists: {e}");
        }

        let menu = self.build();
        self.tray.init(menu);
    }

    fn build(&mut self) -> Menu {
        let settings = match Settings::load() {
            Ok(settings) => settings,
            Err(e) => {
                eprintln!("Error loading settings: {e}");
                // Return empty menu if settings fail to load
                self.settings = None;
                return Menu::new();
            }
        };

        let menu = build_menu(&settings);
        self.settings = Some(settings);

        menu
    }

    pub fn handle_menu_event(&mut self, event: &MenuEvent) -> bool {
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

        // O(1) lookup for dynamic menu items
        let command = self.lookup_command(menu_id);
        if let Some(cmd) = command {
            let terminal = self
                .settings
                .as_ref()
                .map(|s| Terminal::from(s.terminal.as_str()))
                .unwrap_or_default();

            if let Err(e) = terminal.launch(&cmd) {
                eprintln!("Error: {e}");
            }
        }

        false
    }

    /// O(1) lookup for action and host commands by menu ID.
    fn lookup_command(&self, menu_id: &str) -> Option<String> {
        let settings = self.settings.as_ref()?;

        // Check for action prefix: "action_{index}"
        if let Some(index_str) = menu_id.strip_prefix(MENU_ID_ACTION_PREFIX) {
            let index: usize = index_str.parse().ok()?;
            let action = settings.actions.get(NodeId::from_index(index))?;
            return Some(action.cmd.clone());
        }

        // Check for host prefix: "host_{index}"
        if let Some(index_str) = menu_id.strip_prefix(MENU_ID_HOST_PREFIX) {
            let index: usize = index_str.parse().ok()?;
            let host = settings.hosts.get(NodeId::from_index(index))?;
            return Some(host.command());
        }

        None
    }

    fn configure(&self) {
        let Some(path) = Settings::config_path() else {
            eprintln!("Error: Could not determine config path");
            return;
        };

        let editor = self
            .settings
            .as_ref()
            .map_or("default", |s| s.editor.as_str());

        let path_display = path.display();
        let result = match editor {
            "default" => open::that(&path),
            editor if is_terminal_editor(editor) => {
                let cmd = format!("{editor} {path_display}");
                let terminal = self
                    .settings
                    .as_ref()
                    .map(|s| Terminal::from(s.terminal.as_str()))
                    .unwrap_or_default();
                terminal.launch(&cmd).map_err(std::io::Error::other)
            }
            editor => open::with(&path, editor),
        };

        if let Err(e) = result {
            eprintln!("Error opening config: {e}");
        }
    }

    fn reload(&mut self) {
        let menu = self.build();
        self.tray.set_menu(menu);
    }
}

fn is_terminal_editor(editor: &str) -> bool {
    matches!(
        editor,
        "nano" | "vim" | "vi" | "nvim" | "emacs" | "micro" | "ne" | "joe" | "pico" | "ed"
    )
}
