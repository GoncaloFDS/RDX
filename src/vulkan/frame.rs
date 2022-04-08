use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::semaphore::Semaphore;

pub struct Frame {
    pub command_buffer: CommandBuffer,
    pub semaphore: Semaphore,
}

impl Frame {
    pub fn new(command_buffer: CommandBuffer, semaphore: Semaphore) -> Self {
        Self {
            command_buffer,
            semaphore,
        }
    }

    pub fn destroy(&self, device: &Device) {
        self.semaphore.destroy(device);
    }
}
