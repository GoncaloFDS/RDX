use glam::{const_mat4, EulerRot, Mat4, Quat, Vec3, Vec4};
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
    right: Vec4,
    up: Vec4,
    forward: Vec4,
    orientation: Quat,

    yaw: f32,
    pitch: f32,

    moving_forward: bool,
    moving_back: bool,
    moving_right: bool,
    moving_left: bool,
    moving_up: bool,
    moving_down: bool,

    mouse_left_pressed: bool,

    look_speed: f32,
    move_speed: f32,
}

impl Camera {
    pub fn new(eye: Vec3, center: Vec3) -> Self {
        let view = Mat4::look_at_rh(eye, center, Vec3::Y);

        let inverse_view = view.inverse();
        let (_, orientation, position) = inverse_view.to_scale_rotation_translation();

        let right = inverse_view * Vec4::X;
        let up = inverse_view * Vec4::Y;
        let forward = inverse_view * -Vec4::Z;

        let (pitch, yaw, _) = orientation.to_euler(EulerRot::XYZ);

        Camera {
            position,
            right,
            up,
            forward,
            orientation,
            yaw,
            pitch,
            moving_forward: false,
            moving_back: false,
            moving_right: false,
            moving_left: false,
            moving_up: false,
            moving_down: false,
            mouse_left_pressed: false,
            look_speed: 10.0,
            move_speed: 100.0,
        }
    }

    pub fn view(&self) -> Mat4 {
        Mat4::from_quat(self.orientation).inverse()
            * Mat4::from_translation(self.position).inverse()
    }

    pub fn projection(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_lh(60.0f32.to_radians(), aspect_ratio, 0.01, 10000.0) * X_MAT
    }

    pub fn handle_input(&mut self, input: KeyboardInput) {
        match input.virtual_keycode.unwrap() {
            VirtualKeyCode::W => self.moving_forward = input.state == ElementState::Pressed,
            VirtualKeyCode::S => self.moving_back = input.state == ElementState::Pressed,
            VirtualKeyCode::A => self.moving_left = input.state == ElementState::Pressed,
            VirtualKeyCode::D => self.moving_right = input.state == ElementState::Pressed,
            VirtualKeyCode::Q => self.moving_down = input.state == ElementState::Pressed,
            VirtualKeyCode::E => self.moving_up = input.state == ElementState::Pressed,
            _ => {}
        }
    }

    pub fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        match button {
            MouseButton::Left => self.mouse_left_pressed = state == ElementState::Pressed,
            MouseButton::Right => {}
            MouseButton::Middle => {}
            MouseButton::Other(_) => {}
        }
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32, delta_time: f32) {
        if self.mouse_left_pressed {
            self.yaw += x * self.look_speed * delta_time;
            self.pitch += y * self.look_speed * delta_time;
            self.orientation = Quat::from_axis_angle(-Vec3::Y, self.yaw)
                * Quat::from_axis_angle(Vec3::X, self.pitch);

            self.update_vectors();
        }
    }

    fn update_vectors(&mut self) {
        let rotation = Mat4::from_quat(self.orientation);
        self.right = rotation * Vec4::X;
        self.up = rotation * Vec4::Y;
        self.forward = rotation * -Vec4::Z;
    }

    pub fn update_camera(&mut self, delta_time: f32) -> bool {
        let move_amount = delta_time * self.move_speed;
        if self.moving_forward {
            self.move_forward(move_amount)
        }
        if self.moving_back {
            self.move_forward(-move_amount)
        }
        if self.moving_right {
            self.move_right(move_amount)
        }
        if self.moving_left {
            self.move_right(-move_amount)
        }
        if self.moving_up {
            self.move_up(move_amount)
        }
        if self.moving_down {
            self.move_up(-move_amount)
        }

        let updated = self.moving_forward
            || self.moving_back
            || self.moving_left
            || self.moving_right
            || self.moving_up
            || self.moving_down;

        updated
    }

    fn move_forward(&mut self, amount: f32) {
        self.position += amount * self.forward.truncate();
    }

    fn move_right(&mut self, amount: f32) {
        self.position += amount * self.right.truncate();
    }

    fn move_up(&mut self, amount: f32) {
        self.position += amount * self.up.truncate();
    }
}
