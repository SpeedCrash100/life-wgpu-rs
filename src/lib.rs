use winit::{event_loop::EventLoop, window::WindowBuilder};

mod event_chain;
use event_chain::*;

mod app;
use app::App;

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let mut window = WindowBuilder::new()
        .with_title("Life")
        .build(&event_loop)
        .expect("Failed to create window");

    let app = App::new(&window).await;

    let mut event_chain_handlers: [EventChainElementBox; 2] = [
        Box::new(CloseHandler::new()),
        Box::new(DrawHandler::new(app)),
    ];

    event_loop.run(move |event, _, control_flow| {
        for event_hnd in event_chain_handlers.iter_mut() {
            if event_hnd.handle(&event, &mut window, control_flow) {
                break;
            }
        }
    });
}
