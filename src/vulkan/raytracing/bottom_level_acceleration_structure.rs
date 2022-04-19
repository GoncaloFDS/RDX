use crate::vulkan::buffer::Buffer;
use crate::vulkan::device::Device;
use crate::vulkan::raytracing::acceleration_structure;
use crate::vulkan::raytracing::acceleration_structure::AccelerationStructure;
use crate::vulkan::raytracing::bottom_level_geometry::BottomLevelGeometry;
use crate::vulkan::raytracing::raytracing_properties::RaytracingProperties;
use erupt::vk;

pub struct BottomLevelAccelerationStructure {
    handle: vk::AccelerationStructureKHR,
    geometries: BottomLevelGeometry,
    build_sizes_info: vk::AccelerationStructureBuildSizesInfoKHR,
}

impl AccelerationStructure for BottomLevelAccelerationStructure {
    fn build_sizes(&self) -> vk::AccelerationStructureBuildSizesInfoKHR {
        self.build_sizes_info
    }
}

impl BottomLevelAccelerationStructure {
    pub fn new(
        device: &Device,
        raytracing_properties: RaytracingProperties,
        geometries: BottomLevelGeometry,
    ) -> Self {
        let build_geometry_info = vk::AccelerationStructureBuildGeometryInfoKHRBuilder::new()
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE_KHR)
            .geometries(geometries.geometry())
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD_KHR)
            ._type(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL_KHR)
            .src_acceleration_structure(vk::AccelerationStructureKHR::null());

        let max_primitive_counts: Vec<_> = geometries
            .build_offset_info()
            .iter()
            .map(|info| info.primitive_count)
            .collect();

        let build_sizes_info = acceleration_structure::get_acceleration_structure_build_sizes(
            device,
            &build_geometry_info,
            &max_primitive_counts,
            &raytracing_properties,
        );

        BottomLevelAccelerationStructure {
            handle: vk::AccelerationStructureKHR::null(),
            geometries,
            build_sizes_info,
        }
    }

    fn destroy(&self, device: &Device) {
        unsafe {
            device
                .handle()
                .destroy_acceleration_structure_khr(self.handle, None);
        }
    }

    pub fn handle(&self) -> vk::AccelerationStructureKHR {
        self.handle
    }

    pub fn generate(&mut self, device: &Device, blas_buffer: &Buffer, buffer_offset: u64) {
        if !self.handle.is_null() {
            return;
        }

        let create_info = vk::AccelerationStructureCreateInfoKHRBuilder::new()
            ._type(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL_KHR)
            .size(self.build_sizes_info.acceleration_structure_size)
            .buffer(blas_buffer.handle())
            .offset(buffer_offset);

        self.handle = unsafe {
            device
                .handle()
                .create_acceleration_structure_khr(&create_info, None)
                .unwrap()
        };
    }

    pub fn get_build_info(
        &self,
        scratch_buffer_address: vk::DeviceAddress,
        scratch_offset: u64,
    ) -> (
        vk::AccelerationStructureBuildGeometryInfoKHRBuilder,
        &[vk::AccelerationStructureBuildRangeInfoKHR],
    ) {
        let build_geometry_info = vk::AccelerationStructureBuildGeometryInfoKHRBuilder::new()
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE_KHR)
            .geometries(self.geometries.geometry())
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD_KHR)
            ._type(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL_KHR)
            .src_acceleration_structure(vk::AccelerationStructureKHR::null())
            .dst_acceleration_structure(self.handle)
            .scratch_data(vk::DeviceOrHostAddressKHR {
                device_address: scratch_buffer_address + scratch_offset,
            });
        let build_offsets = self.geometries.build_offset_info();
        (build_geometry_info, build_offsets)
    }

    pub fn get_address(&self, device: &Device) -> u64 {
        let address_info = vk::AccelerationStructureDeviceAddressInfoKHRBuilder::new()
            .acceleration_structure(self.handle);

        unsafe {
            device
                .handle()
                .get_acceleration_structure_device_address_khr(&address_info)
        }
    }
}
