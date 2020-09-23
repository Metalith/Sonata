#[macro_use]
extern crate log;

mod control;
mod movement;
mod render;

use control::{ControlData, ControlSystem};
use movement::MoveSystem;
use render::RenderSystem;

use specs::*;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

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

fn main() {
    env_logger::init();
    debug!("Program started");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Rusty Sonata").build(&event_loop).unwrap();
    let mut input = WinitInputHelper::new();

    let mut world = World::new();
    world.register::<Player>();
    world.register::<Transform>();
    world.register::<Movement>();

    world.insert(ControlData::default()); // Let's use some start value

    world.create_entity().with(Player).with(Movement::default()).with(Transform::default()).build();

    let mut dispatcher = DispatcherBuilder::new()
        .with(ControlSystem::default(), "Control", &[])
        .with(MoveSystem::default(), "Move", &[])
        .with_thread_local(RenderSystem::new(window))
        .build();

    event_loop.run(move |event, _, control_flow| {
        ControlSystem::update(&mut world, &event, &mut input);
        match event {
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
            Event::MainEventsCleared => {
                dispatcher.dispatch(&mut world);
                world.maintain();
            }
            _ => (),
        }
    });
}
