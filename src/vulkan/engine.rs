use crate::vulkan::device::Device;
use crate::vulkan::renderer::Renderer;
use crate::vulkan::scene::Scene;
use erupt::vk;
use std::rc::Rc;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct Engine {
    window: Window,
    device: Rc<Device>,
    renderer: Renderer,
    scene: Scene,
}

impl Engine {
    pub fn new(width: u32, height: u32, name: &str) -> (Engine, EventLoop<()>) {
        let (window, event_loop) = Self::new_window(width, height, name);

        let device = Rc::new(Device::new(
            &[
                vk::KHR_SWAPCHAIN_EXTENSION_NAME,
                vk::KHR_ACCELERATION_STRUCTURE_EXTENSION_NAME,
                vk::KHR_RAY_TRACING_PIPELINE_EXTENSION_NAME,
                vk::KHR_BUFFER_DEVICE_ADDRESS_EXTENSION_NAME,
                vk::KHR_DEFERRED_HOST_OPERATIONS_EXTENSION_NAME,
            ],
            vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE,
        ));

        let mut renderer = Renderer::new(device.clone());

        renderer.setup(&window);

        let scene = Scene::new();
        renderer.upload_meshes(&scene);

        let engine = Engine {
            window,
            device,
            renderer,
            scene,
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
        self.renderer.recreate_swapchain(&self.window, &self.scene);
    }

    pub fn run(&mut self) {
        self.renderer.draw_frame();
    }

    pub fn shutdown(&self) {
        self.renderer.shutdown();
    }
}
