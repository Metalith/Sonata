use specs::{Join, Read, ReadStorage, System, WriteStorage};

use crate::{uv::Lerp, DeltaTime, Movement, TimeStep, Transform};

pub struct MoveSystem {
    timestep: TimeStep,
}

impl MoveSystem {
    pub fn new() -> Self {
        Self { timestep: TimeStep::new(1000) }
    }
}

impl<'a> System<'a> for MoveSystem {
    type SystemData = (Read<'a, DeltaTime>, WriteStorage<'a, Transform>, ReadStorage<'a, Movement>);

    fn run(&mut self, (delta_time, mut transform_storage, vel_storage): Self::SystemData) {
        // let (stepped, delta) = self.timestep.step();
        // if stepped {
        let delta = delta_time.delta.as_secs_f32();
        for (transform, vel) in (&mut transform_storage, &vel_storage).join() {
            transform.pos[0] += vel.vel[0] * delta;
            transform.pos[1] += vel.vel[1] * delta;
            transform.pos[2] += vel.vel[2] * delta;

            let new_dir = transform.dir * vel.rot;
            transform.dir = transform.dir.lerp(new_dir, delta / (1.0 / self.timestep.fps)) * vel.rot;
        }
        // }
    }
}
