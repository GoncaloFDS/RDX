use crate::camera::Camera;
use crate::user_interface::UserInterface;
use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::command_pool::CommandPool;
use crate::vulkan::debug_utils::DebugMessenger;
use crate::vulkan::depth_buffer::DepthBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::fence::Fence;
use crate::vulkan::framebuffer::Framebuffer;
use crate::vulkan::graphics_pipeline::GraphicsPipeline;
use crate::vulkan::render_pass::RenderPass;
use crate::vulkan::scene::{Scene, Texture, TextureImage};
use crate::vulkan::semaphore::Semaphore;
use crate::vulkan::swapchain::Swapchain;
use crate::vulkan::uniform_buffer::{UniformBuffer, UniformBufferObject};
use crate::vulkan::vertex::EguiVertex;
use erupt::vk;
use glam::{vec2, vec4, Vec2};
use std::collections::HashMap;
use std::mem::size_of;
use std::rc::Rc;
use winit::window::Window;

const VERTICES_PER_QUAD: u64 = 4;
const VERTEX_BUFFER_SIZE: u64 = 1024 * 1024 * VERTICES_PER_QUAD;
const INDEX_BUFFER_SIZE: u64 = 1024 * 1024 * 2;

const FONT_IMAGE_ID: u32 = 10;

pub struct PushConstants {
    pub screen_size: Vec2,
}

pub struct Renderer {
    device: Rc<Device>,
    _debug_messenger: DebugMessenger,
    command_pool: CommandPool,
    swapchain: Swapchain,
    depth_buffer: DepthBuffer,
    render_pass: RenderPass,
    graphics_pipeline: GraphicsPipeline,
    ui_pipeline: GraphicsPipeline,
    framebuffers: Vec<Framebuffer>,
    present_semaphores: Vec<Semaphore>,
    render_semaphores: Vec<Semaphore>,
    fences: Vec<Fence>,
    vertex_buffers: Vec<Buffer>,
    index_buffers: Vec<Buffer>,
    uniform_buffers: Vec<UniformBuffer>,
    current_frame: usize,
    textures: HashMap<u32, Texture>,
    texture_images: HashMap<u32, TextureImage>,
    egui_texture_version: u64,
}

impl Renderer {
    pub fn new() -> Self {
        let device = Rc::new(Device::new());

        let debug_messenger = DebugMessenger::new(device.clone());

        let command_pool = CommandPool::new(device.clone(), device.graphics_family_index(), true);

        let swapchain = Swapchain::uninitialized(device.clone());
        let depth_buffer = DepthBuffer::uninitialized(device.clone());
        let render_pass = RenderPass::uninitialized(device.clone());
        let graphics_pipeline = GraphicsPipeline::uninitialized(device.clone());
        let ui_pipeline = GraphicsPipeline::uninitialized(device.clone());

        Renderer {
            device,
            _debug_messenger: debug_messenger,
            command_pool,
            swapchain,
            depth_buffer,
            render_pass,
            graphics_pipeline,
            ui_pipeline,
            framebuffers: vec![],
            present_semaphores: vec![],
            render_semaphores: vec![],
            fences: vec![],
            vertex_buffers: vec![],
            index_buffers: vec![],
            uniform_buffers: vec![],
            current_frame: 0,
            textures: Default::default(),
            texture_images: Default::default(),
            egui_texture_version: 0,
        }
    }

    pub fn setup(&mut self, window: &Window) {
        self.create_swapchain(window);
        self.create_depth_buffer(window);
        self.create_default_render_pass();
        self.create_uniform_buffers();
        self.create_pipelines();
        self.create_framebuffers();
        self.create_command_buffers();
        self.create_sync_structures();
        self.allocate_ui_buffers();
    }

    fn delete_swapchain(&mut self) {
        self.command_pool.reset();
        self.swapchain.cleanup();
        self.ui_pipeline.cleanup();
        self.graphics_pipeline.cleanup();
        self.render_pass.cleanup();
        self.uniform_buffers.clear();
        self.framebuffers.clear();
        self.present_semaphores.clear();
        self.render_semaphores.clear();
        self.fences.clear();
        self.egui_texture_version = 0;
        self.current_frame = 0;
    }

    pub fn recreate_swapchain(&mut self, window: &Window) {
        self.device.wait_idle();
        self.delete_swapchain();
        self.setup(window);
    }

    pub fn update(&mut self, camera: &Camera) {
        let extent = self.swapchain.extent();
        let aspect_ratio = extent.width as f32 / extent.height as f32;
        let ubo = UniformBufferObject {
            view_model: camera.view().into(),
            projection: camera.projection(aspect_ratio).into(),
        };
        self.uniform_buffers[self.current_frame].update_gpu_buffer(&ubo);
    }

