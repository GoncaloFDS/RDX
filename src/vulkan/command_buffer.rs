use crate::vulkan::buffer::Buffer;
use crate::vulkan::device::Device;
use bytemuck::{cast_slice, Pod};
use erupt::vk;

#[derive(Copy, Clone)]
pub struct CommandBuffer {
    handle: vk::CommandBuffer,
}

impl CommandBuffer {
    pub fn new(handle: vk::CommandBuffer) -> Self {
        Self { handle }
    }

    pub fn handle(&self) -> vk::CommandBuffer {
        self.handle
    }

    pub fn begin(&self, device: &Device, usage: vk::CommandBufferUsageFlags) {
        let begin_info = vk::CommandBufferBeginInfoBuilder::new().flags(usage);

        unsafe {
            device
                .handle()
                .begin_command_buffer(self.handle, &begin_info)
                .unwrap()
        }
    }

    pub fn end(&self, device: &Device) {
        unsafe { device.handle().end_command_buffer(self.handle).unwrap() }
    }

    pub fn begin_rendering(&self, device: &Device, rendering_info: &vk::RenderingInfo) {
        unsafe {
            device
                .handle()
                .cmd_begin_rendering(self.handle, rendering_info)
        }
    }

    pub fn end_rendering(&self, device: &Device) {
        unsafe { device.handle().cmd_end_rendering(self.handle) }
    }

    pub fn bind_pipeline(
        &self,
        device: &Device,
        bind_point: vk::PipelineBindPoint,
        pipeline: vk::Pipeline,
    ) {
        unsafe {
            device
                .handle()
                .cmd_bind_pipeline(self.handle, bind_point, pipeline);
        }
    }

    pub fn bind_vertex_buffer(&self, device: &Device, vertex_buffers: &[&Buffer], offsets: &[u64]) {
        let vertex_buffers = vertex_buffers
            .iter()
            .map(|buffer| buffer.handle())
            .collect::<Vec<_>>();
        unsafe {
            device
                .handle()
                .cmd_bind_vertex_buffers(self.handle, 0, &vertex_buffers, offsets);
        }
    }

    pub fn bind_index_buffer(&self, device: &Device, index_buffer: &Buffer, offset: u64) {
        unsafe {
            device.handle().cmd_bind_index_buffer(
                self.handle,
                index_buffer.handle(),
                offset,
                vk::IndexType::UINT32,
            )
        }
    }

    pub fn bind_descriptor_sets(
        &self,
        device: &Device,
        pipeline_bind_point: vk::PipelineBindPoint,
        pipeline_layout: vk::PipelineLayout,
        descriptor_sets: &[vk::DescriptorSet],
    ) {
        unsafe {
            device.handle().cmd_bind_descriptor_sets(
                self.handle,
                pipeline_bind_point,
                pipeline_layout,
                0,
                descriptor_sets,
                &[],
            );
        }
    }

    pub fn set_scissor(&self, device: &Device, first_scissor: u32, scissors: &[vk::Rect2DBuilder]) {
        unsafe {
            device
                .handle()
                .cmd_set_scissor(self.handle, first_scissor, scissors)
        }
    }

    pub fn set_viewport(
        &self,
        device: &Device,
        first_viewport: u32,
        viewports: &[vk::ViewportBuilder],
    ) {
        unsafe {
            device
                .handle()
                .cmd_set_viewport(self.handle, first_viewport, viewports)
        }
    }

    pub fn image_memory_barrier(
        &self,
        device: &Device,
        image: vk::Image,
        subresource: vk::ImageSubresourceRange,
        src_stage_mask: vk::PipelineStageFlags2,
        dst_stage_mask: vk::PipelineStageFlags2,
        src_access_mask: vk::AccessFlags2,
        dst_access_mask: vk::AccessFlags2,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        unsafe {
            device.handle().cmd_pipeline_barrier2(
                self.handle,
                &vk::DependencyInfoBuilder::new().image_memory_barriers(&[
                    vk::ImageMemoryBarrier2Builder::new()
                        .src_stage_mask(src_stage_mask)
                        .dst_stage_mask(dst_stage_mask)
                        .src_access_mask(src_access_mask)
                        .dst_access_mask(dst_access_mask)
                        .old_layout(old_layout)
                        .new_layout(new_layout)
                        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                        .image(image)
                        .subresource_range(subresource),
                ]),
            );
        }
    }

