use specs::*;
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

use crate::{DeltaTime, Movement, Player, WinitEventData};

enum Direction {
    Forward,
    Horizontal,
    Vertical,
}

impl Direction {
    /// Returns keys used for direction in +, - order e.g. Forward, Backward
    pub fn keys(&self) -> (VirtualKeyCode, VirtualKeyCode) {
        match *self {
            Direction::Forward => (VirtualKeyCode::W, VirtualKeyCode::S),
            Direction::Horizontal => (VirtualKeyCode::D, VirtualKeyCode::A),
            Direction::Vertical => (VirtualKeyCode::Space, VirtualKeyCode::LShift),
        }
    }
}

pub struct ControlSystem {
    input: WinitInputHelper,
}

impl ControlSystem {
    pub fn new() -> Self {
        Self { input: WinitInputHelper::new() }
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
}

impl<'a> System<'a> for ControlSystem {
    type SystemData = (Read<'a, WinitEventData>, Read<'a, DeltaTime>, ReadStorage<'a, Player>, WriteStorage<'a, Movement>);

    fn run(&mut self, (event_queue, delta_time, player_storage, mut movement_storage): Self::SystemData) {
        for event in &event_queue.events {
            self.input.update(&event);
        }

        // TODO: Replace this with a more physics based approach
        let mut dir = [
            self.process_dir_key(Direction::Horizontal),
            self.process_dir_key(Direction::Forward),
            self.process_dir_key(Direction::Vertical),
        ];
        for i in &mut dir {
            *i *= delta_time.last_frame.elapsed().as_secs_f32();
        }

        for (_, m) in (&player_storage, &mut movement_storage).join() {
            m.vel = dir;
        }
    }
}
