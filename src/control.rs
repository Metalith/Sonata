use specs::*;
use winit::event::{Event, VirtualKeyCode};
use winit_input_helper::WinitInputHelper;

use crate::{Movement, Player};

#[derive(Default)]
pub struct ControlData {
    pub dir: [f32; 3],
}

#[derive(Default)]
pub struct ControlSystem {}

impl ControlSystem {
    pub fn update(world: &mut World, event: &Event<()>, input: &mut WinitInputHelper) {
        if input.update(event) {
            let mut dir = [0.0, 0.0, 0.0];
            if input.key_held(VirtualKeyCode::S) {
                dir[1] = -1.0;
            } else if input.key_held(VirtualKeyCode::W) {
                dir[1] = 1.0;
            }

            if input.key_held(VirtualKeyCode::A) {
                dir[0] = -1.0;
            } else if input.key_held(VirtualKeyCode::D) {
                dir[0] = 1.0;
            }

            if input.key_held(VirtualKeyCode::LShift) {
                dir[2] = -1.0;
            } else if input.key_held(VirtualKeyCode::Space) {
                dir[2] = 1.0;
            }

            *world.write_resource() = ControlData { dir };
        }
    }
}

impl<'a> System<'a> for ControlSystem {
    type SystemData = (Read<'a, ControlData>, ReadStorage<'a, Player>, WriteStorage<'a, Movement>);

    fn run(&mut self, (control_data, player_storage, mut movement_storage): Self::SystemData) {
        for (_, m) in (&player_storage, &mut movement_storage).join() {
            m.vel = control_data.dir;
        }
    }
}
