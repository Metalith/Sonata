use specs::{Join, ReadStorage, System, WriteStorage};

use crate::{Movement, TimeStep, Transform};

pub struct MoveSystem {
    timestep: TimeStep,
}

impl MoveSystem {
    pub fn new() -> Self {
        Self { timestep: TimeStep::new(240) }
    }
}

impl<'a> System<'a> for MoveSystem {
    type SystemData = (WriteStorage<'a, Transform>, ReadStorage<'a, Movement>);

    fn run(&mut self, (mut transform_storage, vel_storage): Self::SystemData) {
        let (stepped, _) = self.timestep.step();
        if stepped {
            // TODO: This may need delta time to work correctly. Not sure
            for (transform, vel) in (&mut transform_storage, &vel_storage).join() {
                transform.pos[0] += vel.vel[0];
                transform.pos[1] += vel.vel[1];
                transform.pos[2] += vel.vel[2];

                transform.dir = transform.dir * vel.rot;
            }
        }
    }
}
