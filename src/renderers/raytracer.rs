use crate::camera::Camera;
use crate::chunk::Chunk;
use crate::renderers::Renderer;
use crate::scene::Scene;
use crate::user_interface::UserInterface;
use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::command_pool::CommandPool;
use crate::vulkan::descriptor_binding::DescriptorBinding;
use crate::vulkan::descriptor_set_manager::DescriptorSetManager;
use crate::vulkan::device::Device;
use crate::vulkan::image::Image;
use crate::vulkan::instance::Instance;
use crate::vulkan::pipeline_layout::PipelineLayout;
use crate::vulkan::raytracing::acceleration_structure::{
    get_total_memory_requirements, AccelerationStructure,
};
use crate::vulkan::raytracing::bottom_level_acceleration_structure::BottomLevelAccelerationStructure;
use crate::vulkan::raytracing::bottom_level_geometry::BottomLevelGeometry;
use crate::vulkan::raytracing::raytracing_pipeline::RaytracingPipeline;
use crate::vulkan::raytracing::raytracing_properties::RaytracingProperties;
use crate::vulkan::raytracing::shader_binding_table::{Entry, ShaderBindingTable};
use crate::vulkan::raytracing::top_level_acceleration_structure::{
    AccelerationInstance, TopLevelAccelerationStructure,
};
use crate::vulkan::sampler::{Sampler, SamplerInfo};
use crate::vulkan::shader_module::{Shader, ShaderModule};
use crate::vulkan::subresource_range::SubresourceRange;
use crate::vulkan::texture_image::TextureImage;
use crate::vulkan::uniform_buffer::{UniformBuffer, UniformBufferObject};
use crate::vulkan::vertex::{ModelVertex, Std430ModelVertex};
use bevy_ecs::prelude::World;
use erupt::vk;
use glam::*;
use rayon::prelude::*;
use std::mem::size_of;

const STAGING_BUFFER_SIZE: u64 = 1024 * 1024 * 1000;
const VERTEX_BUFFER_SIZE: u64 = 1024 * 1024 * 1000;
const INDEX_BUFFER_SIZE: u64 = 1024 * 1024 * 1000;
const BLAS_BUFFER_SIZE: u64 = 1024 * 1024 * 1600;
const TLAS_BUFFER_SIZE: u64 = 1024 * 1024 * 16;
const MAX_INSTANCE_COUNT: u64 = 2048;
const INSTANCE_BUFFER_SIZE: u64 =
    size_of::<vk::AccelerationStructureInstanceKHR>() as u64 * MAX_INSTANCE_COUNT * 100;

pub struct Raytracer {
    pipeline: RaytracingPipeline,
    pipeline_layout: PipelineLayout,
    descriptor_set_manager: DescriptorSetManager,
    top_structures: Vec<TopLevelAccelerationStructure>,
    bottom_structures: Vec<BottomLevelAccelerationStructure>,
    accumulation_image: Image,
    output_image: Image,
    raytracing_properties: RaytracingProperties,
    blas_buffer: Buffer,
    blas_scratch_buffer: Buffer,
    top_structures_buffer: Buffer,
    top_structures_scratch_buffer: Buffer,
    instances_buffer: Buffer,
    shader_binding_table: ShaderBindingTable,
    vertex_buffer: Buffer,
    staging_vertex_buffer: Buffer,
    index_buffer: Buffer,
    staging_index_buffer: Buffer,
    offset_buffer: Buffer,
    uniform_buffers: Vec<UniformBuffer>,
    material_buffer: Buffer,
    texture_images: Vec<TextureImage>,
    sampler: Sampler,
    first: bool,
}

