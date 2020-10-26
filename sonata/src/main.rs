#[macro_use]
extern crate log;
extern crate ultraviolet as uv;

mod components;
mod control;
mod model;
mod movement;
mod render;
mod timestep;

use std::time::Instant;

pub use components::*;
pub use timestep::*;

use control::ControlSystem;
use movement::MoveSystem;
use render::RenderSystem;

use specs::*;
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();
    debug!("Program started");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Rusty Sonata").build(&event_loop).unwrap();

    let mut world = World::new();
    world.register::<Player>();
    world.register::<Transform>();
    world.register::<Movement>();

    world.insert(DeltaTime::default());
    world.insert(WinitEventData::default());
    world.insert(ControlData::default());

    // Player
    world
        .create_entity()
        .with(Player::default())
        .with(Movement::default())
        .with(Transform {
            pos: uv::Vec3::new(0.0, 0.0, 4.0),
            dir: uv::Rotor3::from_euler_angles(0.0f32.to_radians(), 0.0, 180.0f32.to_radians()), // Look at center from above due to colinearity
        })
        .build();

    // XY Grid

    let mut dispatcher = DispatcherBuilder::new()
        .with(ControlSystem::new(), "Control", &[])
        .with(MoveSystem::new(), "Move", &[])
        .with_thread_local(RenderSystem::new(window))
        .build();

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
