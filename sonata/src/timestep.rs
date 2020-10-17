use std::time::Instant;

pub struct TimeStep {
    pub fps: f32,
    fps_delta: f32,
    last_step: Instant,
}

impl TimeStep {
    pub fn new(fps: i32) -> Self {
        Self {
            fps: fps as f32,
            fps_delta: 1.0 / fps as f32,
            last_step: Instant::now(),
        }
    }
    pub fn step(&mut self) -> (bool, f32) {
        let delta = self.last_step.elapsed().as_secs_f32();
        if delta > self.fps_delta {
            self.last_step = Instant::now();
            (true, delta)
        } else {
            (false, 0.0)
        }
    }
}
