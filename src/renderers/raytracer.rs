use crate::renderers::Renderer;
use crate::user_interface::UserInterface;
use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::command_pool::CommandPool;
use crate::vulkan::descriptor_binding::DescriptorBinding;
use crate::vulkan::descriptor_set_manager::DescriptorSetManager;
use crate::vulkan::device::Device;
use crate::vulkan::image::Image;
use crate::vulkan::pipeline_layout::PipelineLayout;
use crate::vulkan::raytracing::acceleration_structure::{
    get_total_memory_requirements, AccelerationStructure,
};
use crate::vulkan::raytracing::bottom_level_acceleration_structure::BottomLevelAccelerationStructure;
use crate::vulkan::raytracing::bottom_level_geometry::BottomLevelGeometry;
use crate::vulkan::raytracing::raytracing_pipeline::RaytracingPipeline;
use crate::vulkan::raytracing::raytracing_properties::RaytracingProperties;
use crate::vulkan::raytracing::shader_binding_table::{Entry, ShaderBindingTable};
use crate::vulkan::raytracing::top_level_acceleration_structure::TopLevelAccelerationStructure;
use crate::vulkan::sampler::{Sampler, SamplerInfo};
use crate::vulkan::shader_module::{Shader, ShaderModule};
use crate::vulkan::subresource_range::SubresourceRange;
use crate::vulkan::texture_image::TextureImage;
use crate::vulkan::uniform_buffer::UniformBuffer;
use crate::vulkan::vertex::ModelVertex;
use crevice::std430::AsStd430;
use erupt::vk;
use glam::{vec2, vec3};
use std::mem::size_of;

const VERTICES_PER_QUAD: u64 = 4;
const VERTEX_BUFFER_SIZE: u64 = 1024 * 1024 * VERTICES_PER_QUAD;
const INDEX_BUFFER_SIZE: u64 = 1024 * 1024 * 2;
const MAX_INSTANCE_COUNT: u64 = 2048;
const INSTANCE_BUFFER_SIZE: u64 =
    size_of::<vk::AccelerationStructureInstanceKHR>() as u64 * MAX_INSTANCE_COUNT;

pub struct Raytracer {
    pipeline: RaytracingPipeline,
    pipeline_layout: PipelineLayout,
    descriptor_set_manager: DescriptorSetManager,
    raygen_index: u32,
    miss_index: u32,
    triangle_hit_group_index: u32,
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
    index_buffer: Buffer,
    offset_buffer: Buffer,
    uniform_buffers: Vec<UniformBuffer>,
    material_buffer: Buffer,
    // sampler: Sampler,
}

