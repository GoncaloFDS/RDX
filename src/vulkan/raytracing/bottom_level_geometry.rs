use crate::vulkan::buffer::Buffer;
use crate::vulkan::vertex::Std430ModelVertex;
use erupt::vk;
use std::mem::size_of;

#[derive(Default)]
pub struct BottomLevelGeometry {
    geometry: Vec<vk::AccelerationStructureGeometryKHRBuilder<'static>>,
    build_offset_info: Vec<vk::AccelerationStructureBuildRangeInfoKHR>,
}

impl BottomLevelGeometry {
    pub fn geometry(&self) -> &[vk::AccelerationStructureGeometryKHRBuilder] {
        &self.geometry
    }

    pub fn build_offset_info(&self) -> &[vk::AccelerationStructureBuildRangeInfoKHR] {
        &self.build_offset_info
    }

    pub fn count(&self) -> usize {
        self.geometry.len()
    }

    pub fn add_geometry_triangles(
        &mut self,
        vertex_buffer: &Buffer,
        index_buffer: &Buffer,
        vertex_offset: u32,
        vertex_count: u32,
        index_offset: u32,
        index_count: u32,
        is_opaque: bool,
    ) {
        let triangles = vk::AccelerationStructureGeometryDataKHR {
            triangles: *vk::AccelerationStructureGeometryTrianglesDataKHRBuilder::new()
                .vertex_data(vk::DeviceOrHostAddressConstKHR {
                    device_address: vertex_buffer.get_device_address(),
                })
                .vertex_stride(size_of::<Std430ModelVertex>() as _)
                .max_vertex(vertex_count)
                .vertex_format(vk::Format::R32G32B32_SFLOAT)
                .index_data(vk::DeviceOrHostAddressConstKHR {
                    device_address: index_buffer.get_device_address(),
                })
                .index_type(vk::IndexType::UINT32), //.transform_data()
        };

        let geometry = vk::AccelerationStructureGeometryKHRBuilder::new()
            .geometry(triangles)
            .flags(if is_opaque {
                vk::GeometryFlagsKHR::OPAQUE_KHR
            } else {
                vk::GeometryFlagsKHR::empty()
            });

        let build_offset_info = vk::AccelerationStructureBuildRangeInfoKHRBuilder::new()
            .first_vertex(vertex_offset / size_of::<Std430ModelVertex>() as u32)
            .primitive_offset(index_offset)
            .primitive_count(index_count / 3)
            .transform_offset(0);

        self.geometry.push(geometry);
        self.build_offset_info.push(*build_offset_info)
    }
}
