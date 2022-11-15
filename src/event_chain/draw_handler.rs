use std::sync::{Arc, Mutex, MutexGuard};

use winit::window::Window;
use winit::{dpi::PhysicalSize, event::WindowEvent};

use super::{ControlFlow, Event, EventChainElement};

pub trait DrawHandlerSubscriber {
    fn resize(&mut self, new_size: PhysicalSize<u32>);
    fn reconfigure(&mut self);
    fn draw(&mut self) -> Result<(), wgpu::SurfaceError>;
}

pub struct DrawHandler<S: DrawHandlerSubscriber> {
    subscriber: Arc<Mutex<S>>,
}

impl<S: DrawHandlerSubscriber> DrawHandler<S> {
    pub fn new(subscriber: Arc<Mutex<S>>) -> Self {
        DrawHandler { subscriber }
    }

    fn subscriber<'s>(&'s self) -> MutexGuard<'s, S> {
        self.subscriber.lock().unwrap()
    }
}

impl<S: DrawHandlerSubscriber> EventChainElement for DrawHandler<S> {
    fn handle(&mut self, event: &Event, window: &mut Window, _: &mut ControlFlow) -> bool {
        match event {
            Event::WindowEvent { window_id, event } if window.id() == *window_id => match event {
                WindowEvent::Resized(new_size) => {
                    self.subscriber().resize(*new_size);
                    true
                }
                WindowEvent::ScaleFactorChanged {
                    scale_factor: _,
                    new_inner_size,
                } => {
                    self.subscriber().resize(**new_inner_size);
                    true
                }
                _ => false,
            },
            Event::RedrawRequested(window_id) if window.id() == *window_id => {
                match self.subscriber().draw() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => self.subscriber().reconfigure(),
                    Err(e) => eprintln!("{:?}", e),
                }
                true
            }
            Event::MainEventsCleared => {
                window.request_redraw();
                true
            }
            _ => false,
        }
    }
}
