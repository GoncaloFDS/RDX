use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::semaphore::Semaphore;

pub struct Frame {
    pub command_buffer: CommandBuffer,
    pub complete: Semaphore,
}

impl Frame {
    pub fn new(cmd: CommandBuffer, complete: Semaphore) -> Self {
        Self {
            command_buffer: cmd,
            complete,
        }
    }
}
