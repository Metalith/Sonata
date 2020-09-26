#[macro_use]
extern crate log;

mod control;
mod movement;
mod render;

use std::time::{Duration, Instant};

use control::ControlSystem;
use movement::MoveSystem;
use render::RenderSystem;

use specs::*;
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Component, Default, Debug)]
#[storage(NullStorage)]
pub struct Player;

#[derive(Component, Default, Debug)]
#[storage(VecStorage)]
pub struct Movement {
    vel: [f32; 3],
}

#[derive(Component, Default, Debug)]
#[storage(VecStorage)]
pub struct Transform {
    pos: [f32; 3],
}

pub struct DeltaTime {
    pub last_frame: Instant, // Workaround for current version of imgui-rs
    pub delta: Duration,
}

impl Default for DeltaTime {
    fn default() -> Self {
        Self {
            last_frame: Instant::now(),
            delta: Duration::default(),
        }
    }
}

#[derive(Default)]
pub struct WinitEventData {
    pub events: Vec<Event<'static, ()>>,
}

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

    world.create_entity().with(Player).with(Movement::default()).with(Transform::default()).build();

    let mut dispatcher = DispatcherBuilder::new()
        .with(ControlSystem::new(), "Control", &[])
        .with(MoveSystem::default(), "Move", &[])
        .with_thread_local(RenderSystem::new(window))
        .build();

    let mut last_frame = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        if let Some(event) = event.to_static() {
            world.write_resource::<WinitEventData>().events.push(event.clone());
            match event {
                Event::NewEvents(_) => {
                    let now = Instant::now();
                    world.write_resource::<DeltaTime>().delta = now - last_frame;
                    world.write_resource::<DeltaTime>().last_frame = last_frame;
                    last_frame = now;
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
