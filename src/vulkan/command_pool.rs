use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;
use erupt::vk;
use std::rc::Rc;

pub struct CommandPool {
    device: Rc<Device>,
    handle: vk::CommandPool,
    command_buffers: Vec<CommandBuffer>,
}

impl CommandPool {
    pub fn new(device: Rc<Device>, queue_family_index: u32, reset: bool) -> Self {
        let create_info = vk::CommandPoolCreateInfoBuilder::new()
            .queue_family_index(queue_family_index)
            .flags(if reset {
                vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
            } else {
                vk::CommandPoolCreateFlags::empty()
            });
        let command_pool = unsafe { device.create_command_pool(&create_info, None).unwrap() };

        CommandPool {
            device,
            handle: command_pool,
            command_buffers: vec![],
        }
    }

    pub fn allocate(&mut self, count: u32) {
        let alloc_info = vk::CommandBufferAllocateInfoBuilder::new()
            .command_pool(self.handle)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);

        let command_buffers = unsafe { self.device.allocate_command_buffers(&alloc_info).unwrap() };

        self.command_buffers = command_buffers
            .iter()
            .map(|cb| CommandBuffer::new(*cb))
            .collect::<Vec<_>>();
    }

    pub fn reset(&mut self) {
        self.command_buffers.clear();
    }

    pub fn begin(&self, i: usize) -> CommandBuffer {
        let command_buffer = self.command_buffers[i];
        command_buffer.begin(&self.device);
        command_buffer
    }

    pub fn single_time_submit(
        device: &Device,
        command_pool: &CommandPool,
        action: impl FnOnce(CommandBuffer),
    ) {
        let alloc_info = vk::CommandBufferAllocateInfoBuilder::new()
            .command_pool(command_pool.handle)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = unsafe { device.allocate_command_buffers(&alloc_info).unwrap()[0] };

        let command_buffer = CommandBuffer::new(command_buffer);

        command_buffer.begin_one_time_submit(device);

        action(command_buffer);

        command_buffer.end(device);

        let submit_buffers = [command_buffer.handle()];
        let submit_info = vk::SubmitInfoBuilder::new().command_buffers(&submit_buffers);

        let graphics_queue = device.graphics_queue();

        unsafe {
            device
                .queue_submit(graphics_queue, &[submit_info], None)
                .unwrap();
            device.device_wait_idle().unwrap();

            // device.free_command_buffers(command_pool.handle, &[command_buffer.handle()])
        }
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            if !self.command_buffers.is_empty() {
                let command_buffers = self
                    .command_buffers
                    .iter()
                    .map(|cb| cb.handle())
                    .collect::<Vec<_>>();
                self.device
                    .free_command_buffers(self.handle, &command_buffers);
            }
            self.device.destroy_command_pool(Some(self.handle), None);
        }
    }
}
