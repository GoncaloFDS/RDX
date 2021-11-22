use crate::vulkan::buffer::Buffer;
use crate::vulkan::device::Device;
use erupt::vk;
use log::debug;

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

    pub fn bind_vertex_buffer(&self, device: &Device, vertex_buffers: &[Buffer]) {
        let vertex_buffers = vertex_buffers
            .iter()
            .map(|buffer| buffer.handle())
            .collect::<Vec<_>>();
        unsafe {
            device.cmd_bind_vertex_buffers(self.handle, 0, &vertex_buffers, &[0]);
        }
    }

    pub fn bind_index_buffer(&self, device: &Device, index_buffer: &Buffer) {
        unsafe {
            device.cmd_bind_index_buffer(
                self.handle,
                index_buffer.handle(),
                0,
                vk::IndexType::UINT32,
            )
        }
    }

    pub fn draw_indexed(&self, device: &Device) {
        unsafe {
            device.cmd_draw_indexed(self.handle, 3, 1, 0, 0, 0);
        }
    }
}
