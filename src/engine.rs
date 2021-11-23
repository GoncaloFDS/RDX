use crate::camera::Camera;
use crate::input::Input;
use crate::time::Time;
use crate::vulkan::renderer::Renderer;
use crate::vulkan::scene::Scene;
use glam::{vec3, Vec3};
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, KeyboardInput, MouseButton};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct Engine {
    time: Time,
    window: Window,
    renderer: Renderer,
    scene: Scene,
    camera: Camera,
    input: Input,
}

impl Engine {
    pub fn new(width: u32, height: u32, name: &str) -> (Engine, EventLoop<()>) {
        let (window, event_loop) = Self::new_window(width, height, name);

        let mut renderer = Renderer::new();

        renderer.setup(&window);

        let scene = Scene::new();
        renderer.upload_meshes(&scene);

        let engine = Engine {
            time: Time::new(),
            window,
            renderer,
            scene,
            camera: Camera::new(vec3(0.0, 0.0, 1.0), Vec3::ZERO),
            input: Default::default(),
        };

        (engine, event_loop)
    }

    fn new_window(width: u32, height: u32, name: &str) -> (Window, EventLoop<()>) {
        log::debug!("Creating new Window");
        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title(name)
            .with_inner_size(LogicalSize::new(width, height))
            .with_resizable(true)
            .build(&event_loop)
            .unwrap();

        (window, event_loop)
    }

    pub fn resize(&mut self) {
        self.renderer.recreate_swapchain(&self.window);
    }

    pub fn run(&mut self) {
        self.camera.update_camera(self.time.delta_time());
        self.renderer.update(&self.camera);
        self.renderer.draw_frame();
        self.time.tick();
    }

    pub fn handle_key_input(&mut self, input: KeyboardInput) {
        if input.virtual_keycode.is_some() {
            self.camera.handle_input(input)
        }
    }

    pub fn handle_mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.input.update(position);
        self.camera.handle_mouse_move(
            self.input.delta_x(),
            self.input.delta_y(),
            self.time.delta_time(),
        );
    }

    pub fn handle_mouse_input(&mut self, input: MouseButton, state: ElementState) {
        self.camera.handle_mouse_input(input, state);
    }

    pub fn shutdown(&self) {
        self.renderer.shutdown();
    }
}
