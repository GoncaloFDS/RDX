use crate::device::Device;
use crate::renderer::Renderer;
use crate::{HEIGHT, WIDTH, WINDOW_NAME};
use erupt::vk;
use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct RdxEngine {
    window: Window,
    device: Arc<Device>,
    renderer: Renderer,
}

impl RdxEngine {
    pub fn new() -> (RdxEngine, EventLoop<()>) {
        let (event_loop, window) = Self::new_window(WIDTH, HEIGHT, WINDOW_NAME);

        let device = Arc::new(Device::new(
            &[
                vk::KHR_SWAPCHAIN_EXTENSION_NAME,
                vk::KHR_ACCELERATION_STRUCTURE_EXTENSION_NAME,
                vk::KHR_RAY_TRACING_PIPELINE_EXTENSION_NAME,
                vk::KHR_BUFFER_DEVICE_ADDRESS_EXTENSION_NAME,
                vk::KHR_DEFERRED_HOST_OPERATIONS_EXTENSION_NAME,
            ],
            vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE,
        ));

        let renderer = Renderer::new(device.clone(), &window);

        let engine = RdxEngine {
            window,
            device,
            renderer,
        };

        (engine, event_loop)
    }

    fn new_window(width: u32, height: u32, name: &str) -> (EventLoop<()>, Window) {
        log::debug!("Creating new Window");
        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title(name)
            .with_inner_size(LogicalSize::new(width, height))
            .with_resizable(true)
            .build(&event_loop)
            .unwrap();

        (event_loop, window)
    }

    pub fn init_systems(&mut self) {
        self.renderer.init(&self.window);
    }

    pub fn resize(&mut self) {}

    pub fn run(&mut self) {}
}
