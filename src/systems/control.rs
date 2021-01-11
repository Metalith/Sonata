use specs::*;
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

use crate::{ControlData, MouseState, Movement, Player, Transform, WinitEventData};

use super::timestep::TimeStep;

enum Direction {
    Forward,
    Horizontal,
    Vertical,
    Roll,
}

impl Direction {
    /// Returns keys used for direction in +, - order e.g. Forward, Backward
    pub fn keys(&self) -> (VirtualKeyCode, VirtualKeyCode) {
        match *self {
            Direction::Forward => (VirtualKeyCode::W, VirtualKeyCode::S),
            Direction::Horizontal => (VirtualKeyCode::A, VirtualKeyCode::D),
            Direction::Vertical => (VirtualKeyCode::Space, VirtualKeyCode::LShift),
            Direction::Roll => (VirtualKeyCode::Q, VirtualKeyCode::E),
        }
    }
}

pub struct ControlSystem {
    input: WinitInputHelper,
    mouse_speed: f32,
    timestep: TimeStep,
}

impl ControlSystem {
    pub fn new() -> Self {
        Self {
            input: WinitInputHelper::new(),
            mouse_speed: 0.01,
            timestep: TimeStep::new(300),
        }
    }
}

impl ControlSystem {
    fn process_dir_key(&self, dir: Direction) -> f32 {
        let (forward, backward) = dir.keys();
        if self.input.key_held(forward) {
            1.0
        } else if self.input.key_held(backward) {
            -1.0
        } else {
            0.0
        }
    }

    fn get_mouse_delta(&self, control_data: &ControlData) -> (f32, f32) {
        if let Some((curr_x, curr_y)) = self.input.mouse() {
            (curr_x - control_data.last_mouse_pos.0, curr_y - control_data.last_mouse_pos.1)
        } else {
            (0.0, 0.0)
        }
    }
}

impl<'a> System<'a> for ControlSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Read<'a, WinitEventData>,
        Write<'a, ControlData>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Movement>,
    );

    fn run(&mut self, (event_queue, mut control_data, transform_storage, mut player_storage, mut movement_storage): Self::SystemData) {
        for event in &event_queue.events {
            self.input.update(&event);
        }
        // Mouse Control
        if self.input.key_pressed(VirtualKeyCode::Tab) {
            control_data.last_mouse_pos = self.input.mouse().unwrap_or((0.0, 0.0));
            control_data.mouse_state = match control_data.mouse_state {
                MouseState::Ui => MouseState::Fly,
                MouseState::Fly => MouseState::Ui,
            };
        }

        let (stepped, delta) = self.timestep.step();
        if stepped {
            for (_, movement, transform) in (&mut player_storage, &mut movement_storage, &transform_storage).join() {
                // Velocity
                // TODO: Replace this with a more physics based approach
                let mut dir_vec = uv::Vec3::new(
                    self.process_dir_key(Direction::Horizontal),
                    self.process_dir_key(Direction::Vertical),
                    self.process_dir_key(Direction::Forward),
                );
                transform.dir.rotate_vec(&mut dir_vec);
                movement.vel = dir_vec;

                // Rotation
                if control_data.mouse_state == MouseState::Fly {
                    let (delta_x, delta_y) = self.get_mouse_delta(&control_data);
                    let roll = self.process_dir_key(Direction::Roll);

                    let yaw_rot = uv::Rotor3::from_rotation_xz((delta_x * self.mouse_speed).to_radians());
                    let pitch_rot = uv::Rotor3::from_rotation_yz((delta_y * self.mouse_speed).to_radians());
                    let roll_rot = uv::Rotor3::from_rotation_xy((roll * 360.0 * delta).to_radians());

                    movement.rot = roll_rot * yaw_rot * pitch_rot;
                } else {
                    movement.rot = uv::Rotor3::default();
                }
            }
            if control_data.mouse_state == MouseState::Fly {
                control_data.set_mouse = true;
            };
        }
    }
}
