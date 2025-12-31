mod app;
mod config;
mod ssh;
mod terminal;

use app::App;
use muda::MenuEvent;
use tao::event_loop::{ControlFlow, EventLoopBuilder};

enum UserEvent {
    MenuEvent(MenuEvent),
}

fn main() {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let mut app = App::default();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            tao::event::Event::NewEvents(tao::event::StartCause::Init) => {
                app.init();
            }
            tao::event::Event::UserEvent(UserEvent::MenuEvent(event)) => {
                if app.handle_menu_event(event) {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}
