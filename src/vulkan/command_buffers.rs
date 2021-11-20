use crate::vulkan::device::Device;
use erupt::vk;
use std::ops::Deref;
use std::rc::Rc;

pub struct CommandBuffers {
    device: Rc<Device>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
}

impl CommandBuffers {
    pub fn new(device: Rc<Device>, queue_family_index: u32, reset: bool) -> Self {
        let create_info = vk::CommandPoolCreateInfoBuilder::new()
            .queue_family_index(queue_family_index)
            .flags(if reset {
                vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
            } else {
                vk::CommandPoolCreateFlags::empty()
            });
        let command_pool = unsafe { device.create_command_pool(&create_info, None).unwrap() };

        CommandBuffers {
            device,
            command_pool,
            command_buffers: vec![],
        }
    }

    pub fn allocate(&mut self, count: u32) {
        let alloc_info = vk::CommandBufferAllocateInfoBuilder::new()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);

        let mut command_buffers =
            unsafe { self.device.allocate_command_buffers(&alloc_info).unwrap() };

        self.command_buffers.append(&mut command_buffers);
    }

    pub fn begin(&self, i: usize) -> vk::CommandBuffer {
        let begin_info = vk::CommandBufferBeginInfoBuilder::new()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);
        unsafe {
            self.device
                .begin_command_buffer(self.command_buffers[i], &begin_info)
                .unwrap()
        }

        self.command_buffers[i]
    }

    pub fn end(&self, i: usize) {
        unsafe {
            self.device
                .end_command_buffer(self.command_buffers[i])
                .unwrap();
        }
    }

    pub fn single_time_submit(&self, action: impl Fn(vk::CommandBuffer)) {
        let alloc_info = vk::CommandBufferAllocateInfoBuilder::new()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer =
            unsafe { self.device.allocate_command_buffers(&alloc_info).unwrap()[0] };

        let begin_info = vk::CommandBufferBeginInfoBuilder::new()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &begin_info)
                .unwrap()
        }

        action(command_buffer);

        unsafe { self.device.end_command_buffer(command_buffer).unwrap() }

        let submit_buffers = [command_buffer];
        let submit_info = vk::SubmitInfoBuilder::new().command_buffers(&submit_buffers);

        let graphics_queue = self.device.graphics_queue();

        unsafe {
            self.device
                .queue_submit(graphics_queue, &[submit_info], None)
                .unwrap();
            self.device.device_wait_idle().unwrap();
        }
    }
}

impl Drop for CommandBuffers {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            if !self.command_buffers.is_empty() {
                self.device
                    .free_command_buffers(self.command_pool, &self.command_buffers);
            }
            self.device
                .destroy_command_pool(Some(self.command_pool), None);
        }
    }
}
