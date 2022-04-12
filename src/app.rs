use crate::camera::Camera;
use crate::renderers::egui_renderer::EguiRenderer;
use crate::renderers::raytracer::Raytracer;
use crate::renderers::Renderer;
use crate::user_interface::UserInterface;
use crate::vulkan::device::Device;
use crate::vulkan::instance::Instance;
use crate::vulkan::raytracing::raytracing_properties::RaytracingProperties;
use erupt::vk;
use glam::vec3;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub struct App {
    window: Window,
    device: Device,
    instance: Instance,
    render_queue: Vec<Box<dyn Renderer>>,
    ui: UserInterface,
    camera: Camera,
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

        let egui_renderer = EguiRenderer::new(&mut device, &ui);
        let raytracer = Raytracer::new(&mut device, raytracing_properties);
        let render_queue: Vec<Box<dyn Renderer>> =
            vec![Box::new(raytracer), Box::new(egui_renderer)];

        let camera = Camera::new(vec3(0.0, 0.0, -1.0), vec3(0.0, 0.0, 0.0));

        let app = App {
            window,
            device,
            instance,
            render_queue,
            ui,
            camera,
        };

        (app, event_loop)
    }

    pub fn on_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => {
                let ui_captured_input = self.ui.on_event(&event);
                if ui_captured_input {
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => {
                        log::info!("Closing Window");
                        *control_flow = ControlFlow::Exit
                    }
                    WindowEvent::Resized(size) => self.resize(vk::Extent2D {
                        width: size.width,
                        height: size.height,
                    }),
                    WindowEvent::KeyboardInput { input, .. } => {
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
                        }
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

    pub fn resize(&mut self, size: vk::Extent2D) {
        self.device.resize_swapchain(size);
    }

    pub fn run(&mut self) {
        puffin::GlobalProfiler::lock().new_frame();
        puffin::profile_function!();
        let acquired_frame = self
            .device
            .acquire_swapchain_frame(&self.instance, u64::MAX);

        if acquired_frame.invalidate_images {
            self.recreate_swapchain();
        }

        self.camera.update_camera(0.1);
        self.ui.update(&self.window);
        self.render_queue.iter_mut().for_each(|renderer| {
            renderer.update(
                &mut self.device,
                acquired_frame.image_index,
                &self.camera,
                &mut self.ui,
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
    }

    fn recreate_swapchain(&mut self) {
        self.device.recreate_swapchain();
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
