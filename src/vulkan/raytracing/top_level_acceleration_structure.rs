use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::raytracing::acceleration_structure;
use crate::vulkan::raytracing::acceleration_structure::AccelerationStructure;
use crate::vulkan::raytracing::raytracing_properties::RaytracingProperties;
use erupt::vk;
use glam::{u32, Mat4};

pub struct TopLevelAccelerationStructure {
    handle: vk::AccelerationStructureKHR,
    max_instance_count: u32,
    top_as_geometry: vk::AccelerationStructureGeometryKHRBuilder<'static>,
    build_sizes_info: vk::AccelerationStructureBuildSizesInfoKHR,
}

impl AccelerationStructure for TopLevelAccelerationStructure {
    fn build_sizes(&self) -> vk::AccelerationStructureBuildSizesInfoKHR {
        self.build_sizes_info
    }
}

impl TopLevelAccelerationStructure {
    pub fn handle(&self) -> vk::AccelerationStructureKHR {
        self.handle
    }

    pub fn new(
        device: &Device,
        raytracing_properties: RaytracingProperties,
        instance_address: vk::DeviceAddress,
        max_instance_count: u32,
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
            &[max_instance_count],
            &raytracing_properties,
        );

        TopLevelAccelerationStructure {
            handle: Default::default(),
            max_instance_count,
            top_as_geometry,
            build_sizes_info,
        }
    }

    pub fn generate(
        &mut self,
        device: &Device,
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
            device
                .handle()
                .create_acceleration_structure_khr(&create_info, None)
                .unwrap()
        };

        let build_offset_info = vk::AccelerationStructureBuildRangeInfoKHRBuilder::new()
            .primitive_count(self.max_instance_count)
            .build();

        let build_offset_info = &build_offset_info as *const _;

        build_geometry_info.dst_acceleration_structure = self.handle;
        build_geometry_info.scratch_data.device_address =
            scratch_buffer.get_device_address(device) + scratch_offset;

        command_buffer.build_acceleration_structure(
            device,
            &[build_geometry_info],
            &[build_offset_info],
        )
    }

    fn destroy(&self, device: &Device) {
        unsafe {
            log::debug!("dropping tlas");
            device
                .handle()
                .destroy_acceleration_structure_khr(self.handle, None);
        }
    }
}

pub struct AccelerationInstance {
    id: u32,
    blas_id: u32,
    hit_group: u32,
    transform: Mat4,
}

impl AccelerationInstance {
    pub fn new(id: u32, blas_id: u32, hit_group: u32, transform: Mat4) -> Self {
        AccelerationInstance {
            id,
            blas_id,
            hit_group,
            transform,
        }
    }

    pub fn generate(&self, blas_address: u64) -> vk::AccelerationStructureInstanceKHR {
        *vk::AccelerationStructureInstanceKHRBuilder::new()
            .instance_custom_index(self.id)
            .mask(0xFF)
            .instance_shader_binding_table_record_offset(self.hit_group)
            .flags(vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE_KHR)
            .acceleration_structure_reference(blas_address)
            .transform(*vk::TransformMatrixKHRBuilder::new().matrix([
                self.transform.row(0).to_array(),
                self.transform.row(1).to_array(),
                self.transform.row(2).to_array(),
            ]))
    }
}

impl AccelerationInstance {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn blas_id(&self) -> u32 {
        self.blas_id
    }

    pub fn transform(&self) -> Mat4 {
        self.transform
    }
}
