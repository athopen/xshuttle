mod xshuttle;

use clap::Parser;
use tray::MenuEvent;
use winit::application::ApplicationHandler;
use winit::event::StartCause;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use xshuttle::{Application, UserEvent};

const VERSION: &str = concat!(env!("XSHUTTLE_VERSION"), " ", env!("XSHUTTLE_BUILD_HASH"));

#[derive(Parser)]
#[command(name = "xshuttle", version = VERSION)]
struct Arguments {}

impl ApplicationHandler<UserEvent> for Application {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
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
    use objc2_core_foundation::CFRunLoop;
    if let Some(rl) = CFRunLoop::main() {
        rl.wake_up();
    }
}

#[cfg(target_os = "linux")]
fn run_gtk_thread(quit_proxy: winit::event_loop::EventLoopProxy<UserEvent>) {
    use tray::MenuId;

    std::thread::spawn(move || {
        gtk::init().unwrap();

        let mut app = Application::default();
        app.init();

        let receiver = MenuEvent::receiver();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            while let Ok(event) = receiver.try_recv() {
                if app.handle_menu_event(event) {
                    let quit_event = MenuEvent {
                        id: MenuId::new("quit"),
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

fn main() {
    Arguments::parse();

    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    #[cfg(target_os = "linux")]
    run_gtk_thread(proxy);

    #[cfg(not(target_os = "linux"))]
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    event_loop.run_app(&mut Application::default()).unwrap();
}
