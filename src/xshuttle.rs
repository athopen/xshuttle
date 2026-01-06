use std::collections::HashMap;

use config::{Config, config_path, ensure_config_exists, load};
use ssh::parse_ssh_config;
use terminal::Terminal;
use tray::{MENU_ID_CONFIGURE, MENU_ID_QUIT, MENU_ID_RELOAD, Menu, MenuEvent, Tray, build_menu};
use winit::application::ApplicationHandler;
use winit::event::StartCause;
use winit::event_loop::ActiveEventLoop;

#[derive(Debug)]
pub enum UserEvent {
    MenuEvent(MenuEvent),
}

#[derive(Default)]
pub struct Application {
    config: Option<Config>,
    tray: Tray,
    menu_id_map: HashMap<String, String>,
}

impl Application {
    pub fn init(&mut self) {
        if let Err(e) = ensure_config_exists() {
            eprintln!("Warning: Could not ensure config exists: {}", e);
        }

        let menu = self.build();
        self.tray.init(menu);
    }

    fn build(&mut self) -> Menu {
        let config = load().unwrap_or_else(|e| {
            eprintln!("Warning: {}", e);
            Config::default()
        });

        let hosts = parse_ssh_config();
        let (menu, menu_id_map) = build_menu(&config.entries, &hosts);

        self.menu_id_map = menu_id_map;
        self.config = Some(config);

        menu
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

impl ApplicationHandler<UserEvent> for Application {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            // On Linux, the app is initialized in the GTK thread instead
            #[cfg(not(target_os = "linux"))]
            self.init();

            #[cfg(target_os = "macos")]
            wake_macos_run_loop();
        }
    }

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        let UserEvent::MenuEvent(event) = event;

        // On Linux, this is a quit signal from the GTK thread
        #[cfg(target_os = "linux")]
        {
            let _ = event;
            event_loop.exit();
        }

        // On other platforms, handle the menu event directly
        #[cfg(not(target_os = "linux"))]
        if self.handle_menu_event(event) {
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: winit::event::WindowEvent,
    ) {
    }
}

#[cfg(target_os = "macos")]
fn wake_macos_run_loop() {
    // Wake the run loop to ensure the tray icon appears immediately
    use objc2_core_foundation::CFRunLoop;
    if let Some(rl) = CFRunLoop::main() {
        rl.wake_up();
    }
}

#[cfg(target_os = "linux")]
pub fn run_gtk_thread(quit_proxy: winit::event_loop::EventLoopProxy<UserEvent>) {
    // On Linux, winit doesn't use GTK but tray-icon requires it.
    // Run the app in a dedicated GTK thread.
    std::thread::spawn(move || {
        gtk::init().unwrap();

        let mut app = Application::default();
        app.init();

        // Poll for menu events in the GTK main loop
        let receiver = MenuEvent::receiver();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            while let Ok(event) = receiver.try_recv() {
                if app.handle_menu_event(event) {
                    // Signal the winit event loop to exit
                    let quit_event = MenuEvent {
                        id: tray::MenuId::new("quit"),
                    };
                    let _ = quit_proxy.send_event(UserEvent::MenuEvent(quit_event));
                    gtk::main_quit();
                    return gtk::glib::ControlFlow::Break;
                }
            }
            gtk::glib::ControlFlow::Continue
        });

        gtk::main();
    });
}