impl Raytracer {
    pub fn new(device: &mut Device, raytracing_properties: RaytracingProperties) -> Self {
        let first = true;
        let descriptor_bindings = [
            // TAS
            DescriptorBinding::new(
                0,
                1,
                vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
                vk::ShaderStageFlags::RAYGEN_KHR,
            ),
            // Accumulation and output Image
            DescriptorBinding::new(
                1,
                1,
                vk::DescriptorType::STORAGE_IMAGE,
                vk::ShaderStageFlags::RAYGEN_KHR,
            ),
            DescriptorBinding::new(
                2,
                1,
                vk::DescriptorType::STORAGE_IMAGE,
                vk::ShaderStageFlags::RAYGEN_KHR,
            ),
            // Camera
            DescriptorBinding::new(
                3,
                1,
                vk::DescriptorType::UNIFORM_BUFFER,
                vk::ShaderStageFlags::RAYGEN_KHR | vk::ShaderStageFlags::MISS_KHR,
            ),
            // Vertex, Index, Material
            DescriptorBinding::new(
                4,
                1,
                vk::DescriptorType::STORAGE_BUFFER,
                vk::ShaderStageFlags::CLOSEST_HIT_KHR,
            ),
            DescriptorBinding::new(
                5,
                1,
                vk::DescriptorType::STORAGE_BUFFER,
                vk::ShaderStageFlags::CLOSEST_HIT_KHR,
            ),
            DescriptorBinding::new(
                6,
                1,
                vk::DescriptorType::STORAGE_BUFFER,
                vk::ShaderStageFlags::CLOSEST_HIT_KHR,
            ),
            DescriptorBinding::new(
                7,
                1,
                vk::DescriptorType::STORAGE_BUFFER,
                vk::ShaderStageFlags::CLOSEST_HIT_KHR,
            ),
            // Textures
            DescriptorBinding::new(
                8,
                1,
                vk::DescriptorType::SAMPLED_IMAGE,
                vk::ShaderStageFlags::CLOSEST_HIT_KHR,
            ),
            DescriptorBinding::new(
                9,
                1,
                vk::DescriptorType::SAMPLER,
                vk::ShaderStageFlags::CLOSEST_HIT_KHR,
            ),
        ];

        let descriptor_set_manager = DescriptorSetManager::new(device, &descriptor_bindings, 3);

        let pipeline_layout = PipelineLayout::new(
            device,
            &[descriptor_set_manager.descriptor_set_layout()],
            &[],
        );

        let raytracing_shader = ShaderModule::new(device, Shader::Raytracing);

        let shader_stages = [
            raytracing_shader.shader_stage(vk::ShaderStageFlagBits::RAYGEN_KHR, "raygen\0"),
            raytracing_shader.shader_stage(vk::ShaderStageFlagBits::MISS_KHR, "miss\0"),
            raytracing_shader
                .shader_stage(vk::ShaderStageFlagBits::CLOSEST_HIT_KHR, "closest_hit\0"),
        ];

        let raygen_index = 0;
        let miss_index = 1;
        let triangle_hit_group_index = 2;
        let shader_groups = [
            vk::RayTracingShaderGroupCreateInfoKHRBuilder::new()
                ._type(vk::RayTracingShaderGroupTypeKHR::GENERAL_KHR)
                .general_shader(raygen_index)
                .closest_hit_shader(vk::SHADER_UNUSED_KHR)
                .any_hit_shader(vk::SHADER_UNUSED_KHR)
                .intersection_shader(vk::SHADER_UNUSED_KHR),
            vk::RayTracingShaderGroupCreateInfoKHRBuilder::new()
                ._type(vk::RayTracingShaderGroupTypeKHR::GENERAL_KHR)
                .general_shader(miss_index)
                .closest_hit_shader(vk::SHADER_UNUSED_KHR)
                .any_hit_shader(vk::SHADER_UNUSED_KHR)
                .intersection_shader(vk::SHADER_UNUSED_KHR),
            vk::RayTracingShaderGroupCreateInfoKHRBuilder::new()
                ._type(vk::RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP_KHR)
                .general_shader(vk::SHADER_UNUSED_KHR)
                .closest_hit_shader(triangle_hit_group_index)
                .any_hit_shader(vk::SHADER_UNUSED_KHR)
                .intersection_shader(vk::SHADER_UNUSED_KHR),
        ];

        let pipeline_info = vk::RayTracingPipelineCreateInfoKHRBuilder::new()
            .flags(vk::PipelineCreateFlags::empty())
            .stages(&shader_stages)
            .groups(&shader_groups)
            .max_pipeline_ray_recursion_depth(1)
            .layout(pipeline_layout.handle())
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(0);

        let pipeline =
            RaytracingPipeline::new(device, &[pipeline_info], vk::PipelineCache::default());

        let raygen_groups = [Entry::new(raygen_index)];
        let miss_groups = [Entry::new(miss_index)];
        let hit_groups = [Entry::new(triangle_hit_group_index)];
        let shader_binding_table = ShaderBindingTable::new(
            device,
            &pipeline,
            &raytracing_properties,
            &raygen_groups,
            &miss_groups,
            &hit_groups,
        );

        // tlas
        let instances_buffer = Buffer::empty(
            device,
            INSTANCE_BUFFER_SIZE,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            gpu_alloc::UsageFlags::HOST_ACCESS | gpu_alloc::UsageFlags::DEVICE_ADDRESS,
        );

        let top_structures = vec![TopLevelAccelerationStructure::new(
            device,
            raytracing_properties,
            instances_buffer.get_device_address(device),
            MAX_INSTANCE_COUNT as u32,
        )];

        let top_structures_buffer = Buffer::empty(
            device,
            TLAS_BUFFER_SIZE,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
            gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
        );

        let top_structures_scratch_buffer = Buffer::empty(
            device,
            TLAS_BUFFER_SIZE,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::STORAGE_BUFFER,
            gpu_alloc::UsageFlags::DEVICE_ADDRESS,
        );

        // blas
        let blas_buffer = Buffer::empty(
            device,
            BLAS_BUFFER_SIZE,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
            gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
        );

        let blas_scratch_buffer = Buffer::empty(
            device,
            BLAS_BUFFER_SIZE,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::STORAGE_BUFFER,
            gpu_alloc::UsageFlags::DEVICE_ADDRESS,
        );

        // images
        let accumulation_image = Image::new(
            device,
            device.swapchain().extent(),
            vk::Format::R32G32B32A32_SFLOAT,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::STORAGE,
            vk::ImageAspectFlags::COLOR,
        );

        let extent = device.swapchain().extent();
        let format = device.swapchain().surface_format();
        let output_image = Image::new(
            device,
            extent,
            format.format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC,
            vk::ImageAspectFlags::COLOR,
        );

        let buffer_usage_flags = vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | vk::BufferUsageFlags::STORAGE_BUFFER
            | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
        let vertex_buffer = Buffer::empty(
            device,
            VERTEX_BUFFER_SIZE,
            vk::BufferUsageFlags::VERTEX_BUFFER
                | vk::BufferUsageFlags::TRANSFER_DST
                | buffer_usage_flags,
            gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
        );
        let staging_vertex_buffer = Buffer::empty(
            device,
            STAGING_BUFFER_SIZE,
            vk::BufferUsageFlags::TRANSFER_SRC,
            gpu_alloc::UsageFlags::HOST_ACCESS,
        );

        let index_buffer = Buffer::empty(
            device,
            INDEX_BUFFER_SIZE,
            vk::BufferUsageFlags::INDEX_BUFFER
                | vk::BufferUsageFlags::TRANSFER_DST
                | buffer_usage_flags,
            gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
        );
        let staging_index_buffer = Buffer::empty(
            device,
            STAGING_BUFFER_SIZE,
            vk::BufferUsageFlags::TRANSFER_SRC,
            gpu_alloc::UsageFlags::HOST_ACCESS,
        );

        let offset_buffer = Buffer::empty(
            device,
            INSTANCE_BUFFER_SIZE,
            vk::BufferUsageFlags::STORAGE_BUFFER | buffer_usage_flags,
            gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS | gpu_alloc::UsageFlags::HOST_ACCESS,
        );

        let uniform_buffers = (0..3)
            .map(|_| UniformBuffer::new(device))
            .collect::<Vec<_>>();

        let material_buffer = Buffer::empty(
            device,
            VERTEX_BUFFER_SIZE,
            vk::BufferUsageFlags::STORAGE_BUFFER | buffer_usage_flags,
            gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
        );

        let sampler = Sampler::new(device, &SamplerInfo::default());

        Raytracer {
            pipeline,
            pipeline_layout,
            descriptor_set_manager,
            top_structures,
            bottom_structures: vec![],
            accumulation_image,
            output_image,
            raytracing_properties,
            blas_buffer,
            blas_scratch_buffer,
            top_structures_buffer,
            top_structures_scratch_buffer,
            instances_buffer,
            shader_binding_table,
            vertex_buffer,
            staging_vertex_buffer,
            index_buffer,
            staging_index_buffer,
            offset_buffer,
            uniform_buffers,
            material_buffer,
            texture_images: vec![],
            sampler,
            first,
        }
    }

