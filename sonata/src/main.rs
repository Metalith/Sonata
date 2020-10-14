#[macro_use]
extern crate log;
extern crate ultraviolet as uv;

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MouseState {
    Ui,
    Fly,
}

impl Default for MouseState {
    fn default() -> Self {
        MouseState::Ui
    }
}

#[derive(Component, Default, Debug)]
#[storage(NullStorage)]
pub struct Player;

#[derive(Component, Default, Debug)]
#[storage(VecStorage)]
pub struct Movement {
    pub vel: uv::Vec3,
    pub rot: uv::Rotor3,
}

#[derive(Component, Default, Debug)]
#[storage(VecStorage)]
pub struct Transform {
    pub pos: uv::Vec3,
    pub dir: uv::Rotor3,
}

#[derive(Debug, Default)]
pub struct ControlData {
    pub set_mouse: bool,
    pub last_mouse_pos: (f32, f32),
    pub mouse_state: MouseState,
}

pub struct DeltaTime {
    pub delta: Duration,
    pub start_time: Instant,
}

impl Default for DeltaTime {
    fn default() -> Self {
        Self {
            delta: Duration::default(),
            start_time: Instant::now(),
        }
    }
}

#[derive(Default)]
pub struct WinitEventData {
    pub events: Vec<Event<'static, ()>>,
}

struct TimeStep {
    fps_delta: f32,
    last_step: Instant,
}

impl TimeStep {
    pub fn new(fps: i32) -> Self {
        Self {
            fps_delta: 1.0 / fps as f32,
            last_step: Instant::now(),
        }
    }
    pub fn step(&mut self) -> (bool, f32) {
        let delta = self.last_step.elapsed().as_secs_f32();
        if delta > self.fps_delta {
            self.last_step = Instant::now();
            (true, delta)
        } else {
            (false, 0.0)
        }
    }
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
    world.insert(ControlData::default());

    // Make generic look at for all axes
    world
        .create_entity()
        .with(Player::default())
        .with(Movement::default())
        .with(Transform {
            pos: uv::Vec3::new(0.0, 0.0, 4.0),
            dir: uv::Rotor3::from_euler_angles(0.0f32.to_radians(), 0.0, 180.0f32.to_radians()), // Look at center from above due to colinearity
        })
        .build();

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
