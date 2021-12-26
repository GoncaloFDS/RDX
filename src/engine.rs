use crate::camera::Camera;
use crate::chunk::{
    Biome, Chunk, ChunkCoord, NoiseSettings, TerrainGenerator, CHUNK_DRAW_RANGE, CHUNK_SIZE,
    MAP_SIZE,
};
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
    let noise_settings = NoiseSettings::new(144, 1, 0.008);
    let biome = Biome::new(noise_settings);
    let chunks_to_spawn: Vec<_> = (0..MAP_SIZE)
        .collect::<Vec<_>>()
        .par_iter()
        .flat_map(|&x| {
            (0..MAP_SIZE)
                .into_iter()
                .map(|z| {
                    let x = x - MAP_SIZE / 2;
                    let z = z - MAP_SIZE / 2;
                    let mut chunk = Chunk::new(ivec3(x * CHUNK_SIZE, 0, z * CHUNK_SIZE));
                    let chunk_coord = ChunkCoord::new(x, z);
                    // log::debug!("inserting {:?}", chunk_coord);
                    TerrainGenerator::generate_chunk(&mut chunk, &biome);
                    (chunk, chunk_coord)
                })
                .collect::<Vec<_>>()
        })
        .collect();

    world.spawn_batch(chunks_to_spawn);

    // world.spawn((TerrainGenerator::new(Vec3::ZERO),));
}

fn get_chunks_around_camera(world: &mut World, camera: &Camera) -> bool {
    // TODO: remove
    let noise_settings = NoiseSettings::new(144, 1, 0.008);
    let biome = Biome::new(noise_settings);
    //

    let camera_chunk = Chunk::chunk_coords_from_world_position(camera.position());
    let start_x = camera_chunk.x() - CHUNK_DRAW_RANGE;
    let start_z = camera_chunk.z() - CHUNK_DRAW_RANGE;
    let end_x = camera_chunk.x() + CHUNK_DRAW_RANGE;
    let end_z = camera_chunk.z() + CHUNK_DRAW_RANGE;

    let new_chunks_coords = (start_x..end_x)
        .into_iter()
        .flat_map(|x| {
            (start_z..end_z)
                .into_iter()
                .map(|z| ChunkCoord::new(x, z))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let existing = world
        .query::<&ChunkCoord>()
        .iter()
        .map(|(_, &chunk)| chunk)
        .collect::<Vec<_>>();

    let chunks_to_spawn = new_chunks_coords
        .iter()
        .filter(|chunk_coord| !existing.contains(chunk_coord))
        .map(|&chunk_coord| {
            // log::debug!("inserting {:?}", chunk_coord);
            let mut chunk = Chunk::new(ivec3(
                chunk_coord.x() * CHUNK_SIZE,
                0,
                chunk_coord.z() * CHUNK_SIZE,
            ));
            TerrainGenerator::generate_chunk(&mut chunk, &biome);
            (chunk, chunk_coord)
        })
        .collect::<Vec<_>>();

    let should_update = !chunks_to_spawn.is_empty();
    world.spawn_batch(chunks_to_spawn);

    should_update
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
        renderer.upload_textures(&scene);
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
        if get_chunks_around_camera(&mut self.world, &self.camera) {
            self.renderer.upload_scene_buffers(&self.scene, &self.world);
        }
        self.renderer
            .update(&self.camera, &mut self.ui, &mut self.world, &self.scene);
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
