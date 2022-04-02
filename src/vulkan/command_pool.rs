use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;
use erupt::{vk, SmallVec};

pub struct CommandPool {
    handle: vk::CommandPool,
}

impl CommandPool {
    pub fn new(device: &Device, queue_family_index: u32, reset: bool) -> Self {
        let create_info = vk::CommandPoolCreateInfoBuilder::new()
            .queue_family_index(queue_family_index)
            .flags(if reset {
                vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
            } else {
                vk::CommandPoolCreateFlags::empty()
            });
        let command_pool = unsafe {
            device
                .handle()
                .create_command_pool(&create_info, None)
                .unwrap()
        };

        Self {
            handle: command_pool,
        }
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.handle().destroy_command_pool(self.handle, None);
        }
    }

    pub fn allocate(&self, device: &Device, count: u32) -> Vec<CommandBuffer> {
        let alloc_info = vk::CommandBufferAllocateInfoBuilder::new()
            .command_pool(self.handle)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);

        let command_buffers = unsafe {
            device
                .handle()
                .allocate_command_buffers(&alloc_info)
                .unwrap()
        };

        command_buffers
            .iter()
            .map(|cb| CommandBuffer::new(*cb))
            .collect::<Vec<_>>()
    }

    pub fn free_command_buffers(&self, device: &Device, command_buffers: &[CommandBuffer]) {
        let command_buffer_handles = command_buffers
            .iter()
            .map(|cb| cb.handle())
            .collect::<SmallVec<_>>();
        unsafe {
            device
                .handle()
                .free_command_buffers(self.handle, &command_buffer_handles);
        }
    }
}
