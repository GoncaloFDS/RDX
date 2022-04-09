use crate::vulkan::device::Device;
use crate::vulkan::raytracing::raytracing_properties::RaytracingProperties;
use erupt::vk;

pub trait AccelerationStructure {
    fn build_sizes(&self) -> vk::AccelerationStructureBuildSizesInfoKHR;
}

pub fn get_acceleration_structure_build_sizes(
    device: &Device,
    build_geometry_info: &vk::AccelerationStructureBuildGeometryInfoKHR,
    max_primitive_counts: &[u32],
    raytracing_properties: &RaytracingProperties,
) -> vk::AccelerationStructureBuildSizesInfoKHR {
    let mut size_info = unsafe {
        device.handle().get_acceleration_structure_build_sizes_khr(
            vk::AccelerationStructureBuildTypeKHR::DEVICE_KHR,
            build_geometry_info,
            max_primitive_counts,
        )
    };

    let acceleration_structure_alignment = 256;
    let scratch_alignment =
        raytracing_properties.min_acceleration_structure_scratch_offset_alignment() as u64;

    size_info.acceleration_structure_size = round_up(
        size_info.acceleration_structure_size,
        acceleration_structure_alignment,
    );
    size_info.build_scratch_size = round_up(size_info.build_scratch_size, scratch_alignment);

    size_info
}

pub fn round_up(size: u64, granularity: u64) -> u64 {
    ((size + granularity - 1) / granularity) * granularity
}

pub fn get_total_memory_requirements<T: AccelerationStructure>(
    acceleration_structures: &[T],
) -> vk::AccelerationStructureBuildSizesInfoKHRBuilder {
    let (acceleration_structure_size, build_scratch_size, update_scratch_size) =
        acceleration_structures.iter().fold(
            (0, 0, 0),
            |(acceleration_structure_size, build_scratch_size, update_scratch_size),
             acceleration_structure| {
                (
                    acceleration_structure_size
                        + acceleration_structure
                            .build_sizes()
                            .acceleration_structure_size,
                    build_scratch_size + acceleration_structure.build_sizes().build_scratch_size,
                    update_scratch_size + acceleration_structure.build_sizes().update_scratch_size,
                )
            },
        );

    vk::AccelerationStructureBuildSizesInfoKHRBuilder::new()
        .acceleration_structure_size(acceleration_structure_size)
        .update_scratch_size(update_scratch_size)
        .build_scratch_size(build_scratch_size)
}
