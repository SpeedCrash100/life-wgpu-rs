pub type EventChainElementBox = Box<dyn EventChainElement>;
pub type Event<'a> = winit::event::Event<'a, ()>;

pub use winit::event_loop::ControlFlow;

mod close_handler;
pub use close_handler::CloseHandler;

mod draw_handler;
pub use draw_handler::DrawHandler;
pub use draw_handler::DrawHandlerSubscriber;

mod keyboard_handler;
pub use keyboard_handler::KeyboardHandler;
pub use keyboard_handler::KeyboardHandlerSubscriber;

use winit::window::Window;

pub trait EventChainElement {
    fn handle(
        &mut self,
        event: &Event,
        window: &mut Window,
        control_flow: &mut ControlFlow,
    ) -> bool;
}
