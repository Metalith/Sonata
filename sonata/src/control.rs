use specs::*;
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

use crate::{Movement, Player, WinitEventData};

pub struct ControlSystem {
    input: WinitInputHelper,
}

impl ControlSystem {
    pub fn new() -> Self {
        Self { input: WinitInputHelper::new() }
    }
}

impl<'a> System<'a> for ControlSystem {
    type SystemData = (Read<'a, WinitEventData>, ReadStorage<'a, Player>, WriteStorage<'a, Movement>);

    fn run(&mut self, (event_queue, player_storage, mut movement_storage): Self::SystemData) {
        for event in &event_queue.events {
            self.input.update(&event);
        }

        // TODO: Replace this with a more physics based approach
        // TODO: Use time delta to set this
        let mut dir = [0.0, 0.0, 0.0];
        if self.input.key_held(VirtualKeyCode::S) {
            dir[1] = -1.0;
        } else if self.input.key_held(VirtualKeyCode::W) {
            dir[1] = 1.0;
        }

        if self.input.key_held(VirtualKeyCode::A) {
            dir[0] = -1.0;
        } else if self.input.key_held(VirtualKeyCode::D) {
            dir[0] = 1.0;
        }

        if self.input.key_held(VirtualKeyCode::LShift) {
            dir[2] = -1.0;
        } else if self.input.key_held(VirtualKeyCode::Space) {
            dir[2] = 1.0;
        }

        for (_, m) in (&player_storage, &mut movement_storage).join() {
            m.vel = dir;
        }
    }
}
