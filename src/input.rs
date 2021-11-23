use winit::dpi::PhysicalPosition;

#[derive(Default)]
pub struct Input {
    last_pos: (f32, f32),
    delta: (f32, f32),
}

impl Input {
    pub fn update(&mut self, new_position: PhysicalPosition<f64>) {
        let new_position = (new_position.x as f32, new_position.y as f32);
        self.delta = (
            new_position.0 - self.last_pos.0,
            -(new_position.1 - self.last_pos.1),
        );
        self.last_pos = new_position;
    }

    pub fn delta_x(&self) -> f32 {
        self.delta.0
    }

    pub fn delta_y(&self) -> f32 {
        self.delta.1
    }
}
