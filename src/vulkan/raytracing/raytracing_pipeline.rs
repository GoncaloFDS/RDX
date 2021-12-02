use crate::vulkan::buffer::Buffer;
use crate::vulkan::descriptor_binding::DescriptorBinding;
use crate::vulkan::descriptor_set_manager::DescriptorSetManager;
use crate::vulkan::device::Device;
use crate::vulkan::image_view::ImageView;
use crate::vulkan::pipeline_layout::PipelineLayout;
use crate::vulkan::raytracing::top_level_acceleration_structure::TopLevelAccelerationStructure;
use crate::vulkan::shader_module::ShaderModule;
use crate::vulkan::swapchain::Swapchain;
use crate::vulkan::uniform_buffer::UniformBuffer;
use erupt::vk;
use std::rc::Rc;

pub struct RaytracingPipeline {
    handle: vk::Pipeline,
    device: Rc<Device>,
    descriptor_set_manager: Option<DescriptorSetManager>,
    pipeline_layout: PipelineLayout,

    raygen_index: u32,
    miss_index: u32,
    triangle_hit_group_index: u32,
}

impl RaytracingPipeline {
    pub fn handle(&self) -> vk::Pipeline {
        self.handle
    }

    pub fn pipeline_layout(&self) -> &PipelineLayout {
        &self.pipeline_layout
    }

    pub fn descriptor_set_manager(&self) -> &DescriptorSetManager {
        self.descriptor_set_manager.as_ref().unwrap()
    }

    pub fn raygen_index(&self) -> u32 {
        self.raygen_index
    }

    pub fn miss_index(&self) -> u32 {
        self.miss_index
    }

    pub fn triangle_hit_group_index(&self) -> u32 {
        self.triangle_hit_group_index
    }

    pub fn uninitialized(device: Rc<Device>) -> Self {
        let pipeline_layout = PipelineLayout::uninitialized(device.clone());
        RaytracingPipeline {
            handle: Default::default(),
            device,
            descriptor_set_manager: None,
            pipeline_layout,
            raygen_index: 0,
            miss_index: 0,
            triangle_hit_group_index: 0,
        }
    }

    pub fn new(
        device: Rc<Device>,
        swapchain: &Swapchain,
        tlas: &TopLevelAccelerationStructure,
        accumulation_view: &ImageView,
        output_view: &ImageView,
        uniform_buffers: &[UniformBuffer],
        vertex_buffer: &Buffer,
        index_buffer: &Buffer,
    ) -> Self {
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
            // Vertex, Index, Material, Offset
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
            // DescriptorBinding::new(
            //     7,
            //     1,
            //     vk::DescriptorType::STORAGE_BUFFER,
            //     vk::ShaderStageFlags::CLOSEST_HIT_KHR,
            // ),
            // Textures
            // DescriptorBinding::new(
            //     8,
            //     texture_image_views.len() as u32,
            //     vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            //     vk::ShaderStageFlags::CLOSEST_HIT_KHR,
            // ),
        ];

        let descriptor_set_manager =
            DescriptorSetManager::new(device.clone(), &descriptor_bindings, uniform_buffers.len());

        // extract this
        swapchain.images().iter().enumerate().for_each(|(i, _)| {
            let tlas_handle = [tlas.handle()];
            let tlas_info = vk::WriteDescriptorSetAccelerationStructureKHRBuilder::new()
                .acceleration_structures(&tlas_handle);

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

            // let material_buffer_info = [vk::DescriptorBufferInfoBuilder::new()
            //     .buffer(material_buffer.handle())
            //     .range(vk::WHOLE_SIZE)];

            // let image_infos: Vec<_> = texture_image_views
            //     .iter()
            //     .zip(texture_samplers)
            //     .map(|(image_view, sampler)| {
            //         vk::DescriptorImageInfoBuilder::new()
            //             .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            //             .image_view(*image_view)
            //             .sampler(*sampler)
            //     })
            //     .collect();

            let descriptor_writes = [
                descriptor_set_manager.bind_acceleration_structure(i as u32, 0, &tlas_info),
                descriptor_set_manager.bind_image(i as u32, 1, &accumulation_image_info),
                descriptor_set_manager.bind_image(i as u32, 2, &output_image_info),
                descriptor_set_manager.bind_buffer(i as u32, 3, &uniform_buffer_info),
                descriptor_set_manager.bind_buffer(i as u32, 4, &vertex_buffer_info),
                descriptor_set_manager.bind_buffer(i as u32, 5, &index_buffer_info),
                // descriptor_set_manager.bind_buffer(i as u32, 6, &material_buffer_info),
                // descriptor_set_manager.bind_image(i as u32, 7, &image_infos),
            ];

            descriptor_set_manager.update_descriptors(&descriptor_writes);
        });
        // extract this

        let pipeline_layout = PipelineLayout::new(
            device.clone(),
            &[descriptor_set_manager.descriptor_set_layout()],
            &[],
        );

        let raytracing_shader = ShaderModule::new(device.clone(), "raytracing");

        let shader_stages = [
            raytracing_shader.create_shader_stage(vk::ShaderStageFlagBits::RAYGEN_KHR, "raygen\0"),
            raytracing_shader.create_shader_stage(vk::ShaderStageFlagBits::MISS_KHR, "miss\0"),
            raytracing_shader
                .create_shader_stage(vk::ShaderStageFlagBits::CLOSEST_HIT_KHR, "closest_hit\0"),
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
            .stages(&shader_stages)
            .groups(&shader_groups)
            .max_pipeline_ray_recursion_depth(1)
            .layout(pipeline_layout.handle())
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(0);

        let raytracing_pipeline = unsafe {
            device
                .create_ray_tracing_pipelines_khr(None, None, &[pipeline_info], None)
                .unwrap()[0]
        };

        RaytracingPipeline {
            handle: raytracing_pipeline,
            device,
            descriptor_set_manager: Some(descriptor_set_manager),
            pipeline_layout,
            raygen_index,
            miss_index,
            triangle_hit_group_index,
        }
    }
}