impl Raytracer {
    pub fn new(device: &mut Device, raytracing_properties: RaytracingProperties) -> Self {
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
            // DescriptorBinding::new(
            //     8,
            //     1,
            //     vk::DescriptorType::SAMPLED_IMAGE,
            //     vk::ShaderStageFlags::CLOSEST_HIT_KHR,
            // ),
            // DescriptorBinding::new(
            //     9,
            //     1,
            //     vk::DescriptorType::SAMPLER,
            //     vk::ShaderStageFlags::CLOSEST_HIT_KHR,
            // ),
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

        let tlas_memory_requirements = get_total_memory_requirements(&top_structures);

        let top_structures_buffer = Buffer::empty(
            device,
            tlas_memory_requirements.acceleration_structure_size,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
            gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
        );

        let top_structures_scratch_buffer = Buffer::empty(
            device,
            tlas_memory_requirements.build_scratch_size,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::STORAGE_BUFFER,
            gpu_alloc::UsageFlags::DEVICE_ADDRESS,
        );

        // blas
        let blas_buffer = Buffer::empty(
            device,
            2048,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
            gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
        );

        let blas_scratch_buffer = Buffer::empty(
            device,
            2048,
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
            vk::BufferUsageFlags::VERTEX_BUFFER | buffer_usage_flags,
            gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS | gpu_alloc::UsageFlags::HOST_ACCESS,
        );

        let index_buffer = Buffer::empty(
            device,
            INDEX_BUFFER_SIZE,
            vk::BufferUsageFlags::INDEX_BUFFER | buffer_usage_flags,
            gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS | gpu_alloc::UsageFlags::HOST_ACCESS,
        );

        let offset_buffer = Buffer::empty(
            device,
            VERTEX_BUFFER_SIZE,
            vk::BufferUsageFlags::STORAGE_BUFFER | buffer_usage_flags,
            gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
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

        // let sampler = Sampler::new(device, &SamplerInfo::default());

        //

        Raytracer {
            pipeline,
            pipeline_layout,
            descriptor_set_manager,
            raygen_index,
            miss_index,
            triangle_hit_group_index,
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
            index_buffer,
            offset_buffer,
            uniform_buffers,
            material_buffer,
            // sampler,
        }
    }

    fn update_descriptors(&mut self, device: &mut Device) {
        (0..3).enumerate().for_each(|(i, _)| {
            let top_structures_handle = [self.top_structures[0].handle()];
            let mut top_structures_info =
                vk::WriteDescriptorSetAccelerationStructureKHRBuilder::new()
                    .acceleration_structures(&top_structures_handle);

            let accumulation_image_info = [vk::DescriptorImageInfoBuilder::new()
                .image_view(self.accumulation_image.view())
                .image_layout(vk::ImageLayout::GENERAL)];

            let output_image_info = [vk::DescriptorImageInfoBuilder::new()
                .image_view(self.output_image.view())
                .image_layout(vk::ImageLayout::GENERAL)];

            let uniform_buffer_info = [vk::DescriptorBufferInfoBuilder::new()
                .buffer(self.uniform_buffers[i].buffer().handle())
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

            // let image_infos: Vec<_> = self
            //     .textures
            //     .iter()
            //     .map(|texture_image| {
            //         vk::DescriptorImageInfoBuilder::new()
            //             .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            //             .image_view(texture_image.image_view())
            //     })
            //     .collect();

            // let sampler_info =
            //     [vk::DescriptorImageInfoBuilder::new().sampler(self.sampler.handle())];

            let descriptor_writes = [
                self.descriptor_set_manager.bind_acceleration_structure(
                    i as u32,
                    0,
                    &mut top_structures_info,
                ),
                self.descriptor_set_manager
                    .bind_image(i as u32, 1, &accumulation_image_info),
                self.descriptor_set_manager
                    .bind_image(i as u32, 2, &output_image_info),
                self.descriptor_set_manager
                    .bind_buffer(i as u32, 3, &uniform_buffer_info),
                self.descriptor_set_manager
                    .bind_buffer(i as u32, 4, &vertex_buffer_info),
                self.descriptor_set_manager
                    .bind_buffer(i as u32, 5, &index_buffer_info),
                self.descriptor_set_manager
                    .bind_buffer(i as u32, 6, &material_buffer_info),
                self.descriptor_set_manager
                    .bind_buffer(i as u32, 7, &offset_buffer_info),
                // self.descriptor_set_manager
                //     .bind_image(i as u32, 8, &image_infos),
                // self.descriptor_set_manager
                //     .bind_image(i as u32, 9, &sampler_info),
            ];

            self.descriptor_set_manager
                .update_descriptors(device, &descriptor_writes);
        });
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

    fn update(&mut self, device: &mut Device, ui: &mut UserInterface) {
        puffin::profile_function!();
        let vertices = vec![
            ModelVertex::new(vec3(0.0, -0.5, 0.0), vec2(0.0, 0.0)).as_std430(),
            ModelVertex::new(vec3(0.5, 0.5, 0.0), vec2(0.0, 0.0)).as_std430(),
            ModelVertex::new(vec3(-0.5, 0.5, 0.0), vec2(0.0, 0.0)).as_std430(),
        ];
        let indices = vec![0, 1, 2];
        let offsets = vec![(0u32, 0)];

        //
        self.vertex_buffer.write_data(device, &vertices, 0);
        self.index_buffer.write_data(device, &indices, 0);

        //
        let mut geometries = BottomLevelGeometry::default();
        geometries.add_geometry_triangles(
            device,
            &self.vertex_buffer,
            &self.index_buffer,
            0,
            3,
            3,
            0,
            true,
        );

        self.bottom_structures = vec![BottomLevelAccelerationStructure::new(
            device,
            self.raytracing_properties,
            geometries,
        )];

        CommandPool::single_time_submit(device, |command_buffer| {
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
        });

        self.update_descriptors(device);
    }

    fn destroy(&mut self, device: &mut Device) {
        todo!()
    }
}
