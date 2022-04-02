use crate::vulkan::device::Device;
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

    pub fn begin(&self, device: &Device) {
        let begin_info = vk::CommandBufferBeginInfoBuilder::new()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

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
}
