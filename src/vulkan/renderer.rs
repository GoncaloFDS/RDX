use crate::vulkan::command_buffers::CommandBuffers;
use crate::vulkan::debug_utils::DebugMessenger;
use crate::vulkan::depth_buffer::DepthBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::framebuffer::Framebuffer;
use crate::vulkan::graphics_pipeline::GraphicsPipeline;
use crate::vulkan::scene::Scene;
use crate::vulkan::semaphore::Semaphore;
use crate::vulkan::swapchain::Swapchain;
use crate::vulkan::uniform_buffer::UniformBuffer;
use erupt::vk;
use std::rc::Rc;
use winit::window::Window;

pub struct Renderer {
    device: Rc<Device>,
    debug_messenger: DebugMessenger,
    command_buffers: CommandBuffers,
    swapchain: Option<Swapchain>,
    depth_buffer: Option<DepthBuffer>,
    graphics_pipeline: Option<GraphicsPipeline>,
    framebuffers: Vec<Framebuffer>,
    present_semaphores: Vec<Semaphore>,
    render_semaphores: Vec<Semaphore>,
    fences: Vec<Semaphore>,
    uniform_buffers: Vec<UniformBuffer>,
}

impl Renderer {
    pub fn new(device: Rc<Device>) -> Self {
        let debug_messenger = DebugMessenger::new(device.clone());

        let mut command_buffers =
            CommandBuffers::new(device.clone(), device.get_graphics_family_index(), true);

        Renderer {
            device,
            debug_messenger,
            command_buffers,
            swapchain: None,
            depth_buffer: None,
            graphics_pipeline: None,
            framebuffers: vec![],
            present_semaphores: vec![],
            render_semaphores: vec![],
            fences: vec![],
            uniform_buffers: vec![],
        }
    }

    pub fn create_swapchain(&mut self, window: &Window, scene: &Scene) {
        let extent = vk::Extent2D {
            width: window.inner_size().width,
            height: window.inner_size().height,
        };
        self.command_buffers.allocate(3);

        self.swapchain = Some(Swapchain::new(self.device.clone(), window));
        self.depth_buffer = Some(DepthBuffer::new(
            self.device.clone(),
            &self.command_buffers,
            extent,
        ));

        let swapchain = self.swapchain.as_ref().unwrap();
        for _ in swapchain.swapchain_images() {
            self.present_semaphores
                .push(Semaphore::new(self.device.clone()));
            self.render_semaphores
                .push(Semaphore::new(self.device.clone()));
            self.fences.push(Semaphore::new(self.device.clone()));
            self.uniform_buffers
                .push(UniformBuffer::new(self.device.clone()));
        }

        let depth_buffer = self.depth_buffer.as_ref().unwrap();
        self.graphics_pipeline = Some(GraphicsPipeline::new(
            self.device.clone(),
            &swapchain,
            depth_buffer,
            &self.uniform_buffers,
            scene,
            false,
        ));

        let graphics_pipeline = self.graphics_pipeline.as_ref().unwrap();
        for swapchain_image in swapchain.swapchain_images() {
            self.framebuffers.push(Framebuffer::new(
                self.device.clone(),
                &swapchain_image.view,
                graphics_pipeline.render_pass(),
                &swapchain,
                depth_buffer,
            ));
        }
    }

    pub fn delete_swapchain(&mut self) {}
}
