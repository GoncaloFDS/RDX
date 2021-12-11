use crate::camera::Camera;
use crate::engine::{Block, Chunk, ChunkCell, Position};
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
use crate::vulkan::image::Image;
use crate::vulkan::image_view::ImageView;
use crate::vulkan::model::Instance;
use crate::vulkan::raytracing::acceleration_structure::{
    get_total_memory_requirements, AccelerationStrutcture,
};
use crate::vulkan::raytracing::bottom_level_acceleration_structure::BottomLevelAccelerationStructure;
use crate::vulkan::raytracing::bottom_level_geometry::BottomLevelGeometry;
use crate::vulkan::raytracing::raytracing_pipeline::RaytracingPipeline;
use crate::vulkan::raytracing::raytracing_properties::RaytracingProperties;
use crate::vulkan::raytracing::shader_binding_table::{Entry, ShaderBindingTable};
use crate::vulkan::raytracing::top_level_acceleration_structure::TopLevelAccelerationStructure;
use crate::vulkan::render_pass::RenderPass;
use crate::vulkan::sampler::{Sampler, SamplerInfo};
use crate::vulkan::scene::Scene;
use crate::vulkan::semaphore::Semaphore;
use crate::vulkan::swapchain::Swapchain;
use crate::vulkan::texture::Texture;
use crate::vulkan::texture_image::TextureImage;
use crate::vulkan::uniform_buffer::{UniformBuffer, UniformBufferObject};
use crate::vulkan::vertex::EguiVertex;
use erupt::vk;
use glam::{vec2, vec3, vec4, Mat4};
use hecs::World;
use puffin::profile_scope;
use rayon::prelude::*;
use std::mem::size_of;
use std::rc::Rc;
use std::sync::Arc;
use winit::window::Window;

const VERTICES_PER_QUAD: u64 = 4;
const VERTEX_BUFFER_SIZE: u64 = 1024 * 1024 * VERTICES_PER_QUAD;
const INDEX_BUFFER_SIZE: u64 = 1024 * 1024 * 2;
const MAX_INSTANCE_COUNT: u64 = 600000;
const INSTANCE_BUFFER_SIZE: u64 =
    size_of::<vk::AccelerationStructureInstanceKHR>() as u64 * MAX_INSTANCE_COUNT;

pub struct DrawIndexed {
    pub vertex_offset: u64,
    pub index_offset: u64,
    pub index_count: u32,
    pub id: usize,
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
    raytracing_pipeline: RaytracingPipeline,
    framebuffers: Vec<Framebuffer>,
    present_semaphores: Vec<Semaphore>,
    render_semaphores: Vec<Semaphore>,
    fences: Vec<Fence>,
    vertex_buffers: Vec<Buffer>,
    index_buffers: Vec<Buffer>,
    material_buffer: Buffer,
    uniform_buffers: Vec<UniformBuffer>,
    current_frame: usize,
    egui_texture: Texture,
    egui_texture_image: TextureImage,
    texture_images: Vec<TextureImage>,
    sampler: Sampler,
    egui_texture_version: u64,
    draw_indexed: Vec<DrawIndexed>,
    top_structures: Vec<TopLevelAccelerationStructure>,
    bottom_structures: Vec<BottomLevelAccelerationStructure>,
    accumulation_image: Image,
    accumulation_image_view: ImageView,
    output_image: Image,
    output_image_view: ImageView,
    raytracing_properties: RaytracingProperties,
    blas_buffer: Buffer,
    blas_scratch_buffer: Buffer,
    top_structures_buffer: Buffer,
    top_structures_scratch_buffer: Buffer,
    instances_buffer: Buffer,
    shader_binding_table: ShaderBindingTable,
    static_vertex_buffer: Buffer,
    static_index_buffer: Buffer,
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
        let raytracing_pipeline = RaytracingPipeline::uninitialized(device.clone());
        let accumulation_image = Image::uninitialized(device.clone());
        let accumulation_image_view = ImageView::uninitialized(device.clone());
        let output_image = Image::uninitialized(device.clone());
        let output_image_view = ImageView::uninitialized(device.clone());
        let blas_buffer = Buffer::uninitialized(device.clone());
        let blas_scratch_buffer = Buffer::uninitialized(device.clone());
        let top_structures_buffer = Buffer::uninitialized(device.clone());
        let top_structures_scratch_buffer = Buffer::uninitialized(device.clone());
        let instances_buffer = Buffer::uninitialized(device.clone());
        let shader_binding_table = ShaderBindingTable::uninitialized(device.clone());
        let static_vertex_buffer = Buffer::uninitialized(device.clone());
        let static_index_buffer = Buffer::uninitialized(device.clone());
        let material_buffer = Buffer::uninitialized(device.clone());
        let egui_texture = Texture::default();
        let egui_texture_image = TextureImage::uninitialized(device.clone());