    pub fn draw_frame(&mut self, ui: &mut UserInterface) {
        let fence = &self.fences[self.current_frame];
        let render_semaphore = &self.render_semaphores[self.current_frame];
        let present_semaphore = &self.present_semaphores[self.current_frame];

        let timeout = u64::MAX;
        fence.wait(timeout);

        if let Some(current_frame) = self
            .swapchain
            .acquire_next_image(timeout, Some(render_semaphore.handle()))
        {
            self.current_frame = current_frame;
        } else {
            log::debug!("failed to acquire next image");
            return;
        }

        let command_buffer = self.command_pool.begin(self.current_frame as _);
        self.render(command_buffer, self.current_frame);

        {
            ui.render();
            {
                let texture = ui.egui().texture();
                if texture.version != self.egui_texture_version {
                    self.egui_texture_version = texture.version;

                    let data = texture
                        .pixels
                        .iter()
                        .flat_map(|&r| vec![r, r, r, r])
                        .collect::<Vec<_>>();

                    let font_texture = Texture::new(texture.width as _, texture.height as _, data);
                    let font_image =
                        TextureImage::new(self.device.clone(), &self.command_pool, &font_texture);

                    self.textures.insert(FONT_IMAGE_ID, font_texture);
                    self.texture_images.insert(FONT_IMAGE_ID, font_image);
                    //
                    {
                        let descriptor_set_manager = self.ui_pipeline.descriptor_set_manager();
                        if let Some(font_image) = self.texture_images.get(&FONT_IMAGE_ID) {
                            self.swapchain
                                .images()
                                .iter()
                                .enumerate()
                                .for_each(|(i, _)| {
                                    let image_info = [vk::DescriptorImageInfoBuilder::new()
                                        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                                        .image_view(font_image.image_view().handle())
                                        .sampler(font_image.sampler().handle())];

                                    let descriptor_writes =
                                        [descriptor_set_manager.bind_image(i as _, 0, &image_info)];

                                    descriptor_set_manager.update_descriptors(&descriptor_writes);
                                });
                        }
                    }
                }
            }

            let push_constants = [[
                self.swapchain.extent().width as f32,
                self.swapchain.extent().height as f32,
            ]];

            {
                command_buffer.bind_pipeline(
                    &self.device,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.ui_pipeline.handle(),
                );
                // bind descriptors sets
                command_buffer.bind_descriptor_sets(
                    &self.device,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.ui_pipeline.pipeline_layout().handle(),
                    &[self
                        .ui_pipeline
                        .descriptor_set_manager()
                        .descriptor_set(self.current_frame)],
                );
                command_buffer.push_constants(
                    &self.device,
                    self.ui_pipeline.pipeline_layout().handle(),
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    &push_constants,
                );
            }

            let mut vertex_start = 0;
            let mut index_start = 0;
            for egui::ClippedMesh(rect, mesh) in ui.clipped_meshes() {
                if mesh.vertices.is_empty() || mesh.indices.is_empty() {
                    continue;
                }
                // user image...
                // ...
                //

                let vertices_count = mesh.vertices.len() as u64;
                let indices_count = mesh.indices.len() as u64;

                {
                    let vertices = mesh
                        .vertices
                        .iter()
                        .map(|vertex| {
                            EguiVertex::new(
                                vec2(vertex.pos.x, vertex.pos.y),
                                vec2(vertex.uv.x, vertex.uv.y),
                                vec4(
                                    vertex.color.r() as f32 / 255.0,
                                    vertex.color.g() as f32 / 255.0,
                                    vertex.color.b() as f32 / 255.0,
                                    vertex.color.a() as f32 / 255.0,
                                ),
                            )
                        })
                        .collect::<Vec<_>>();
                    self.vertex_buffers[0].write_data(&vertices, vertex_start);
                    let indices = &mesh.indices;
                    self.index_buffers[0].write_data(&indices, index_start);
                    // bind vertex and index buffers
                    let buffers = [&self.vertex_buffers[0]];
                    command_buffer.bind_vertex_buffer(
                        &self.device,
                        &buffers,
                        &[vertex_start * size_of::<EguiVertex>() as u64],
                    );
                    command_buffer.bind_index_buffer(
                        &self.device,
                        &self.index_buffers[0],
                        index_start * size_of::<u32>() as u64,
                    );
                    // draw
                    command_buffer.draw_indexed(&self.device, indices.len() as _);
                }
                vertex_start += vertices_count;
                index_start += indices_count;
            }
        }

        command_buffer.end_render_pass(&self.device);
        command_buffer.end(&self.device);
        fence.reset();

        self.device.submit(
            &[command_buffer.handle()],
            &[render_semaphore.handle()],
            &[present_semaphore.handle()],
            &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            Some(fence.handle()),
        );
    }

