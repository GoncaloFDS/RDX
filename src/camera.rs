use glam::*;
use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};

// Intermediate transformation that aligns the axes with the expected Vulkan
const X_MAT: Mat4 = const_mat4!(
    [1., 0., 0., 0.],
    [0., -1., 0., 0.],
    [0., 0., -1., 0.],
    [0., 0., 0., 1.]
);

pub struct Camera {
    position: Vec3,
    orientation: Quat,
    movement: Movement,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    look_speed: f32,
    move_speed: f32,
    mouse_left_pressed: bool,
}

#[derive(Default)]
struct Movement {
    forward: bool,
    back: bool,
    right: bool,
    left: bool,
    up: bool,
    down: bool,
}

impl Camera {
    pub fn new(eye: Vec3, center: Vec3) -> Self {
        let view = Mat4::look_at_rh(eye, center, Vec3::Y);
        let orientation = Quat::from_mat4(&view);

        let v = Mat4::from_quat(orientation).to_cols_array_2d();
        let forward = -vec3(v[0][2], v[1][2], v[2][2]);
        let right = vec3(v[0][0], v[1][0], v[2][0]);
        let up = right.cross(forward);

        Camera {
            position: eye,
            orientation,
            movement: Default::default(),
            forward,
            right,
            up,
            look_speed: 1.0,
            move_speed: 40.0,
            mouse_left_pressed: false,
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn view(&self) -> Mat4 {
        let t = Mat4::from_translation(-self.position);
        let r = Mat4::from_quat(self.orientation);
        r * t
    }

    pub fn projection(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_lh(60.0f32.to_radians(), aspect_ratio, 0.01, 10000.0) * X_MAT
    }

    pub fn handle_input(&mut self, input: KeyboardInput) {
        match input.virtual_keycode.unwrap() {
            VirtualKeyCode::W => self.movement.forward = input.state == ElementState::Pressed,
            VirtualKeyCode::S => self.movement.back = input.state == ElementState::Pressed,
            VirtualKeyCode::A => self.movement.left = input.state == ElementState::Pressed,
            VirtualKeyCode::D => self.movement.right = input.state == ElementState::Pressed,
            VirtualKeyCode::Q => self.movement.down = input.state == ElementState::Pressed,
            VirtualKeyCode::E => self.movement.up = input.state == ElementState::Pressed,
            _ => {}
        }
    }

    pub fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        if button == MouseButton::Left {
            self.mouse_left_pressed = state == ElementState::Pressed
        }
    }

    pub fn handle_mouse_move(&mut self, dx: f32, dy: f32, delta_time: f32) {
        if self.mouse_left_pressed {
            let xa = -dy * self.look_speed * delta_time;
            let ya = dx * self.look_speed * delta_time;
            let delta_quat = Quat::from_vec4(vec4(xa, ya, 0.0, 1.0));
            self.orientation = (delta_quat * self.orientation).normalize();
            self.set_up(Vec3::Y);
            self.update_vectors();
        }
    }

    fn set_up(&mut self, up: Vec3) {
        let view = self.view().to_cols_array_2d();
        let dir = -vec3(view[0][2], view[1][2], view[2][2]);
        self.orientation =
            Quat::from_mat4(&Mat4::look_at_rh(self.position, self.position + dir, up));
    }

    fn update_vectors(&mut self) {
        let v = Mat4::from_quat(self.orientation).to_cols_array_2d();
        self.forward = -vec3(v[0][2], v[1][2], v[2][2]);
        self.right = vec3(v[0][0], v[1][0], v[2][0]);
        self.up = self.right.cross(self.forward);
    }

    pub fn update_camera(&mut self, delta_time: f32) -> bool {
        // FIXME: use delta_time
        let move_amount = delta_time * self.move_speed;
        if self.movement.forward {
            self.move_forward(move_amount)
        }
        if self.movement.back {
            self.move_forward(-move_amount)
        }
        if self.movement.right {
            self.move_right(move_amount)
        }
        if self.movement.left {
            self.move_right(-move_amount)
        }
        if self.movement.up {
            self.move_up(move_amount)
        }
        if self.movement.down {
            self.move_up(-move_amount)
        }

        self.movement.forward
            || self.movement.back
            || self.movement.left
            || self.movement.right
            || self.movement.up
            || self.movement.down
    }

    fn move_forward(&mut self, amount: f32) {
        self.position += amount * self.forward;
    }

    fn move_right(&mut self, amount: f32) {
        self.position += amount * self.right;
    }

    fn move_up(&mut self, amount: f32) {
        self.position += amount * self.up;
    }
}