        let raytracing_properties = RaytracingProperties::new(&device);
        let sampler = Sampler::new(device.clone(), &SamplerInfo::default());

        Renderer {
            device,
            _debug_messenger: debug_messenger,
            command_pool,
            swapchain,
            depth_buffer,
            render_pass,
            graphics_pipeline,
            ui_pipeline,
            raytracing_pipeline,
            framebuffers: vec![],
            present_semaphores: vec![],
            render_semaphores: vec![],
            fences: vec![],
            vertex_buffers: vec![],
            index_buffers: vec![],
            material_buffer,
            uniform_buffers: vec![],
            current_frame: 0,
            egui_texture,
            egui_texture_image,
            texture_images: Default::default(),
            sampler,
            egui_texture_version: 0,
            draw_indexed: vec![],
            top_structures: vec![],
            bottom_structures: vec![],
            accumulation_image,
            accumulation_image_view,
            output_image,
            output_image_view,
            raytracing_properties,
            blas_buffer,
            blas_scratch_buffer,
            top_structures_buffer,
            top_structures_scratch_buffer,
            instances_buffer,
            shader_binding_table,
            static_vertex_buffer,
            static_index_buffer,
        }
    }

    fn create(&mut self, window: &Window) {
        self.create_swapchain(window);
        self.create_depth_buffer(window);
        self.create_default_render_pass();
        self.create_uniform_buffers();
        self.create_output_images();
        self.create_pipelines();
        self.create_shader_binding_table();
        self.create_framebuffers();
        self.create_command_buffers();
        self.create_sync_structures();
    }

    pub fn setup(&mut self, window: &Window) {
        self.create(window);
        self.allocate_ui_buffers();
    }

    fn reset(&mut self) {
        self.command_pool.reset();
        self.swapchain = Swapchain::uninitialized(self.device.clone());
        self.uniform_buffers.clear();
        self.framebuffers.clear();
        self.present_semaphores.clear();
        self.render_semaphores.clear();
        self.fences.clear();
        self.egui_texture_version = 0;
        self.current_frame = 0;
    }

    pub fn recreate(&mut self, window: &Window) {
        self.device.wait_idle();
        self.reset();
        self.create(window);
    }

    pub fn update(&mut self, camera: &Camera, ui: &mut UserInterface, world: &mut World) {
        puffin::profile_function!();
        let extent = self.swapchain.extent();
        let aspect_ratio = extent.width as f32 / extent.height as f32;
        let view_model = camera.view();
        let projection = camera.projection(aspect_ratio);
        let ubo = UniformBufferObject {
            view_model,
            projection,
            view_model_inverse: view_model.inverse(),
            projection_inverse: projection.inverse(),
        };
        self.uniform_buffers[self.current_frame].update_gpu_buffer(&ubo);

        ui.update();

        self.update_ui_buffers(ui);
        let texture = ui.egui().texture();
        if texture.version != self.egui_texture_version {
            self.update_font_texture(texture);
        }

        self.update_acceleration_structures(world);
        self.update_descriptor_sets();
    }

    fn update_descriptor_sets(&mut self) {
        let top_structures = &self.top_structures[0];
        let accumulation_view = &self.accumulation_image_view;
        let output_view = &self.output_image_view;
        let uniform_buffers = &self.uniform_buffers;
        let vertex_buffer = &self.static_vertex_buffer;
        let index_buffer = &self.static_index_buffer;
        let material_buffer = &self.material_buffer;
        let descriptor_set_manager = self.raytracing_pipeline.descriptor_set_manager();
        self.swapchain
            .images()
            .iter()
            .enumerate()
            .for_each(|(i, _)| {
                let top_structures_handle = [top_structures.handle()];
                let top_structures_info =
                    vk::WriteDescriptorSetAccelerationStructureKHRBuilder::new()
                        .acceleration_structures(&top_structures_handle);

                let accumulation_image_info = [vk::DescriptorImageInfoBuilder::new()
                    .image_view(accumulation_view.handle())
                    .image_layout(vk::ImageLayout::GENERAL)];

                let output_image_info = [vk::DescriptorImageInfoBuilder::new()
                    .image_view(output_view.handle())
                    .image_layout(vk::ImageLayout::GENERAL)];

                let uniform_buffer_info = [vk::DescriptorBufferInfoBuilder::new()
                    .buffer(uniform_buffers[i].buffer().handle())
                    .range(vk::WHOLE_SIZE)];

                let vertex_buffer_info = [vk::DescriptorBufferInfoBuilder::new()
                    .buffer(vertex_buffer.handle())
                    .range(vk::WHOLE_SIZE)];

                let index_buffer_info = [vk::DescriptorBufferInfoBuilder::new()
                    .buffer(index_buffer.handle())
                    .range(vk::WHOLE_SIZE)];

                let material_buffer_info = [vk::DescriptorBufferInfoBuilder::new()
                    .buffer(material_buffer.handle())
                    .range(vk::WHOLE_SIZE)];

                let image_infos: Vec<_> = self
                    .texture_images
                    .iter()
                    .map(|texture_image| {
                        vk::DescriptorImageInfoBuilder::new()
                            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .image_view(texture_image.image_view().handle())
                    })
                    .collect();

                let sampler_info =
                    [vk::DescriptorImageInfoBuilder::new().sampler(self.sampler.handle())];

                let descriptor_writes = [
                    descriptor_set_manager.bind_acceleration_structure(
                        i as u32,
                        0,
                        &top_structures_info,
                    ),
                    descriptor_set_manager.bind_image(i as u32, 1, &accumulation_image_info),
                    descriptor_set_manager.bind_image(i as u32, 2, &output_image_info),
                    descriptor_set_manager.bind_buffer(i as u32, 3, &uniform_buffer_info),
                    descriptor_set_manager.bind_buffer(i as u32, 4, &vertex_buffer_info),
                    descriptor_set_manager.bind_buffer(i as u32, 5, &index_buffer_info),
                    descriptor_set_manager.bind_buffer(i as u32, 6, &material_buffer_info),
                    descriptor_set_manager.bind_image(i as u32, 7, &image_infos),
                    descriptor_set_manager.bind_image(i as u32, 8, &sampler_info),
                ];

                descriptor_set_manager.update_descriptors(&descriptor_writes);
            });
    }

    fn update_ui_buffers(&mut self, ui: &mut UserInterface) {
        puffin::profile_function!();
        let mut vertex_start = 0;
        let mut index_start = 0;
        self.draw_indexed.clear();
        for egui::ClippedMesh(_rect, mesh) in ui.clipped_meshes() {
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

        self.egui_texture = font_texture;
        self.egui_texture_image = font_image;

        let descriptor_set_manager = self.ui_pipeline.descriptor_set_manager();
        self.swapchain
            .images()
            .iter()
            .enumerate()
            .for_each(|(i, _)| {
                let image_info = [vk::DescriptorImageInfoBuilder::new()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(self.egui_texture_image.image_view().handle())
                    .sampler(self.egui_texture_image.sampler().handle())];

                let descriptor_writes = [descriptor_set_manager.bind_image(i as _, 0, &image_info)];

                descriptor_set_manager.update_descriptors(&descriptor_writes);
            });
    }

    pub fn draw_frame(&mut self) {
        puffin::profile_function!();
        self.fences[self.current_frame].wait(u64::MAX);

        match self.get_current_frame() {
            None => {
                log::debug!("failed to acquire next swapchain image");
                return;
            }
            Some(current_frame) => self.current_frame = current_frame,
        };

        let command_buffer = self.command_pool.begin(self.current_frame as _);

        self.raytrace(command_buffer);

        command_buffer.begin_render_pass(
            &self.device,
            self.render_pass.handle(),
            self.framebuffers[self.current_frame].handle(),
            self.swapchain.extent(),
        );

        // self.render(command_buffer);
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

        command_buffer.bind_vertex_buffer(&self.device, &[&self.vertex_buffers[0]], &[0]);
        command_buffer.bind_index_buffer(&self.device, &self.index_buffers[0], 0);
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

    fn raytrace(&mut self, command_buffer: CommandBuffer) {
        let extent = self.swapchain.extent();

        let subresource_range = vk::ImageSubresourceRangeBuilder::new()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        command_buffer.image_memory_barrier(
            &self.device,
            self.accumulation_image.handle(),
            *subresource_range,
            vk::AccessFlags::empty(),
            vk::AccessFlags::SHADER_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
        );

        command_buffer.image_memory_barrier(
            &self.device,
            self.output_image.handle(),
            *subresource_range,
            vk::AccessFlags::empty(),
            vk::AccessFlags::SHADER_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
        );

        command_buffer.bind_pipeline(
            &self.device,
            vk::PipelineBindPoint::RAY_TRACING_KHR,
            self.raytracing_pipeline.handle(),
        );

        command_buffer.bind_descriptor_sets(
            &self.device,
            vk::PipelineBindPoint::RAY_TRACING_KHR,
            self.raytracing_pipeline.pipeline_layout().handle(),
            &[self
                .raytracing_pipeline
                .descriptor_set_manager()
                .descriptor_set(self.current_frame)],
        );

        let raygen_sbt = self.shader_binding_table.raygen_device_region();
        let miss_sbt = self.shader_binding_table.miss_device_region();
        let hit_sbt = self.shader_binding_table.hit_device_region();
        let callable_sbt = self.shader_binding_table.callable_device_region();

        command_buffer.trace_rays(
            &self.device,
            &raygen_sbt,
            &miss_sbt,
            &hit_sbt,
            &callable_sbt,
            extent,
        );

        command_buffer.image_memory_barrier(
            &self.device,
            self.output_image.handle(),
            *subresource_range,
            vk::AccessFlags::SHADER_WRITE,
            vk::AccessFlags::TRANSFER_READ,
            vk::ImageLayout::GENERAL,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        );

        let swapchain_image = self.swapchain.images()[self.current_frame];
        command_buffer.image_memory_barrier(
            &self.device,
            swapchain_image,
            *subresource_range,
            vk::AccessFlags::empty(),
            vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );

        let image_copy_region = vk::ImageCopyBuilder::new()
            .src_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .src_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .dst_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .dst_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            });

        command_buffer.copy_image(
            &self.device,
            self.output_image.handle(),
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            swapchain_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            image_copy_region,
        );

        command_buffer.image_memory_barrier(
            &self.device,
            swapchain_image,
            *subresource_range,
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::empty(),
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
        );
    }

    pub fn shutdown(&self) {
        self.device.wait_idle();
    }

    pub fn upload_scene_buffers(&mut self, scene: &Scene) {
        puffin::profile_function!();

        let mut staging_buffers = vec![];
        CommandPool::single_time_submit(&self.device, &self.command_pool, |command_buffer| {
            let (material_buffer, staging_material_buffer) = command_buffer
                .create_device_local_buffer_with_data::<_>(
                    self.device.clone(),
                    vk::BufferUsageFlags::STORAGE_BUFFER
                        | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
                    &scene.materials(),
                );
            self.material_buffer = material_buffer;
            staging_buffers.push(staging_material_buffer);

            for model in scene.models() {
                for mesh in model.meshes() {
                    let buffer_usage_flags = vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                        | vk::BufferUsageFlags::STORAGE_BUFFER
                        | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;

                    let (index_buffer, staging_index_buffer) = command_buffer
                        .create_device_local_buffer_with_data::<_>(
                            self.device.clone(),
                            vk::BufferUsageFlags::INDEX_BUFFER | buffer_usage_flags,
                            &mesh.indices(),
                        );
                    self.static_index_buffer = index_buffer;
                    staging_buffers.push(staging_index_buffer);

                    let (vertex_buffer, staging_vertex_buffer) = command_buffer
                        .create_device_local_buffer_with_data::<_>(
                            self.device.clone(),
                            vk::BufferUsageFlags::VERTEX_BUFFER | buffer_usage_flags,
                            &mesh.vertices(),
                        );
                    self.static_vertex_buffer = vertex_buffer;
                    staging_buffers.push(staging_vertex_buffer);
                }
            }
        });

        scene.textures().iter().for_each(|texture| {
            let texture_image = TextureImage::new(self.device.clone(), &self.command_pool, texture);
            self.texture_images.push(texture_image);
        });

        self.create_acceleration_structures();
    }

    fn create_default_render_pass(&mut self) {
        self.render_pass = RenderPass::new(
            self.device.clone(),
            &self.swapchain,
            &self.depth_buffer,
            vk::AttachmentLoadOp::DONT_CARE,
            vk::AttachmentLoadOp::DONT_CARE,
        );
    }

    fn allocate_ui_buffers(&mut self) {
        let mut vertex_buffer = Buffer::new(
            self.device.clone(),
            VERTEX_BUFFER_SIZE * size_of::<EguiVertex>() as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        );
        vertex_buffer.allocate_memory(
            gpu_alloc::UsageFlags::HOST_ACCESS | gpu_alloc::UsageFlags::DEVICE_ADDRESS,
        );

        let mut index_buffer = Buffer::new(
            self.device.clone(),
            INDEX_BUFFER_SIZE * size_of::<u32>() as u64,
            vk::BufferUsageFlags::INDEX_BUFFER,
        );
        index_buffer.allocate_memory(
            gpu_alloc::UsageFlags::HOST_ACCESS | gpu_alloc::UsageFlags::DEVICE_ADDRESS,
        );

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

        self.raytracing_pipeline =
            RaytracingPipeline::new(self.device.clone(), self.swapchain.images().len() as u32);
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

    fn create_acceleration_structures(&mut self) {
        CommandPool::single_time_submit(&self.device, &self.command_pool, |command_buffer| {
            Renderer::allocate_blas_buffers(
                self.device.clone(),
                &mut self.bottom_structures,
                &mut self.blas_buffer,
                &mut self.blas_scratch_buffer,
                &self.static_vertex_buffer,
                &self.static_index_buffer,
                &self.raytracing_properties,
            );
            Renderer::create_blas(
                command_buffer,
                &mut self.bottom_structures,
                &self.blas_scratch_buffer,
                &self.blas_buffer,
            );
            command_buffer.acceleration_structure_memory_barrier(&self.device);
            Renderer::allocate_top_structures_buffers(
                self.device.clone(),
                &mut self.top_structures,
                &mut self.top_structures_buffer,
                &mut self.top_structures_scratch_buffer,
                &mut self.instances_buffer,
                &self.raytracing_properties,
            );
            Renderer::create_tlas(
                command_buffer,
                &mut self.top_structures,
                &self.top_structures_scratch_buffer,
                &self.top_structures_buffer,
            );
        });
    }

    fn update_acceleration_structures(&mut self, world: &mut World) {
        puffin::profile_function!();
        let mut instances = vec![];
        world.query::<&Chunk>().iter_batched(8).for_each(|batch| {
            profile_scope!("for each");
            batch.for_each(|(id, chunk)| {
                for x in 0..32 {
                    for y in 0..32 {
                        for z in 0..32 {
                            unsafe {
                                let pos = chunk.cells[x][y][z].position;
                                let left = if x > 0 {
                                    chunk.cells[x - 1][y][z].block
                                } else {
                                    Block::Empty
                                };
                                let right = if x < 31 {
                                    chunk.cells[x + 1][y][z].block
                                } else {
                                    Block::Empty
                                };
                                let top = if y < 31 {
                                    chunk.cells[x][y + 1][z].block
                                } else {
                                    Block::Empty
                                };
                                let bot = if y > 0 {
                                    chunk.cells[x][y - 1][z].block
                                } else {
                                    Block::Empty
                                };
                                let front = if z < 31 {
                                    chunk.cells[x][y][z + 1].block
                                } else {
                                    Block::Empty
                                };
                                let back = if z > 0 {
                                    chunk.cells[x][y][z - 1].block
                                } else {
                                    Block::Empty
                                };
                                if left == Block::Empty
                                    || right == Block::Empty
                                    || top == Block::Empty
                                    || bot == Block::Empty
                                    || front == Block::Empty
                                    || back == Block::Empty
                                {
                                    instances.push(Instance::new(
                                        0,
                                        0,
                                        Mat4::from_translation(vec3(
                                            (pos.x + 32 * chunk.x) as f32,
                                            pos.y as f32,
                                            (pos.z + 32 * chunk.y) as f32,
                                        )),
                                    ));
                                }
                            }
                        }
                    }
                }
            });
        });
        // log::debug!("instances: {}", instances.len());
        let old = self.top_structures.pop().unwrap();
        self.submit_top_structures_update(instances, old);
    }

    fn submit_top_structures_update(
        &mut self,
        instances: Vec<Instance>,
        old: TopLevelAccelerationStructure,
    ) {
        puffin::profile_function!();
        CommandPool::single_time_submit(&self.device, &self.command_pool, |command_buffer| {
            puffin::profile_scope!("top_structures update");
            let bottom_structures = &self.bottom_structures;
            let blas_addresses = bottom_structures
                .iter()
                .map(|b| b.get_address())
                .collect::<Vec<_>>();
            let instances = instances
                .par_iter()
                .map(|instance| {
                    TopLevelAccelerationStructure::create_instance(
                        blas_addresses[instance.blas_id() as usize],
                        instance.transform(),
                        instance.id(),
                        0,
                    )
                })
                .collect::<Vec<_>>();
            self.instances_buffer.write_data(&instances, 0);

            self.top_structures.push(TopLevelAccelerationStructure::new(
                self.device.clone(),
                self.raytracing_properties,
                self.instances_buffer.get_device_address(),
                MAX_INSTANCE_COUNT as u32,
            ));

            self.top_structures[0].generate(
                &command_buffer,
                self.top_structures_scratch_buffer.get_device_address(),
                0,
                &self.top_structures_buffer,
                0,
                Some(old.handle()),
            );
        });
    }

    fn allocate_blas_buffers(
        device: Rc<Device>,
        bottom_structures: &mut Vec<BottomLevelAccelerationStructure>,
        blas_buffer: &mut Buffer,
        blas_scratch_buffer: &mut Buffer,
        vertex_buffer: &Buffer,
        index_buffer: &Buffer,
        raytracing_properties: &RaytracingProperties,
    ) {
        bottom_structures.clear();

        let mut geometries = BottomLevelGeometry::default();
        // for model in models {
        geometries.add_geometry_triangles(vertex_buffer, index_buffer, 0, 24, 0, 36, true);

        bottom_structures.push(BottomLevelAccelerationStructure::new(
            device.clone(),
            *raytracing_properties,
            geometries,
        ));

        let memory_requirements = get_total_memory_requirements(bottom_structures);

        *blas_buffer = Buffer::new(
            device.clone(),
            memory_requirements.acceleration_structure_size,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
        );
        blas_buffer.allocate_memory(gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS);

        *blas_scratch_buffer = Buffer::new(
            device,
            memory_requirements.build_scratch_size,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::STORAGE_BUFFER,
        );
        blas_scratch_buffer.allocate_memory(gpu_alloc::UsageFlags::DEVICE_ADDRESS);
    }

    fn create_blas(
        command_buffer: CommandBuffer,
        bottom_structures: &mut [BottomLevelAccelerationStructure],
        scratch_buffer: &Buffer,
        buffer: &Buffer,
    ) {
        let mut result_offset = 0;
        let mut scratch_offset = 0;

        for b in bottom_structures {
            b.generate(
                &command_buffer,
                scratch_buffer,
                scratch_offset,
                buffer,
                result_offset,
            );

            result_offset += b.build_sizes().acceleration_structure_size;
            scratch_offset += b.build_sizes().build_scratch_size;
        }
    }

    fn allocate_top_structures_buffers(
        device: Rc<Device>,
        top_structures: &mut Vec<TopLevelAccelerationStructure>,
        top_structures_buffer: &mut Buffer,
        top_structures_scratch_buffer: &mut Buffer,
        instances_buffer: &mut Buffer,
        raytracing_properties: &RaytracingProperties,
    ) {
        *instances_buffer = Buffer::new(
            device.clone(),
            INSTANCE_BUFFER_SIZE,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );
        instances_buffer.allocate_memory(
            gpu_alloc::UsageFlags::HOST_ACCESS | gpu_alloc::UsageFlags::DEVICE_ADDRESS,
        );

        *top_structures = vec![TopLevelAccelerationStructure::new(
            device.clone(),
            *raytracing_properties,
            instances_buffer.get_device_address(),
            MAX_INSTANCE_COUNT as u32,
        )];

        let memory_requirements = get_total_memory_requirements(top_structures);

        *top_structures_buffer = Buffer::new(
            device.clone(),
            memory_requirements.acceleration_structure_size,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
        );
        top_structures_buffer.allocate_memory(gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS);

        *top_structures_scratch_buffer = Buffer::new(
            device.clone(),
            memory_requirements.build_scratch_size,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::STORAGE_BUFFER,
        );
        top_structures_scratch_buffer.allocate_memory(gpu_alloc::UsageFlags::DEVICE_ADDRESS);
    }

    fn create_tlas(
        command_buffer: CommandBuffer,
        top_structures: &mut [TopLevelAccelerationStructure],
        scratch_buffer: &Buffer,
        buffer: &Buffer,
    ) {
        top_structures[0].generate(
            &command_buffer,
            scratch_buffer.get_device_address(),
            0,
            buffer,
            0,
            None,
        );
    }

    fn create_output_images(&mut self) {
        let extent = self.swapchain.extent();
        let swap_format = self.swapchain.format();
        let tiling = vk::ImageTiling::OPTIMAL;

        self.accumulation_image = Image::new(
            self.device.clone(),
            extent,
            vk::Format::R32G32B32A32_SFLOAT,
            Some(tiling),
            Some(vk::ImageUsageFlags::STORAGE),
        );
        self.accumulation_image.allocate_memory();

        self.accumulation_image_view = ImageView::new(
            self.device.clone(),
            self.accumulation_image.handle(),
            self.accumulation_image.format(),
            vk::ImageAspectFlags::COLOR,
        );

        self.output_image = Image::new(
            self.device.clone(),
            extent,
            swap_format,
            Some(tiling),
            Some(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC),
        );
        self.output_image.allocate_memory();

        self.output_image_view = ImageView::new(
            self.device.clone(),
            self.output_image.handle(),
            self.output_image.format(),
            vk::ImageAspectFlags::COLOR,
        );
    }

    fn create_shader_binding_table(&mut self) {
        let raygen_groups = [Entry::new(self.raytracing_pipeline.raygen_index())];
        let miss_groups = [Entry::new(self.raytracing_pipeline.miss_index())];
        let hit_groups = [Entry::new(
            self.raytracing_pipeline.triangle_hit_group_index(),
        )];
        self.shader_binding_table = ShaderBindingTable::new(
            self.device.clone(),
            &self.raytracing_pipeline,
            &self.raytracing_properties,
            &raygen_groups,
            &miss_groups,
            &hit_groups,
        )
    }
}
