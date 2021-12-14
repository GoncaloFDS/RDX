use crate::vulkan::descriptor_binding::DescriptorBinding;
use crate::vulkan::descriptor_set_manager::DescriptorSetManager;
use crate::vulkan::device::Device;
use crate::vulkan::pipeline_layout::PipelineLayout;
use crate::vulkan::shader_module::ShaderModule;
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

    pub fn new(device: Rc<Device>, descriptor_set_count: u32) -> Self {
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
                3,
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

        let descriptor_set_manager =
            DescriptorSetManager::new(device.clone(), &descriptor_bindings, descriptor_set_count);

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
            .flags(vk::PipelineCreateFlags::empty())
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

impl Drop for RaytracingPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(Some(self.handle), None);
        }
    }
}
