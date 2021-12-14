use crate::camera::Camera;
use crate::chunk::{Chunk, CHUNK_SIZE, MAP_SIZE};
use crate::input::Input;
use crate::time::Time;
use crate::user_interface::UserInterface;
use crate::vulkan::renderer::Renderer;
use crate::vulkan::scene::Scene;
use egui_winit::winit::event::Event;
use egui_winit::winit::event_loop::ControlFlow;
use glam::*;
use hecs::World;
use rayon::prelude::*;
use std::rc::Rc;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

fn spawn_entities(world: &mut World) {
    let chunks_to_spawn: Vec<_> = (0..MAP_SIZE)
        .collect::<Vec<_>>()
        .par_iter()
        .flat_map(|&x| {
            (0..MAP_SIZE)
                .into_iter()
                .map(|z| (Chunk::new(ivec3(x * CHUNK_SIZE, 0, z * CHUNK_SIZE)),))
                .collect::<Vec<_>>()
        })
        .collect();

    world.spawn_batch(chunks_to_spawn);
}

pub struct Engine {
    time: Time,
    window: Rc<Window>,
    renderer: Renderer,
    scene: Scene,
    camera: Camera,
    input: Input,
    ui: UserInterface,
    world: World,
}

impl Engine {
    pub fn new(width: u32, height: u32, name: &str) -> (Engine, EventLoop<()>) {
        puffin::set_scopes_on(true);

        let (window, event_loop) = Self::new_window(width, height, name);
        let window = Rc::new(window);

        let mut world = World::new();
        spawn_entities(&mut world);

        let mut renderer = Renderer::new();

        let scene = Scene::new();
        renderer.upload_scene_buffers(&scene, &world);

        renderer.setup(&window);

        let ui = UserInterface::new(window.clone());

        let engine = Engine {
            time: Time::new(),
            window,
            renderer,
            scene,
            camera: Camera::new(vec3(0.0, 100.0, 0.0), vec3(2.0, 100.0, 2.0)),
            input: Default::default(),
            ui,
            world,
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
        self.renderer.recreate(&self.window);
    }

    pub fn run(&mut self) {
        puffin::GlobalProfiler::lock().new_frame();
        puffin::profile_function!();
        self.camera.update_camera(self.time.delta_time());
        self.renderer
            .update(&self.camera, &mut self.ui, &mut self.world);
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
