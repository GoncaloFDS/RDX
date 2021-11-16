use crate::debug::DebugMessenger;
use crate::device::Device;
use crate::swapchain::Swapchain;
use erupt::utils::surface;
use erupt::vk;
use std::rc::Rc;
use winit::window::Window;

pub struct Renderer {
    device: Rc<Device>,
    _debug_messenger: DebugMessenger,
    swapchain: Swapchain,
}

impl Renderer {
    pub fn new(device: Rc<Device>, window: &Window) -> Self {
        let debug_messenger = DebugMessenger::new(device.get_instance());
        let swapchain = Swapchain::new(device.clone());
        let queue = device.get_device_queue();
        let depth_format = device.get_supported_depth_format();
        Renderer {
            device,
            _debug_messenger: debug_messenger,
            swapchain,
        }
    }

    pub fn init(&mut self, window: &Window) {
        self.init_swapchain(window);
        self.create_swapchain(window.inner_size().width, window.inner_size().height, true);
        // self.create_command_pool();
        // self.create_command_buffers();
        // self.create_synchronization_primitives();
        // self.setup_depth_stencil();
        // self.setup_render_pass();
        // self.create_pipeline_cache();
        // self.setup_framebuffer();
        //
        // self.prepare_vertices();
        // self.prepare_uniform_buffers();
        // self.setup_descriptor_set_layout();
        // self.prepare_pipelines();
        // self.setup_descriptor_pool();
        // self.setup_descriptor_set();
        // self.build_command_buffers();
    }

    pub fn update(&mut self) {
        // self.update_uniform_buffers();
    }

    fn init_swapchain(&mut self, window: &Window) {
        self.swapchain.init_with_window(window);
    }

    fn create_swapchain(&mut self, width: u32, height: u32, vsync: bool) {
        self.swapchain.create(width, height, vsync);
    }
}

fn get_window_extent(window: &Window) -> vk::Extent2D {
    vk::Extent2D {
        width: window.inner_size().width,
        height: window.inner_size().height,
    }
}
