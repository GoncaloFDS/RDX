use std::time::Instant;

pub struct Time {
    now: Instant,
}

impl Time {
    pub fn new() -> Time {
        Time {
            now: Instant::now(),
        }
    }

    pub fn delta_time(&self) -> f32 {
        self.now.elapsed().as_secs_f32()
    }

    pub fn tick(&mut self) {
        self.now = Instant::now();
    }
}
