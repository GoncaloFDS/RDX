use crate::vulkan::buffer::Buffer;
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
use crate::vulkan::uniform_buffer::{UniformBuffer, UniformBufferObject};
use erupt::vk;
use glam::{vec3, Mat4, Vec3};
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
    vertex_buffers: Vec<Buffer>,
    index_buffers: Vec<Buffer>,
    uniform_buffers: Vec<UniformBuffer>,
    current_frame: usize,
}

impl Renderer {
    pub fn new(device: Rc<Device>) -> Self {
        let debug_messenger = DebugMessenger::new(device.clone());

        let command_pool = CommandPool::new(device.clone(), device.graphics_family_index(), true);

        Renderer {
            device,
            _debug_messenger: debug_messenger,
            command_pool,
            swapchain: None,
            depth_buffer: None,
            graphics_pipeline: None,
            framebuffers: vec![],
            present_semaphores: vec![],
            render_semaphores: vec![],
            fences: vec![],
            vertex_buffers: vec![],
            index_buffers: vec![],
            uniform_buffers: vec![],
            current_frame: 0,
        }
    }

    pub fn setup(&mut self, window: &Window) {
        self.create_swapchain(window);
        self.create_uniform_buffers();
        self.create_depth_buffer(window);
        self.create_pipelines();
        self.create_framebuffers();
        self.create_command_buffers();
        self.create_sync_structures();
    }

    fn delete_swapchain(&mut self) {
        self.command_pool.reset();
        self.swapchain = None;
        self.graphics_pipeline = None;
        self.depth_buffer = None;
        self.uniform_buffers.clear();
        self.framebuffers.clear();
        self.fences.clear();
        self.present_semaphores.clear();
        self.render_semaphores.clear();
    }

    pub fn recreate_swapchain(&mut self, window: &Window, scene: &Scene) {
        self.device.wait_idle();
        self.delete_swapchain();
        self.setup(window);
    }

    pub fn draw_frame(&mut self) {
        let extent = self.swapchain.as_ref().unwrap().extent();
        let aspect_ratio = extent.width as f32 / extent.height as f32;
        let ubo = UniformBufferObject {
            view_model: Mat4::look_at_rh(vec3(0.0, 0.0, 1.0), Vec3::ZERO, Vec3::Y).into(),
            projection: Mat4::perspective_rh(60.0f32.to_radians(), aspect_ratio, 0.01, 10000.0)
                .into(),
        };
        self.uniform_buffers[self.current_frame].update_gpu_buffer(&ubo);

        let fence = &self.fences[self.current_frame];
        let render_semaphore = &self.render_semaphores[self.current_frame];
        let present_semaphore = &self.present_semaphores[self.current_frame];

        let timeout = u64::MAX;
        fence.wait(timeout);

        let swapchain = self.swapchain.as_ref().unwrap();

        if let Some(current_frame) =
            swapchain.acquire_next_image(timeout, Some(render_semaphore.handle()))
        {
            self.current_frame = current_frame;
        } else {
            log::debug!("failed to acquire next image");
            return;
        }

        let command_buffer = self.command_pool.begin(self.current_frame as _);
        self.render(command_buffer, self.current_frame);
        self.command_pool.end(self.current_frame as _);

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
            &[self.current_frame as u32],
        );

        self.current_frame = (self.current_frame + 1) % self.fences.len();
    }

    fn render(&self, command_buffer: CommandBuffer, image_index: usize) {
        let graphics_pipeline = self.graphics_pipeline.as_ref().unwrap();
        let swapchain = self.swapchain.as_ref().unwrap();

        command_buffer.begin_render_pass(
            &self.device,
            graphics_pipeline.render_pass().handle(),
            self.framebuffers[image_index].handle(),
            swapchain.extent(),
        );

        command_buffer.bind_pipeline(
            &self.device,
            vk::PipelineBindPoint::GRAPHICS,
            graphics_pipeline.handle(),
        );
        // bind descriptors sets
        command_buffer.bind_descriptor_sets(
            &self.device,
            vk::PipelineBindPoint::GRAPHICS,
            graphics_pipeline.pipeline_layout().handle(),
            &[graphics_pipeline
                .descriptor_set_manager()
                .descriptor_set(image_index)],
        );
        // bind vertex and index buffers
        command_buffer.bind_vertex_buffer(&self.device, &self.vertex_buffers);
        command_buffer.bind_index_buffer(&self.device, &self.index_buffers[0]);
        // draw
        command_buffer.draw_indexed(&self.device);
        command_buffer.end_render_pass(&self.device);
    }

    pub fn shutdown(&self) {
        self.device.wait_idle();
    }

    pub fn upload_meshes(&mut self, scene: &Scene) {
        let mut vertices = vec![];
        let mut indices = vec![];
        for model in scene.models() {
            vertices.extend_from_slice(model.vertices());
            indices.extend_from_slice(model.indices());
        }

        let vertex_buffer = Buffer::with_data(
            self.device.clone(),
            &vertices,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        );

        let index_buffer = Buffer::with_data(
            self.device.clone(),
            &indices,
            vk::BufferUsageFlags::INDEX_BUFFER,
        );

        self.vertex_buffers.push(vertex_buffer);
        self.index_buffers.push(index_buffer);
    }

    fn create_swapchain(&mut self, window: &Window) {
        self.swapchain = Some(Swapchain::new(
            self.device.clone(),
            window,
            vk::PresentModeKHR::MAILBOX_KHR,
        ));
    }

    fn create_depth_buffer(&mut self, window: &Window) {
        let extent = vk::Extent2D {
            width: window.inner_size().width,
            height: window.inner_size().height,
        };
        self.depth_buffer = Some(DepthBuffer::new(
            self.device.clone(),
            &self.command_pool,
            extent,
        ));
    }

    fn create_framebuffers(&mut self) {
        let graphics_pipeline = self.graphics_pipeline.as_ref().unwrap();
        let swapchain = self.swapchain.as_ref().unwrap();
        let depth_buffer = self.depth_buffer.as_ref().unwrap();
        for swapchain_image_view in swapchain.image_views() {
            self.framebuffers.push(Framebuffer::new(
                self.device.clone(),
                swapchain_image_view,
                graphics_pipeline.render_pass(),
                swapchain,
                depth_buffer,
            ));
        }
    }

    fn create_pipelines(&mut self) {
        let swapchain = self.swapchain.as_ref().unwrap();
        let depth_buffer = self.depth_buffer.as_ref().unwrap();
        self.graphics_pipeline = Some(GraphicsPipeline::new(
            self.device.clone(),
            swapchain,
            depth_buffer,
            &self.uniform_buffers,
            false,
        ));
    }

    fn create_command_buffers(&mut self) {
        let count = self.swapchain.as_ref().unwrap().images().len();
        self.command_pool.allocate(count as _);
    }

    fn create_sync_structures(&mut self) {
        let count = self.swapchain.as_ref().unwrap().images().len();

        self.present_semaphores
            .resize_with(count, || Semaphore::new(self.device.clone()));
        self.render_semaphores
            .resize_with(count, || Semaphore::new(self.device.clone()));
        self.fences
            .resize_with(count, || Fence::new(self.device.clone(), true));
    }

    fn create_uniform_buffers(&mut self) {
        let count = self.swapchain.as_ref().unwrap().images().len();
        self.uniform_buffers
            .resize_with(count, || UniformBuffer::new(self.device.clone()));
    }
}
