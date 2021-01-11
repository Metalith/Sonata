#[macro_use]
extern crate log;
extern crate ultraviolet as uv;

mod components;
mod entity_factory;
mod render;
mod systems;

use std::time::Instant;

use components::*;
use entity_factory::EntityFactory;
use render::GraphicContext;
use systems::*;

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
    let window = WindowBuilder::new().with_title("Voyager 0.01").build(&event_loop).unwrap();

    let graphic_context = GraphicContext::new(window.hwnd(), window.hinstance());
    let entity_factory = EntityFactory::new(graphic_context.create_mesh_factory());

    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new()
        .with(ControlSystem::new(), "Control", &[])
        .with(MoveSystem::new(), "Move", &[])
        .with_thread_local(RenderSystem::new(window, graphic_context))
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
