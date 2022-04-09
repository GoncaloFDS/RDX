use crate::vulkan::device::Device;
use crate::vulkan::instance::Instance;
use erupt::{vk, ExtendableFrom};

#[derive(Copy, Clone)]
pub struct RaytracingProperties {
    acceleration_properties: vk::PhysicalDeviceAccelerationStructurePropertiesKHR,
    pipeline_properties: vk::PhysicalDeviceRayTracingPipelinePropertiesKHR,
}

impl RaytracingProperties {
    pub fn new(device: &Device, instance: &Instance) -> Self {
        let mut acceleration_properties =
            vk::PhysicalDeviceAccelerationStructurePropertiesKHR::default();
        let mut pipeline_properties = vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::default();
        let mut properties = vk::PhysicalDeviceProperties2Builder::default()
            .extend_from(&mut acceleration_properties)
            .extend_from(&mut pipeline_properties);

        *properties = unsafe {
            instance.handle().get_physical_device_properties2(
                device.metadata().physical_device(),
                Some(*properties),
            )
        };

        RaytracingProperties {
            acceleration_properties,
            pipeline_properties,
        }
    }
    pub fn min_acceleration_structure_scratch_offset_alignment(&self) -> u32 {
        self.acceleration_properties
            .min_acceleration_structure_scratch_offset_alignment
    }

    pub fn shader_group_handle_size(&self) -> u32 {
        self.pipeline_properties.shader_group_handle_size
    }

    pub fn shader_group_base_alignment(&self) -> u32 {
        self.pipeline_properties.shader_group_base_alignment
    }
}
