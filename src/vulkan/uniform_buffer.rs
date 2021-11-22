use crate::vulkan::buffer::Buffer;
use crate::vulkan::device::Device;
use erupt::vk;
use std::mem::size_of;
use std::rc::Rc;

pub struct UniformBufferObject {
    view_model: glam::Mat4,
    projection: glam::Mat4,
}

pub struct UniformBuffer {
    buffer: Buffer,
}

impl UniformBuffer {
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn new(device: Rc<Device>) -> Self {
        let buffer_size = size_of::<UniformBufferObject>();
        let mut buffer = Buffer::new(
            device,
            buffer_size as _,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
        );
        buffer.allocate_memory(gpu_alloc::UsageFlags::empty());

        UniformBuffer { buffer }
    }

    pub fn update_gpu_buffer(ubo: &UniformBufferObject) {
        log::debug!("update_gpu_buffer not implemented")
    }
}