    pub fn draw(
        &self,
        device: &Device,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            device.handle().cmd_draw(
                self.handle,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
    }

    pub fn draw_indexed(&self, device: &Device, index_count: u32) {
        unsafe {
            device
                .handle()
                .cmd_draw_indexed(self.handle, index_count, 1, 0, 0, 0);
        }
    }

    pub fn push_constants<T: Pod>(
        &self,
        device: &Device,
        layout: vk::PipelineLayout,
        stages: vk::ShaderStageFlags,
        offset: u32,
        push_constants: &T,
    ) {
        let slice = [*push_constants];
        let data: &[u8] = cast_slice(&slice);
        unsafe {
            device.handle().cmd_push_constants(
                self.handle,
                layout,
                stages,
                offset,
                data.len() as u32,
                data.as_ptr() as *const _,
            )
        };
    }

    pub fn build_acceleration_structure(
        &self,
        device: &Device,
        build_info: &[vk::AccelerationStructureBuildGeometryInfoKHRBuilder],
        build_offsets: &[*const vk::AccelerationStructureBuildRangeInfoKHR],
    ) {
        unsafe {
            device.handle().cmd_build_acceleration_structures_khr(
                self.handle,
                build_info,
                build_offsets,
            )
        }
    }

    pub fn acceleration_structure_memory_barrier(&self, device: &Device) {
        let memory_barrier = vk::MemoryBarrierBuilder::new()
            .src_access_mask(
                vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR
                    | vk::AccessFlags::ACCELERATION_STRUCTURE_READ_KHR,
            )
            .dst_access_mask(
                vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR
                    | vk::AccessFlags::ACCELERATION_STRUCTURE_READ_KHR,
            );

        unsafe {
            device.handle().cmd_pipeline_barrier(
                self.handle,
                vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
                vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
                vk::DependencyFlags::default(),
                &[memory_barrier],
                &[],
                &[],
            )
        }
    }

    pub fn trace_rays(
        &self,
        device: &Device,
        raygen: &vk::StridedDeviceAddressRegionKHR,
        miss: &vk::StridedDeviceAddressRegionKHR,
        hit: &vk::StridedDeviceAddressRegionKHR,
        callable: &vk::StridedDeviceAddressRegionKHR,
        extent: vk::Extent2D,
    ) {
        unsafe {
            device.handle().cmd_trace_rays_khr(
                self.handle,
                raygen,
                miss,
                hit,
                callable,
                extent.width,
                extent.height,
                1,
            )
        }
    }

    pub fn copy_image(
        &self,
        device: &Device,
        src_image: vk::Image,
        src_layout: vk::ImageLayout,
        dst_image: vk::Image,
        dst_layout: vk::ImageLayout,
        copy_region: vk::ImageCopyBuilder,
    ) {
        unsafe {
            device.handle().cmd_copy_image(
                self.handle,
                src_image,
                src_layout,
                dst_image,
                dst_layout,
                &[copy_region],
            );
        }
    }

    pub fn copy_buffer(
        &self,
        device: &Device,
        src_buffer: &Buffer,
        dst_buffer: &Buffer,
        size: vk::DeviceSize,
    ) {
        let region = vk::BufferCopyBuilder::new()
            .size(size)
            .src_offset(0)
            .dst_offset(0);

        unsafe {
            device.handle().cmd_copy_buffer(
                self.handle,
                src_buffer.handle(),
                dst_buffer.handle(),
                &[region],
            );
        }
    }
}