    fn update_descriptors(&mut self, device: &mut Device, current_image: usize, camera: &Camera) {
        let extent = device.swapchain().extent();
        let aspect_ratio = extent.width as f32 / extent.height as f32;
        let view_model = camera.view();
        let projection = camera.projection(aspect_ratio);
        let ubo = UniformBufferObject {
            view_model,
            projection,
            view_model_inverse: view_model.inverse(),
            projection_inverse: projection.inverse(),
        };
        self.uniform_buffers[current_image].update_gpu_buffer(device, &ubo);

        let top_structures_handle = [self.top_structures[0].handle()];
        let mut top_structures_info = vk::WriteDescriptorSetAccelerationStructureKHRBuilder::new()
            .acceleration_structures(&top_structures_handle);

        let accumulation_image_info = [vk::DescriptorImageInfoBuilder::new()
            .image_view(self.accumulation_image.view())
            .image_layout(vk::ImageLayout::GENERAL)];

        let output_image_info = [vk::DescriptorImageInfoBuilder::new()
            .image_view(self.output_image.view())
            .image_layout(vk::ImageLayout::GENERAL)];

        let uniform_buffer_info = [vk::DescriptorBufferInfoBuilder::new()
            .buffer(self.uniform_buffers[current_image].buffer().handle())
            .range(vk::WHOLE_SIZE)];

        let vertex_buffer_info = [vk::DescriptorBufferInfoBuilder::new()
            .buffer(self.vertex_buffer.handle())
            .range(vk::WHOLE_SIZE)];

        let index_buffer_info = [vk::DescriptorBufferInfoBuilder::new()
            .buffer(self.index_buffer.handle())
            .range(vk::WHOLE_SIZE)];

        let offset_buffer_info = [vk::DescriptorBufferInfoBuilder::new()
            .buffer(self.offset_buffer.handle())
            .range(vk::WHOLE_SIZE)];

        let material_buffer_info = [vk::DescriptorBufferInfoBuilder::new()
            .buffer(self.material_buffer.handle())
            .range(vk::WHOLE_SIZE)];

        let image_infos: Vec<_> = self
            .texture_images
            .iter()
            .map(|texture_image| {
                vk::DescriptorImageInfoBuilder::new()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(texture_image.image().view())
            })
            .collect();

        let sampler_info = [vk::DescriptorImageInfoBuilder::new().sampler(self.sampler.handle())];

        let descriptor_writes = [
            self.descriptor_set_manager.bind_acceleration_structure(
                current_image,
                0,
                &mut top_structures_info,
            ),
            self.descriptor_set_manager
                .bind_image(current_image, 1, &accumulation_image_info),
            self.descriptor_set_manager
                .bind_image(current_image, 2, &output_image_info),
            self.descriptor_set_manager
                .bind_buffer(current_image, 3, &uniform_buffer_info),
            self.descriptor_set_manager
                .bind_buffer(current_image, 4, &vertex_buffer_info),
            self.descriptor_set_manager
                .bind_buffer(current_image, 5, &index_buffer_info),
            self.descriptor_set_manager
                .bind_buffer(current_image, 6, &material_buffer_info),
            self.descriptor_set_manager
                .bind_buffer(current_image, 7, &offset_buffer_info),
            self.descriptor_set_manager
                .bind_image(current_image, 8, &image_infos),
            self.descriptor_set_manager
                .bind_image(current_image, 9, &sampler_info),
        ];

        self.descriptor_set_manager
            .update_descriptors(device, &descriptor_writes);
    }
}

