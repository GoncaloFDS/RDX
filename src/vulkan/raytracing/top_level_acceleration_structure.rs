use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::raytracing::acceleration_structure;
use crate::vulkan::raytracing::acceleration_structure::AccelerationStrutcture;
use crate::vulkan::raytracing::bottom_level_acceleration_structure::BottomLevelAccelerationStructure;
use crate::vulkan::raytracing::raytracing_properties::RaytracingProperties;
use erupt::vk;
use glam::Mat4;
use std::rc::Rc;

pub struct TopLevelAccelerationStructure {
    handle: vk::AccelerationStructureKHR,
    device: Rc<Device>,
    instances: vk::AccelerationStructureGeometryInstancesDataKHR,
    instances_count: u32,
    top_as_geometry: vk::AccelerationStructureGeometryKHRBuilder<'static>,
    build_geometry_info: vk::AccelerationStructureBuildGeometryInfoKHR,
    build_sizes_info: vk::AccelerationStructureBuildSizesInfoKHR,
    raytracing_properties: RaytracingProperties,
}

impl AccelerationStrutcture for TopLevelAccelerationStructure {
    fn build_sizes(&self) -> vk::AccelerationStructureBuildSizesInfoKHR {
        self.build_sizes_info
    }
}

impl TopLevelAccelerationStructure {
    pub fn handle(&self) -> vk::AccelerationStructureKHR {
        self.handle
    }

    pub fn new(
        device: Rc<Device>,
        raytracing_properties: RaytracingProperties,
        instance_address: vk::DeviceAddress,
        instances_count: u32,
    ) -> Self {
        let instances = vk::AccelerationStructureGeometryInstancesDataKHRBuilder::new()
            .array_of_pointers(false)
            .data(vk::DeviceOrHostAddressConstKHR {
                device_address: instance_address,
            });

        let top_as_geometry = vk::AccelerationStructureGeometryKHRBuilder::new()
            .geometry_type(vk::GeometryTypeKHR::INSTANCES_KHR)
            .geometry(vk::AccelerationStructureGeometryDataKHR {
                instances: *instances,
            });

        let geometries = [top_as_geometry];
        let build_geometry_info = vk::AccelerationStructureBuildGeometryInfoKHRBuilder::new()
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE_KHR)
            .geometries(&geometries)
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD_KHR)
            ._type(vk::AccelerationStructureTypeKHR::TOP_LEVEL_KHR)
            .src_acceleration_structure(vk::AccelerationStructureKHR::null());

        let build_sizes_info = acceleration_structure::get_acceleration_structure_build_sizes(
            &device,
            &build_geometry_info,
            &[instances_count],
            &raytracing_properties,
        );

        TopLevelAccelerationStructure {
            handle: Default::default(),
            device,
            instances: instances.build(),
            instances_count,
            top_as_geometry,
            build_geometry_info: build_geometry_info.build(),
            build_sizes_info,
            raytracing_properties,
        }
    }

    pub fn generate(
        &mut self,
        command_buffer: &CommandBuffer,
        scratch_buffer: &Buffer,
        scratch_offset: u64,
        result_buffer: &Buffer,
        result_offset: u64,
        old_tlas: Option<vk::AccelerationStructureKHR>,
    ) {
        let geometries = [self.top_as_geometry];
        let mut build_geometry_info = vk::AccelerationStructureBuildGeometryInfoKHRBuilder::new()
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE_KHR)
            .geometries(&geometries)
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD_KHR)
            ._type(vk::AccelerationStructureTypeKHR::TOP_LEVEL_KHR)
            .src_acceleration_structure(old_tlas.unwrap_or(vk::AccelerationStructureKHR::null()));

        let create_info = vk::AccelerationStructureCreateInfoKHRBuilder::new()
            ._type(build_geometry_info._type)
            .size(self.build_sizes_info.acceleration_structure_size)
            .buffer(result_buffer.handle())
            .offset(result_offset);

        self.handle = unsafe {
            self.device
                .create_acceleration_structure_khr(&create_info, None)
                .unwrap()
        };

        let build_offset_info = vk::AccelerationStructureBuildRangeInfoKHRBuilder::new()
            .primitive_count(self.instances_count)
            .build();

        let build_offset_info = &build_offset_info as *const _;

        build_geometry_info.dst_acceleration_structure = self.handle;
        build_geometry_info.scratch_data.device_address =
            scratch_buffer.get_device_address() + scratch_offset;

        command_buffer.build_acceleration_structure(
            &self.device,
            &[build_geometry_info],
            &[build_offset_info],
        )
    }

    pub fn create_instance(
        device: &Device,
        blas: &BottomLevelAccelerationStructure,
        transform: Mat4,
        instance_id: u32,
        hit_group_id: u32,
    ) -> vk::AccelerationStructureInstanceKHR {
        let address_info = vk::AccelerationStructureDeviceAddressInfoKHRBuilder::new()
            .acceleration_structure(blas.handle());

        let as_address =
            unsafe { device.get_acceleration_structure_device_address_khr(&address_info) };

        *vk::AccelerationStructureInstanceKHRBuilder::new()
            .instance_custom_index(instance_id)
            .mask(0xFF)
            .instance_shader_binding_table_record_offset(hit_group_id)
            .flags(vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE_KHR)
            .acceleration_structure_reference(as_address)
            .transform(*vk::TransformMatrixKHRBuilder::new().matrix([
                transform.row(0).to_array(),
                transform.row(1).to_array(),
                transform.row(2).to_array(),
            ]))
    }
}

impl Drop for TopLevelAccelerationStructure {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_acceleration_structure_khr(Some(self.handle()), None);
        }
    }
}
