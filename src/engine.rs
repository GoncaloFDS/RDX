use crate::camera::Camera;
use crate::input::Input;
use crate::time::Time;
use crate::user_interface::UserInterface;
use crate::vulkan::renderer::Renderer;
use crate::vulkan::scene::Scene;
use egui_winit::winit::event::Event;
use egui_winit::winit::event_loop::ControlFlow;
use egui_winit::State;
use glam::{vec3, Vec3};
use std::rc::Rc;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct Engine {
    time: Time,
    window: Rc<Window>,
    renderer: Renderer,
    scene: Scene,
    camera: Camera,
    input: Input,
    ui: UserInterface,
}

impl Engine {
    pub fn new(width: u32, height: u32, name: &str) -> (Engine, EventLoop<()>) {
        puffin::set_scopes_on(true);
        let (window, event_loop) = Self::new_window(width, height, name);
        let window = Rc::new(window);

        let mut renderer = Renderer::new();

        let scene = Scene::new();
        renderer.upload_meshes(&scene);

        renderer.setup(&window);

        let ui = UserInterface::new(window.clone());

        let engine = Engine {
            time: Time::new(),
            window,
            renderer,
            scene,
            camera: Camera::new(vec3(0.0, 2.0, 5.0), Vec3::ZERO),
            input: Default::default(),
            ui,
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

    pub fn on_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: window_event,
                ..
            } => {
                let ui_captured_input = self.ui.on_event(&window_event);

                if ui_captured_input {
                    return;
                }

                match window_event {
                    WindowEvent::Resized(size) => {
                        log::info!("Resizing window: {}x{}", size.width, size.height);
                        self.resize();
                    }
                    WindowEvent::CloseRequested => {
                        log::debug!("Closing window");
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                            self.shutdown();
                            *control_flow = ControlFlow::Exit
                        } else if input.virtual_keycode == Some(VirtualKeyCode::F1)
                            && input.state == ElementState::Pressed
                        {
                            self.ui.toggle_settings();
                        } else if input.virtual_keycode == Some(VirtualKeyCode::F2)
                            && input.state == ElementState::Pressed
                        {
                            self.ui.toggle_profiler();
                        } else {
                            self.handle_key_input(input)
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        self.handle_mouse_move(position);
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        self.handle_mouse_input(button, state);
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                self.run();
            }
            _ => (),
        }
    }

    pub fn resize(&mut self) {
        self.renderer.recreate_swapchain(&self.window);
    }

    pub fn run(&mut self) {
        puffin::GlobalProfiler::lock().new_frame();
        puffin::profile_function!();
        self.camera.update_camera(self.time.delta_time());
        self.renderer.update(&self.camera, &mut self.ui);
        self.renderer.draw_frame();
        self.renderer.present_frame();
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