impl Renderer for Raytracer {
    fn fill_command_buffer(
        &self,
        device: &Device,
        command_buffer: &CommandBuffer,
        current_image: usize,
    ) {
        puffin::profile_function!();
        command_buffer.image_memory_barrier(
            device,
            self.accumulation_image.handle(),
            SubresourceRange::with_aspect(vk::ImageAspectFlags::COLOR),
            vk::PipelineStageFlags2::ALL_COMMANDS,
            vk::PipelineStageFlags2::ALL_COMMANDS,
            vk::AccessFlags2::NONE,
            vk::AccessFlags2::SHADER_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
        );

        command_buffer.image_memory_barrier(
            device,
            self.output_image.handle(),
            SubresourceRange::with_aspect(vk::ImageAspectFlags::COLOR),
            vk::PipelineStageFlags2::ALL_COMMANDS,
            vk::PipelineStageFlags2::ALL_COMMANDS,
            vk::AccessFlags2::NONE,
            vk::AccessFlags2::SHADER_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
        );

        command_buffer.bind_pipeline(
            device,
            vk::PipelineBindPoint::RAY_TRACING_KHR,
            self.pipeline.handle(),
        );

        command_buffer.bind_descriptor_sets(
            device,
            vk::PipelineBindPoint::RAY_TRACING_KHR,
            self.pipeline_layout.handle(),
            &[self.descriptor_set_manager.descriptor_set(current_image)],
        );

        let extent = device.swapchain().extent();
        command_buffer.trace_rays(
            device,
            &self.shader_binding_table.raygen_device_region(device),
            &self.shader_binding_table.miss_device_region(device),
            &self.shader_binding_table.hit_device_region(device),
            &self.shader_binding_table.callable_device_region(device),
            extent,
        );

        command_buffer.image_memory_barrier(
            device,
            self.output_image.handle(),
            SubresourceRange::with_aspect(vk::ImageAspectFlags::COLOR),
            vk::PipelineStageFlags2::ALL_COMMANDS,
            vk::PipelineStageFlags2::ALL_COMMANDS,
            vk::AccessFlags2::SHADER_WRITE,
            vk::AccessFlags2::TRANSFER_READ,
            vk::ImageLayout::GENERAL,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        );

        let swapchain_image = device.swapchain().images()[current_image];
        command_buffer.image_memory_barrier(
            device,
            swapchain_image,
            SubresourceRange::with_aspect(vk::ImageAspectFlags::COLOR),
            vk::PipelineStageFlags2::ALL_COMMANDS,
            vk::PipelineStageFlags2::ALL_COMMANDS,
            vk::AccessFlags2::NONE,
            vk::AccessFlags2::TRANSFER_WRITE,
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
            device,
            self.output_image.handle(),
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            swapchain_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            image_copy_region,
        );

        command_buffer.image_memory_barrier(
            device,
            swapchain_image,
            SubresourceRange::with_aspect(vk::ImageAspectFlags::COLOR),
            vk::PipelineStageFlags2::ALL_COMMANDS,
            vk::PipelineStageFlags2::ALL_COMMANDS,
            vk::AccessFlags2::TRANSFER_WRITE,
            vk::AccessFlags2::NONE,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
        );
    }

