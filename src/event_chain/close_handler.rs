use winit::event::WindowEvent;
use winit::window::Window;

use super::{ControlFlow, Event, EventChainElement};

pub struct CloseHandler {}

impl CloseHandler {
    pub fn new() -> Self {
        CloseHandler {}
    }
}

impl EventChainElement for CloseHandler {
    fn handle(
        &mut self,
        event: &Event,
        window: &mut Window,
        control_flow: &mut ControlFlow,
    ) -> bool {
        match event {
            Event::WindowEvent { window_id, event } if window.id() == *window_id => match event {
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }
}
