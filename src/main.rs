#[macro_use]
extern crate log;

mod render;

use render::RenderSystem;

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();
    debug!("Program started");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Rusty Sonata").build(&event_loop).unwrap();

    let mut e = wind::Engine::new();
    e.add_system(RenderSystem::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput { virtual_keycode, state, .. } => match (virtual_keycode, state) {
                    (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                        dbg!();
                        *control_flow = ControlFlow::Exit
                    }
                    _ => {}
                },
            },
            _ => {}
        },
        Event::EventsCleared => {
            e.update();
        }
        _ => (),
    });
}
