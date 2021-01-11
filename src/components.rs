use std::time::{Duration, Instant};

use specs::*;
use winit::event::Event;

use crate::render::models::Mesh;

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

#[derive(Component)]
#[storage(VecStorage)]
pub struct Renderable {
    pub mesh: Mesh,
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
