#[macro_use]
extern crate log;

mod render;

use render::RenderSystem;

use specs::*;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Component, Default, Debug)]
#[storage(NullStorage)]
struct Player;

fn main() {
    env_logger::init();
    debug!("Program started");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Rusty Sonata").build(&event_loop).unwrap();

    let mut world = World::new();
    world.register::<Player>();

    world.create_entity().with(Player).build();

    let mut dispatcher = DispatcherBuilder::new().with_thread_local(RenderSystem::new(window)).build();

    // let mut e = wind::World::new(Events::Update);
    // e.create_system(ControlSystem::default()).with_component::<Player>().build();
    // e.create_system(RenderSystem::new(window)).build();

    // e.create_entity().components(wind::ComponentBuilder::new().with(Player::default()).build()).build();

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
        Event::MainEventsCleared => {
            dispatcher.dispatch(&mut world);
            world.maintain();
        }
        _ => (),
    });
}
