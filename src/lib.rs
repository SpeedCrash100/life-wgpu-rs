use winit::{event_loop::EventLoop, window::WindowBuilder};

mod event_chain;
use event_chain::*;

mod app;
use app::App;

mod camera;

mod shader;

mod life;

mod model;

mod bindable;

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();

    let monitors = event_loop.available_monitors().next().unwrap();
    let mode = monitors
        .video_modes()
        .max_by_key(|m| m.size().height * m.size().width)
        .unwrap();

    let mut window = WindowBuilder::new()
        .with_title("Life")
        .with_fullscreen(Some(winit::window::Fullscreen::Exclusive(mode)))
        .build(&event_loop)
        .expect("Failed to create window");

    let app = App::new(&window).await;

    let mut event_chain_handlers: Vec<EventChainElementBox> = vec![
        Box::new(CloseHandler::new()),
        Box::new(DrawHandler::new(app.clone())),
        Box::new(KeyboardHandler::new(app.clone())),
    ];

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        for event_hnd in event_chain_handlers.iter_mut() {
            if event_hnd.handle(&event, &mut window, control_flow) {
                break;
            }
        }
    });
}
