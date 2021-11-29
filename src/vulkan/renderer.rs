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
use crate::vulkan::graphics_pipeline::{GraphicsPipeline, PushConstants};
use crate::vulkan::model::DrawIndexed;
use crate::vulkan::render_pass::RenderPass;
use crate::vulkan::scene::Scene;
use crate::vulkan::semaphore::Semaphore;
use crate::vulkan::swapchain::Swapchain;
use crate::vulkan::texture::Texture;
use crate::vulkan::texture_image::TextureImage;
use crate::vulkan::uniform_buffer::{UniformBuffer, UniformBufferObject};
use crate::vulkan::vertex::EguiVertex;
use erupt::vk;
use glam::{vec2, vec4};
use std::collections::HashMap;
use std::mem::size_of;
use std::rc::Rc;
use std::sync::Arc;
use winit::window::Window;

const VERTICES_PER_QUAD: u64 = 4;
const VERTEX_BUFFER_SIZE: u64 = 1024 * 1024 * VERTICES_PER_QUAD;
const INDEX_BUFFER_SIZE: u64 = 1024 * 1024 * 2;

const FONT_IMAGE_ID: u32 = 10;

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
    draw_indexed: Vec<DrawIndexed>,
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
            draw_indexed: vec![],
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

    pub fn update(&mut self, camera: &Camera, ui: &mut UserInterface) {
        let extent = self.swapchain.extent();
        let aspect_ratio = extent.width as f32 / extent.height as f32;
        let ubo = UniformBufferObject {
            view_model: camera.view().into(),
            projection: camera.projection(aspect_ratio).into(),
        };
        self.uniform_buffers[self.current_frame].update_gpu_buffer(&ubo);

        ui.update();

        self.update_ui_buffers(ui);
        let texture = ui.egui().texture();
        if texture.version != self.egui_texture_version {
            self.update_font_texture(texture);
        }
    }

    fn update_ui_buffers(&mut self, ui: &mut UserInterface) {
        let mut vertex_start = 0;
        let mut index_start = 0;
        self.draw_indexed.clear();
        for egui::ClippedMesh(rect, mesh) in ui.clipped_meshes() {
            if mesh.vertices.is_empty() || mesh.indices.is_empty() {
                continue;
            }

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

            let id = 0;

            self.vertex_buffers[id].write_data(&vertices, vertex_start);
            self.index_buffers[id].write_data(&mesh.indices, index_start);

            self.draw_indexed.push(DrawIndexed {
                vertex_offset: vertex_start * size_of::<EguiVertex>() as u64,
                index_offset: index_start * size_of::<u32>() as u64,
                index_count: mesh.indices.len() as u32,
                id,
            });

            vertex_start += mesh.vertices.len() as u64;
            index_start += mesh.indices.len() as u64;
        }
    }

    fn update_font_texture(&mut self, texture: Arc<egui::Texture>) {
        self.egui_texture_version = texture.version;

        let data = texture
            .pixels
            .iter()
            .flat_map(|&r| vec![r, r, r, r])
            .collect::<Vec<_>>();

        let font_texture = Texture::new(texture.width as _, texture.height as _, data);
        let font_image = TextureImage::new(self.device.clone(), &self.command_pool, &font_texture);

        self.textures.insert(FONT_IMAGE_ID, font_texture);
        self.texture_images.insert(FONT_IMAGE_ID, font_image);

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

    pub fn draw_frame(&mut self) {
        self.fences[self.current_frame].wait(u64::MAX);

        match self.get_current_frame() {
            None => {
                log::debug!("failed to acquire next swapchain image");
                return;
            }
            Some(current_frame) => self.current_frame = current_frame,
        };

        let command_buffer = self.command_pool.begin(self.current_frame as _);

        command_buffer.begin_render_pass(
            &self.device,
            self.render_pass.handle(),
            self.framebuffers[self.current_frame].handle(),
            self.swapchain.extent(),
        );

        self.render(command_buffer);
        self.render_ui(command_buffer);

        command_buffer.end_render_pass(&self.device);
        command_buffer.end(&self.device);

        self.fences[self.current_frame].reset();

        self.device.submit(
            &[command_buffer.handle()],
            &[self.render_semaphores[self.current_frame].handle()],
            &[self.present_semaphores[self.current_frame].handle()],
            &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            Some(self.fences[self.current_frame].handle()),
        );
    }

    fn get_current_frame(&self) -> Option<usize> {
        self.swapchain.acquire_next_image(
            u64::MAX,
            Some(self.render_semaphores[self.current_frame].handle()),
        )
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

    fn render(&self, command_buffer: CommandBuffer) {
        command_buffer.bind_pipeline(
            &self.device,
            vk::PipelineBindPoint::GRAPHICS,
            self.graphics_pipeline.handle(),
        );
        command_buffer.bind_descriptor_sets(
            &self.device,
            vk::PipelineBindPoint::GRAPHICS,
            self.graphics_pipeline.pipeline_layout().handle(),
            &[self
                .graphics_pipeline
                .descriptor_set_manager()
                .descriptor_set(self.current_frame)],
        );

        command_buffer.bind_vertex_buffer(&self.device, &[&self.vertex_buffers[1]], &[0]);
        command_buffer.bind_index_buffer(&self.device, &self.index_buffers[1], 0);
        command_buffer.draw_indexed(&self.device, 3);
    }

    fn render_ui(&mut self, command_buffer: CommandBuffer) {
        let push_constants = PushConstants {
            screen_size: vec2(
                self.swapchain.extent().width as f32,
                self.swapchain.extent().height as f32,
            ),
        };

        command_buffer.bind_pipeline(
            &self.device,
            vk::PipelineBindPoint::GRAPHICS,
            self.ui_pipeline.handle(),
        );

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

        for draw_indexed in &self.draw_indexed {
            command_buffer.bind_vertex_buffer(
                &self.device,
                &[&self.vertex_buffers[draw_indexed.id]],
                &[draw_indexed.vertex_offset],
            );

            command_buffer.bind_index_buffer(
                &self.device,
                &self.index_buffers[draw_indexed.id],
                draw_indexed.index_offset,
            );

            command_buffer.draw_indexed(&self.device, draw_indexed.index_count);
        }
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
