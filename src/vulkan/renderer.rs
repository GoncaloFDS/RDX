use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::command_pool::CommandPool;
use crate::vulkan::debug_utils::DebugMessenger;
use crate::vulkan::depth_buffer::DepthBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::fence::Fence;
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
    _debug_messenger: DebugMessenger,
    command_pool: CommandPool,
    swapchain: Option<Swapchain>,
    depth_buffer: Option<DepthBuffer>,
    graphics_pipeline: Option<GraphicsPipeline>,
    framebuffers: Vec<Framebuffer>,
    present_semaphores: Vec<Semaphore>,
    render_semaphores: Vec<Semaphore>,
    fences: Vec<Fence>,
    uniform_buffers: Vec<UniformBuffer>,
    current_frame: usize,
}

impl Renderer {
    pub fn new(device: Rc<Device>) -> Self {
        let debug_messenger = DebugMessenger::new(device.clone());

        let command_buffers =
            CommandPool::new(device.clone(), device.graphics_family_index(), true);

        Renderer {
            device,
            _debug_messenger: debug_messenger,
            command_pool: command_buffers,
            swapchain: None,
            depth_buffer: None,
            graphics_pipeline: None,
            framebuffers: vec![],
            present_semaphores: vec![],
            render_semaphores: vec![],
            fences: vec![],
            uniform_buffers: vec![],
            current_frame: 0,
        }
    }

    pub fn create_swapchain(&mut self, window: &Window, scene: &Scene) {
        let extent = vk::Extent2D {
            width: window.inner_size().width,
            height: window.inner_size().height,
        };
        self.command_pool.allocate(3);

        self.swapchain = Some(Swapchain::new(
            self.device.clone(),
            window,
            vk::PresentModeKHR::MAILBOX_KHR,
        ));

        self.depth_buffer = Some(DepthBuffer::new(
            self.device.clone(),
            &self.command_pool,
            extent,
        ));

        let swapchain = self.swapchain.as_ref().unwrap();
        for _ in swapchain.images() {
            self.present_semaphores
                .push(Semaphore::new(self.device.clone()));
            self.render_semaphores
                .push(Semaphore::new(self.device.clone()));
            self.fences.push(Fence::new(self.device.clone(), true));
            self.uniform_buffers
                .push(UniformBuffer::new(self.device.clone()));
        }

        let depth_buffer = self.depth_buffer.as_ref().unwrap();
        self.graphics_pipeline = Some(GraphicsPipeline::new(
            self.device.clone(),
            swapchain,
            depth_buffer,
            &self.uniform_buffers,
            scene,
            false,
        ));

        let graphics_pipeline = self.graphics_pipeline.as_ref().unwrap();

        for swapchain_image_view in swapchain.image_views() {
            self.framebuffers.push(Framebuffer::new(
                self.device.clone(),
                swapchain_image_view,
                graphics_pipeline.render_pass(),
                swapchain,
                depth_buffer,
            ));
        }

        self.command_pool.allocate(swapchain.images().len() as _);
    }

    pub fn delete_swapchain(&mut self) {}
    pub fn recreate_swapchain(&mut self) {}

    pub fn draw_frame(&mut self) {
        let fence = &self.fences[self.current_frame];
        let render_semaphore = &self.render_semaphores[self.current_frame];
        let present_semaphore = &self.present_semaphores[self.current_frame];

        let timeout = u64::MAX;
        fence.wait(timeout);

        let swapchain = self.swapchain.as_ref().unwrap();
        if let Some(image_index) =
            swapchain.acquire_next_image(timeout, Some(render_semaphore.handle()))
        {
            let command_buffer = self.command_pool.begin(image_index as _);
            self.render(command_buffer, image_index);
            self.command_pool.end(image_index as _);

            fence.reset();

            self.device.submit(
                &[command_buffer.handle()],
                &[render_semaphore.handle()],
                &[present_semaphore.handle()],
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                Some(fence.handle()),
            );

            self.device.present(
                &[present_semaphore.handle()],
                &[swapchain.handle()],
                &[image_index],
            );

            self.current_frame = (self.current_frame + 1) % self.fences.len();
        } else {
            self.recreate_swapchain();
        }
    }

    fn render(&self, command_buffer: CommandBuffer, image_index: u32) {
        let graphics_pipeline = self.graphics_pipeline.as_ref().unwrap();
        let swapchain = self.swapchain.as_ref().unwrap();

        command_buffer.begin_render_pass(
            &self.device,
            graphics_pipeline.render_pass().handle(),
            self.framebuffers[image_index as usize].handle(),
            swapchain.extent(),
        );

        command_buffer.bind_pipeline(
            &self.device,
            vk::PipelineBindPoint::GRAPHICS,
            graphics_pipeline.handle(),
        );
        // bind descriptors sets
        // bind vertex and index buffers
        // draw
        command_buffer.end_render_pass(&self.device);
    }

    pub fn shutdown(&self) {
        self.device.wait_idle();
    }
}
