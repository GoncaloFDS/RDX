use crate::vulkan::buffer::Buffer;
use crate::vulkan::device::Device;
use bytemuck::{cast_slice, Pod};
use erupt::vk;
use erupt::vk::DescriptorSet;

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
        descriptor_sets: &[DescriptorSet],
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
}
