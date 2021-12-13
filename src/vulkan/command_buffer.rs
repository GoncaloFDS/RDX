use crate::vulkan::buffer::Buffer;
use crate::vulkan::device::Device;
use bytemuck::{cast_slice, Pod};
use erupt::vk;
use std::mem::size_of_val;
use std::rc::Rc;

#[derive(Copy, Clone)]
pub struct CommandBuffer {
    handle: vk::CommandBuffer,
}

impl CommandBuffer {
    pub fn handle(&self) -> vk::CommandBuffer {
        self.handle
    }

    pub fn new(handle: vk::CommandBuffer) -> Self {
        CommandBuffer { handle }
    }

    pub fn begin(&self, device: &Device) {
        let begin_info = vk::CommandBufferBeginInfoBuilder::new()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        unsafe {
            device
                .begin_command_buffer(self.handle, &begin_info)
                .unwrap()
        }
    }

    pub fn begin_one_time_submit(&self, device: &Device) {
        let begin_info = vk::CommandBufferBeginInfoBuilder::new()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device
                .begin_command_buffer(self.handle, &begin_info)
                .unwrap()
        }
    }

    pub fn end(&self, device: &Device) {
        unsafe {
            device.end_command_buffer(self.handle).unwrap();
        }
    }

    pub fn begin_render_pass(
        &self,
        device: &Device,
        render_pass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
        extent: vk::Extent2D,
    ) {
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.5, 0.2, 0.2, 0.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let render_pass_info = vk::RenderPassBeginInfoBuilder::new()
            .render_pass(render_pass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            })
            .clear_values(&clear_values);

        unsafe {
            device.cmd_begin_render_pass(
                self.handle,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    pub fn end_render_pass(&self, device: &Device) {
        unsafe {
            device.cmd_end_render_pass(self.handle);
        }
    }

    pub fn bind_pipeline(
        &self,
        device: &Device,
        bind_point: vk::PipelineBindPoint,
        pipeline: vk::Pipeline,
    ) {
        unsafe {
            device.cmd_bind_pipeline(self.handle, bind_point, pipeline);
        }
    }

    pub fn bind_vertex_buffer(&self, device: &Device, vertex_buffers: &[&Buffer], offsets: &[u64]) {
        let vertex_buffers = vertex_buffers
            .iter()
            .map(|buffer| buffer.handle())
            .collect::<Vec<_>>();
        unsafe {
            device.cmd_bind_vertex_buffers(self.handle, 0, &vertex_buffers, offsets);
        }
    }

    pub fn bind_index_buffer(&self, device: &Device, index_buffer: &Buffer, offset: u64) {
        unsafe {
            device.cmd_bind_index_buffer(
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
            device.cmd_bind_descriptor_sets(
                self.handle,
                pipeline_bind_point,
                pipeline_layout,
                0,
                descriptor_sets,
                &[],
            );
        }
    }

    pub fn draw_indexed(&self, device: &Device, index_count: u32) {
        unsafe {
            device.cmd_draw_indexed(self.handle, index_count, 1, 0, 0, 0);
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
            device.cmd_push_constants(
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
            device.cmd_build_acceleration_structures_khr(self.handle, build_info, build_offsets)
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
            device.cmd_pipeline_barrier(
                self.handle,
                vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
                vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
                None,
                &[memory_barrier],
                &[],
                &[],
            )
        }
    }

    pub fn image_memory_barrier(
        &self,
        device: &Device,
        image: vk::Image,
        subresource: vk::ImageSubresourceRange,
        src_access_mask: vk::AccessFlags,
        dst_access_mask: vk::AccessFlags,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        let image_barrier = vk::ImageMemoryBarrierBuilder::new()
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(subresource);

        unsafe {
            device.cmd_pipeline_barrier(
                self.handle,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::PipelineStageFlags::ALL_COMMANDS,
                None,
                &[],
                &[],
                &[image_barrier],
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
            device.cmd_trace_rays_khr(
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
            device.cmd_copy_image(
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
            device.cmd_copy_buffer(
                self.handle,
                src_buffer.handle(),
                dst_buffer.handle(),
                &[region],
            );
        }
    }

    pub fn create_device_local_buffer_with_data<T>(
        &self,
        device: Rc<Device>,
        usage: vk::BufferUsageFlags,
        data: &[T],
    ) -> (Buffer, Buffer) {
        let size = size_of_val(data) as u64;
        let staging_buffer =
            Buffer::with_data(device.clone(), data, vk::BufferUsageFlags::TRANSFER_SRC);
        let mut buffer = Buffer::new(
            device.clone(),
            size,
            vk::BufferUsageFlags::TRANSFER_DST | usage,
            gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
        );

        self.copy_buffer(&device, &staging_buffer, &buffer, size);

        (buffer, staging_buffer)
    }
}