    pub fn present_frame(&mut self) {
        let present_semaphore = &self.present_semaphores[self.current_frame];
        let swapchain = &self.swapchain;
        self.device.present(
            &[present_semaphore.handle()],
            &[swapchain.handle()],
            &[self.current_frame as u32],
        );

        self.current_frame = (self.current_frame + 1) % self.fences.len();
    }

    fn render(&self, command_buffer: CommandBuffer, image_index: usize) {
        command_buffer.begin_render_pass(
            &self.device,
            self.render_pass.handle(),
            self.framebuffers[image_index].handle(),
            self.swapchain.extent(),
        );

        command_buffer.bind_pipeline(
            &self.device,
            vk::PipelineBindPoint::GRAPHICS,
            self.graphics_pipeline.handle(),
        );
        // bind descriptors sets
        command_buffer.bind_descriptor_sets(
            &self.device,
            vk::PipelineBindPoint::GRAPHICS,
            self.graphics_pipeline.pipeline_layout().handle(),
            &[self
                .graphics_pipeline
                .descriptor_set_manager()
                .descriptor_set(image_index)],
        );
        // bind vertex and index buffers
        let buffers = [&self.vertex_buffers[1]];
        command_buffer.bind_vertex_buffer(&self.device, &buffers, &[0]);
        command_buffer.bind_index_buffer(&self.device, &self.index_buffers[1], 0);
        // draw
        command_buffer.draw_indexed(&self.device, 3);
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

    fn create_default_render_pass(&mut self) {
        self.render_pass = RenderPass::new(
            self.device.clone(),
            &self.swapchain,
            &self.depth_buffer,
            vk::AttachmentLoadOp::CLEAR,
            vk::AttachmentLoadOp::CLEAR,
        );
    }

    fn allocate_ui_buffers(&mut self) {
        let mut vertex_buffer = Buffer::new(
            self.device.clone(),
            VERTEX_BUFFER_SIZE * size_of::<EguiVertex>() as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        );
        vertex_buffer.allocate_memory(gpu_alloc::UsageFlags::HOST_ACCESS);

        let mut index_buffer = Buffer::new(
            self.device.clone(),
            INDEX_BUFFER_SIZE * size_of::<u32>() as u64,
            vk::BufferUsageFlags::INDEX_BUFFER,
        );
        index_buffer.allocate_memory(gpu_alloc::UsageFlags::HOST_ACCESS);

        self.vertex_buffers.push(vertex_buffer);
        self.index_buffers.push(index_buffer);
    }

    fn create_swapchain(&mut self, window: &Window) {
        self.swapchain =
            Swapchain::new(self.device.clone(), window, vk::PresentModeKHR::MAILBOX_KHR);
    }

    fn create_depth_buffer(&mut self, window: &Window) {
        let extent = vk::Extent2D {
            width: window.inner_size().width,
            height: window.inner_size().height,
        };
        self.depth_buffer = DepthBuffer::new(self.device.clone(), &self.command_pool, extent);
    }

    fn create_framebuffers(&mut self) {
        for swapchain_image_view in self.swapchain.image_views() {
            self.framebuffers.push(Framebuffer::new(
                self.device.clone(),
                swapchain_image_view,
                &self.render_pass,
                &self.swapchain,
                &self.depth_buffer,
            ));
        }
    }

    fn create_pipelines(&mut self) {
        self.graphics_pipeline = GraphicsPipeline::new(
            self.device.clone(),
            &self.swapchain,
            &self.render_pass,
            &self.uniform_buffers,
            false,
        );

        self.ui_pipeline =
            GraphicsPipeline::new_ui(self.device.clone(), &self.swapchain, &self.render_pass);
    }

    fn create_command_buffers(&mut self) {
        let count = self.swapchain.images().len();
        self.command_pool.allocate(count as _);
    }

    fn create_sync_structures(&mut self) {
        let count = self.swapchain.images().len();

        self.present_semaphores
            .resize_with(count, || Semaphore::new(self.device.clone()));
        self.render_semaphores
            .resize_with(count, || Semaphore::new(self.device.clone()));
        self.fences
            .resize_with(count, || Fence::new(self.device.clone(), true));
    }

    fn create_uniform_buffers(&mut self) {
        let count = self.swapchain.images().len();
        self.uniform_buffers
            .resize_with(count, || UniformBuffer::new(self.device.clone()));
    }

    pub fn upload_font_texture(&mut self, egui: &egui::CtxRef) {}
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.swapchain.cleanup();
        self.graphics_pipeline.cleanup();
        self.ui_pipeline.cleanup();
        self.render_pass.cleanup();
    }
}
