use crate::vulkan::buffer::Buffer;
use crate::vulkan::device::Device;
use crevice::std430::{AsStd430, Std430};
use erupt::vk;
use glam::Mat4;
use std::mem::size_of;

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
    pub fn new(device: &mut Device) -> Self {
        let buffer = Buffer::empty(
            device,
            size_of::<Std430UniformBufferObject>() as _,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            gpu_alloc::UsageFlags::HOST_ACCESS,
        );

        UniformBuffer { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn update_gpu_buffer(&mut self, device: &Device, ubo: &UniformBufferObject) {
        puffin::profile_function!();
        self.buffer
            .write_data(device, ubo.as_std430().as_bytes(), 0);
    }
}
