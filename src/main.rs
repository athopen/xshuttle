mod xshuttle;

use clap::Parser;
use winit::event_loop::EventLoop;
use xshuttle::{Application, UserEvent};

#[cfg(not(target_os = "linux"))]
use tray::MenuEvent;

const VERSION: &str = concat!(env!("XSHUTTLE_VERSION"), " ", env!("XSHUTTLE_BUILD_HASH"));

#[derive(Parser)]
#[command(name = "xshuttle", version = VERSION)]
struct Arguments {}

fn main() {
    Arguments::parse();

    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    #[cfg(target_os = "linux")]
    xshuttle::run_gtk_thread(proxy);

    #[cfg(not(target_os = "linux"))]
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    event_loop.run_app(&mut Application::default()).unwrap();
}
