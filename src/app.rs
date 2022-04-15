use crate::camera::Camera;
use crate::chunk::{
    Biome, Chunk, ChunkCoord, NoiseSettings, TerrainGenerator, CHUNK_SIZE, MAP_SIZE,
};
use crate::input::Input;
use crate::renderers::egui_renderer::EguiRenderer;
use crate::renderers::raytracer::Raytracer;
use crate::renderers::Renderer;
use crate::scene::Scene;
use crate::time::Time;
use crate::user_interface::UserInterface;
use crate::vulkan::device::Device;
use crate::vulkan::instance::Instance;
use crate::vulkan::raytracing::raytracing_properties::RaytracingProperties;
use bevy_ecs::prelude::*;
use erupt::vk;
use glam::{ivec3, vec3};
use rayon::prelude::*;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub struct App {
    window: Window,
    device: Device,
    instance: Instance,
    world: World,
    scene: Scene,
    render_queue: Vec<Box<dyn Renderer>>,
    ui: UserInterface,
    camera: Camera,
    input: Input,
    time: Time,
}

impl App {
    pub fn new() -> (Self, EventLoop<()>) {
        log::info!("Starting RDX");
        puffin::set_scopes_on(true);
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("RDX")
            .with_inner_size(PhysicalSize::new(1024, 720))
            .build(&event_loop)
            .unwrap();

        let instance = Instance::new(&window);
        let mut device = Device::new(&instance, &window);
        let raytracing_properties = RaytracingProperties::new(&device, &instance);

        let ui = UserInterface::new(&window);

        let egui_renderer = EguiRenderer::new(&mut device);
        let raytracer = Raytracer::new(&mut device, raytracing_properties);
        let render_queue: Vec<Box<dyn Renderer>> =
            vec![Box::new(raytracer), Box::new(egui_renderer)];

        let mut world = World::default();
        let scene = Scene::new();

        spawn_entities(&mut world);

        let app = App {
            window,
            device,
            instance,
            world,
            scene,
            render_queue,
            ui,
            camera: Camera::new(vec3(20.0, 50.0, 0.0), vec3(0.0, 50.0, 0.0)),
            input: Input::default(),
            time: Time::new(),
        };

        (app, event_loop)
    }

    pub fn on_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => {
                self.run();
            }
            Event::WindowEvent {
                event: window_event,
                ..
            } => {
                let ui_captured_input = self.ui.on_event(&window_event);
                if ui_captured_input {
                    return;
                }

                match window_event {
                    WindowEvent::CloseRequested => self.close_window(control_flow),
                    WindowEvent::Resized(size) => self.resize(size),
                    WindowEvent::KeyboardInput { input, .. } => {
                        self.keyboard_input(control_flow, input)
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        self.mouse_input(button, state);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        self.cursor_moved(position);
                    }
                    _ => {}
                }
            }
            _ => (),
        }
    }

    fn close_window(&mut self, control_flow: &mut ControlFlow) {
        log::info!("Closing Window");
        *control_flow = ControlFlow::Exit
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.device.resize_swapchain(vk::Extent2D {
            width: size.width,
            height: size.height,
        });
    }

    fn run(&mut self) {
        puffin::GlobalProfiler::lock().new_frame();
        puffin::profile_function!();
        let acquired_frame = self
            .device
            .acquire_swapchain_frame(&self.instance, u64::MAX);

        if acquired_frame.invalidate_images {
            self.recreate_swapchain();
        }

        self.camera.update_camera(self.time.delta_time());
        self.ui.update(&self.window);
        self.render_queue.iter_mut().for_each(|renderer| {
            renderer.update(
                &mut self.device,
                acquired_frame.image_index,
                &self.camera,
                &mut self.ui,
                &mut self.world,
                &self.scene,
            )
        });

        let command_buffer = self.device.command_buffer(acquired_frame.frame_index);
        let semaphore = self.device.semaphore(acquired_frame.frame_index).handle();

        command_buffer.begin(&self.device, vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        self.render_queue.iter().for_each(|renderer| {
            renderer.fill_command_buffer(&self.device, command_buffer, acquired_frame.image_index)
        });

        command_buffer.end(&self.device);

        self.device.submit(
            &[vk::SemaphoreSubmitInfoBuilder::new()
                .semaphore(acquired_frame.ready)
                .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)],
            &[vk::SemaphoreSubmitInfoBuilder::new()
                .semaphore(semaphore)
                .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)],
            &[vk::CommandBufferSubmitInfoBuilder::new().command_buffer(command_buffer.handle())],
            acquired_frame.complete,
        );

        self.device
            .queue_present(self.device.queue(), semaphore, acquired_frame.image_index);

        self.time.tick();
    }

    fn recreate_swapchain(&mut self) {
        self.device.recreate_swapchain();
    }

    fn keyboard_input(&mut self, control_flow: &mut ControlFlow, input: KeyboardInput) {
        if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
            *control_flow = ControlFlow::Exit;
        } else if input.virtual_keycode == Some(VirtualKeyCode::F1)
            && input.state == ElementState::Pressed
        {
            self.ui.toggle_settings()
        } else if input.virtual_keycode == Some(VirtualKeyCode::F2)
            && input.state == ElementState::Pressed
        {
            self.ui.toggle_profiler()
        } else {
            self.camera.handle_input(input);
        }
    }

    fn mouse_input(&mut self, input: MouseButton, state: ElementState) {
        self.camera.handle_mouse_input(input, state);
    }

    fn cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.input.update(position);
        self.camera.handle_mouse_move(
            self.input.delta_x(),
            self.input.delta_y(),
            self.time.delta_time(),
        )
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.render_queue
            .iter_mut()
            .for_each(|renderer| renderer.destroy(&mut self.device));
        self.device.wait_idle();
        self.device.destroy();
        self.instance.destroy();
    }
}

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
                    TerrainGenerator::generate_chunk(&mut chunk, &biome);
                    (chunk, chunk_coord)
                })
                .collect::<Vec<_>>()
        })
        .collect();

    world.spawn_batch(chunks_to_spawn);
}
