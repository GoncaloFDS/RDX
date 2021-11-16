use crate::debug::DebugMessenger;
use crate::device::{Device, Extent, ImageInfo};
use crate::swapchain::Swapchain;
use erupt::vk;
use std::sync::Arc;
use winit::window::Window;

pub struct Renderer {
    device: Arc<Device>,
    _debug_messenger: DebugMessenger,
    extent: Extent,
    swapchain: Swapchain,
    depth_format: vk::Format,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    fences: Vec<vk::Fence>,
    render_semaphore: vk::Semaphore,
    present_semaphore: vk::Semaphore,
}

impl Renderer {
    pub fn new(device: Arc<Device>, window: &Window) -> Self {
        let debug_messenger = DebugMessenger::new(device.get_instance());
        let swapchain = Swapchain::new(device.clone());
        let queue = device.get_device_queue();
        let depth_format = device.get_supported_depth_format();
        let extent = Extent::D2 {
            width: window.inner_size().width,
            height: window.inner_size().height,
        };
        Renderer {
            device,
            _debug_messenger: debug_messenger,
            extent,
            swapchain,
            depth_format,
            command_pool: Default::default(),
            command_buffers: vec![],
            fences: vec![],
            render_semaphore: Default::default(),
            present_semaphore: Default::default(),
        }
    }

    pub fn init(&mut self, window: &Window) {
        self.init_swapchain(window);
        self.create_swapchain(window.inner_size().width, window.inner_size().height, true);
        self.create_command_pool();
        self.create_command_buffers();
        self.create_synchronization_primitives();
        self.setup_depth_stencil();
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

    fn create_command_pool(&mut self) {
        self.command_pool = self.device.create_command_pool(
            self.swapchain.queue_node_index,
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        )
    }

    fn create_command_buffers(&mut self) {
        self.command_buffers = self.device.allocate_command_buffers(
            self.command_pool,
            vk::CommandBufferLevel::PRIMARY,
            self.swapchain.images.len() as u32,
        );
    }

    fn create_synchronization_primitives(&mut self) {
        self.fences = self
            .command_buffers
            .iter()
            .map(|_| self.device.create_fence())
            .collect();
        self.present_semaphore = self.device.create_semaphore();
        self.render_semaphore = self.device.create_semaphore();
    }

    fn setup_depth_stencil(&mut self) {
        let depth_image_info = ImageInfo {
            extent: self.extent,
            format: self.depth_format,
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlagBits::_1,
            usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            tiling: vk::ImageTiling::OPTIMAL,
        };
        let depth_image = self.device.create_image(depth_image_info);
    }
}

fn get_window_extent(window: &Window) -> vk::Extent2D {
    vk::Extent2D {
        width: window.inner_size().width,
        height: window.inner_size().height,
    }
}
