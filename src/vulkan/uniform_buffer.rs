use crate::vulkan::buffer::Buffer;
use crate::vulkan::device::Device;
use crevice::std430::{AsStd430, Std430};
use erupt::vk;
use std::mem::size_of;
use std::rc::Rc;
use glam::Mat4;

#[derive(AsStd430)]
pub struct UniformBufferObject {
    pub view_model: Mat4,
    pub projection: Mat4,
    pub view_model_inverse: Mat4,
    pub projection_inverse: Mat4,
}

pub struct UniformBuffer {
    buffer: Buffer,
}

impl UniformBuffer {
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn new(device: Rc<Device>) -> Self {
        let buffer_size = size_of::<Std430UniformBufferObject>();
        let mut buffer = Buffer::new(
            device,
            buffer_size as _,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
        );
        buffer.allocate_memory(gpu_alloc::UsageFlags::HOST_ACCESS);

        UniformBuffer { buffer }
    }

    pub fn update_gpu_buffer(&mut self, ubo: &UniformBufferObject) {
        self.buffer.write_data(ubo.as_std430().as_bytes(), 0);
    }
}
