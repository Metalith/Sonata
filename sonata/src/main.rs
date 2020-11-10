#[macro_use]
extern crate log;
extern crate ultraviolet as uv;

mod components;
mod control;
mod entity_factory;
mod movement;
mod render;
mod timestep;

use std::time::Instant;

pub use components::*;
use entity_factory::EntityFactory;
pub use timestep::*;

use control::ControlSystem;
use movement::MoveSystem;
use render::RenderSystem;

use specs::*;
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::windows::WindowExtWindows,
    window::WindowBuilder,
};

fn main() {
    env_logger::init();
    debug!("Program started");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Rusty Sonata").build(&event_loop).unwrap();

    let renderer = sketch::Renderer::new(window.hwnd(), window.hinstance());
    let entity_factory = EntityFactory::new(renderer.create_mesh_factory());

    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new()
        .with(ControlSystem::new(), "Control", &[])
        .with(MoveSystem::new(), "Move", &[])
        .with_thread_local(RenderSystem::new(window, renderer))
        .build();
    dispatcher.setup(&mut world);

    // Player
    entity_factory.create_player(&mut world, [0.0, 0.0, 4.0]);

    // XY Grid
    entity_factory.create_grid(&mut world);

    let mut last_frame = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        if let Some(event) = event.to_static() {
            world.write_resource::<WinitEventData>().events.push(event.clone());
            match event {
                Event::NewEvents(_) => {
                    world.write_resource::<DeltaTime>().delta = last_frame.elapsed();
                    last_frame = Instant::now();
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => {
                        if input.virtual_keycode == Some(VirtualKeyCode::Escape) && input.state == ElementState::Pressed {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    _ => (),
                },
                Event::MainEventsCleared => {
                    dispatcher.dispatch(&world);
                    world.maintain();
                    world.write_resource::<WinitEventData>().events.clear();
                }
                _ => (),
            }
        }
    });
}
