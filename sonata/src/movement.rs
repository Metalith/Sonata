use specs::{Join, ReadStorage, System, WriteStorage};

use crate::{Movement, Transform};

#[derive(Default)]
pub struct MoveSystem;

impl<'a> System<'a> for MoveSystem {
    type SystemData = (WriteStorage<'a, Transform>, ReadStorage<'a, Movement>);

    fn run(&mut self, (mut transform_storage, vel_storage): Self::SystemData) {
        for (transform, vel) in (&mut transform_storage, &vel_storage).join() {
            transform.pos[0] += vel.vel[0];
            transform.pos[1] += vel.vel[1];
            transform.pos[2] += vel.vel[2];
        }
    }
}