    fn update(
        &mut self,
        device: &mut Device,
        current_image: usize,
        camera: &Camera,
        ui: &mut UserInterface,
        world: &mut World,
        scene: &Scene,
    ) {
        puffin::profile_function!();

        if self.first {
            log::debug!("first update");
            self.first = false;
            scene.textures().iter().for_each(|texture| {
                self.texture_images.push(TextureImage::new(device, texture));
            });

            log::debug!("collect chunks");
            let mut chunks = world
                .query::<&mut Chunk>()
                .iter_mut(world)
                .collect::<Vec<_>>();
            log::debug!("compute chunk meshes");
            let meshes = chunks
                .par_iter_mut()
                .map(|chunk| chunk.compute_chunk_mesh(scene))
                .collect::<Vec<_>>();

            log::debug!("compute chunk offsets");
            let mut vertex_offset = 0;
            let mut index_offset = 0;
            let offsets = meshes
                .iter()
                .map(|mesh| {
                    let offsets = (vertex_offset, index_offset);
                    vertex_offset += mesh.vertices().len() as u32;
                    index_offset += mesh.indices().len() as u32;
                    offsets
                })
                .collect::<Vec<_>>();

            log::debug!("compute chunk vertices");
            let vertices = meshes
                .iter()
                .fold(vec![], |mut acc: Vec<Std430ModelVertex>, mesh| {
                    acc.extend(mesh.vertices());
                    acc
                });

            log::debug!("compute chunk indices");
            let indices = meshes.iter().fold(vec![], |mut acc: Vec<u32>, mesh| {
                acc.extend(mesh.indices());
                acc
            });

            log::debug!("add chunk geometries");
            let mut vertex_offset = 0;
            let mut index_offset = 0;
            for mesh in meshes {
                let vertices_count = mesh.vertices().len() as u32;
                let indices_count = mesh.indices().len() as u32;
                let mut geometries = BottomLevelGeometry::default();
                geometries.add_geometry_triangles(
                    device,
                    &self.vertex_buffer,
                    &self.index_buffer,
                    vertex_offset,
                    vertices_count,
                    index_offset,
                    indices_count,
                    true,
                );

                vertex_offset += vertices_count * size_of::<Std430ModelVertex>() as u32;
                index_offset += indices_count * size_of::<u32>() as u32;

                self.bottom_structures
                    .push(BottomLevelAccelerationStructure::new(
                        device,
                        self.raytracing_properties,
                        geometries,
                    ));
            }

            log::debug!("submit meshes");
            CommandPool::single_time_submit(device, |command_buffer| {
                self.staging_vertex_buffer.write_data(device, &vertices, 0);
                command_buffer.copy_buffer(
                    device,
                    &self.staging_vertex_buffer,
                    &self.vertex_buffer,
                    VERTEX_BUFFER_SIZE,
                );
                self.staging_index_buffer.write_data(device, &indices, 0);
                command_buffer.copy_buffer(
                    device,
                    &self.staging_index_buffer,
                    &self.index_buffer,
                    INDEX_BUFFER_SIZE,
                );
                self.offset_buffer.write_data(device, &offsets, 0);

                // blas
                let mut result_offset = 0;
                let mut scratch_offset = 0;
                for blas in &mut self.bottom_structures {
                    blas.generate(
                        device,
                        &command_buffer,
                        &self.blas_scratch_buffer,
                        scratch_offset,
                        &self.blas_buffer,
                        result_offset,
                    );
                    result_offset += blas.build_sizes().acceleration_structure_size;
                    scratch_offset += blas.build_sizes().build_scratch_size;
                }

                command_buffer.acceleration_structure_memory_barrier(device);
                // tlas
                let mut result_offset = 0;
                let mut scratch_offset = 0;
                for tlas in &mut self.top_structures {
                    tlas.generate(
                        device,
                        &command_buffer,
                        &self.top_structures_scratch_buffer,
                        scratch_offset,
                        &self.top_structures_buffer,
                        result_offset,
                        None,
                    );
                    result_offset += tlas.build_sizes().acceleration_structure_size;
                    scratch_offset += tlas.build_sizes().build_scratch_size;
                }

                // instances
                let blas_address = &self
                    .bottom_structures
                    .iter()
                    .map(|blas| blas.get_address(device))
                    .collect::<Vec<_>>();
                let instances = world
                    .query::<&Chunk>()
                    .iter(world)
                    .enumerate()
                    .map(|(id, chunk)| {
                        let instance =
                            AccelerationInstance::new(id as _, id as _, 0, chunk.transform());
                        instance.generate(blas_address[instance.blas_id() as usize])
                    })
                    .collect::<Vec<_>>();
                self.instances_buffer.write_data(device, &instances, 0);
            });
            log::debug!("uploading meshes: done");
        }

        self.update_descriptors(device, current_image, camera);
    }

    fn destroy(&mut self, device: &mut Device) {
        todo!()
    }
}
