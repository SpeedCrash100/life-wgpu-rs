use std::sync::{Arc, Mutex, MutexGuard};

use winit::dpi::PhysicalPosition;
use winit::event::ElementState;
use winit::event::{MouseButton, WindowEvent};
use winit::window::Window;

use super::{ControlFlow, Event, EventChainElement};

pub trait MouseHandlerSubscriber {
    fn clicked(&mut self, position: PhysicalPosition<f64>);
}

pub struct MouseHandler<S: MouseHandlerSubscriber> {
    position: PhysicalPosition<f64>,
    subscriber: Arc<Mutex<S>>,
}

impl<S: MouseHandlerSubscriber> MouseHandler<S> {
    pub fn new(subscriber: Arc<Mutex<S>>) -> Self {
        let position = (0.0, 0.0).into();

        Self {
            subscriber,
            position,
        }
    }

    fn subscriber<'s>(&'s self) -> MutexGuard<'s, S> {
        self.subscriber.lock().unwrap()
    }
}

impl<S: MouseHandlerSubscriber> EventChainElement for MouseHandler<S> {
    fn handle(&mut self, event: &Event, window: &mut Window, _: &mut ControlFlow) -> bool {
        match event {
            Event::WindowEvent { window_id, event } if window.id() == *window_id => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.position = position.clone();
                    true
                }
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    ..
                } => {
                    self.subscriber().clicked(self.position);
                    true
                }
                _ => false,
            },

            _ => false,
        }
    }
}
