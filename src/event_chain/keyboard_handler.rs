use std::sync::{Arc, Mutex, MutexGuard};

use log::info;
use winit::event::WindowEvent;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use winit::window::Window;

use super::{ControlFlow, Event, EventChainElement};

pub trait KeyboardHandlerSubscriber {
    fn key_pressed(&mut self, key_code: &VirtualKeyCode);
}

pub struct KeyboardHandler<S: KeyboardHandlerSubscriber> {
    subscriber: Arc<Mutex<S>>,
}

impl<S: KeyboardHandlerSubscriber> KeyboardHandler<S> {
    pub fn new(subscriber: Arc<Mutex<S>>) -> Self {
        KeyboardHandler { subscriber }
    }

    fn subscriber<'s>(&'s self) -> MutexGuard<'s, S> {
        self.subscriber.lock().unwrap()
    }
}

impl<S: KeyboardHandlerSubscriber> EventChainElement for KeyboardHandler<S> {
    fn handle(&mut self, event: &Event, window: &mut Window, _: &mut ControlFlow) -> bool {
        match event {
            Event::WindowEvent { window_id, event } if window.id() == *window_id => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode,
                            scancode,
                            ..
                        },
                    ..
                } => {
                    if let Some(k) = virtual_keycode {
                        self.subscriber().key_pressed(k);
                    } else {
                        info!("Ignoring button scancode: {}", scancode);
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }
}
