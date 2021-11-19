use crate::vulkan::device::Device;
use crate::vulkan::device_memory::DeviceMemory;
use erupt::vk;
use std::rc::Rc;
use winit::event::VirtualKeyCode::S;

pub struct Buffer {
    device: Rc<Device>,
    handle: vk::Buffer,
    device_memory: Option<DeviceMemory>,
}

impl Buffer {
    pub fn handle(&self) -> vk::Buffer {
        self.handle
    }

    pub fn new(device: Rc<Device>, size: u64, usage: vk::BufferUsageFlags) -> Self {
        let create_info = vk::BufferCreateInfoBuilder::new()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { device.create_buffer(&create_info, None).unwrap() };

        Buffer {
            device,
            handle: buffer,
            device_memory: None,
        }
    }

    pub fn allocate_memory(&mut self) {
        assert!(self.device_memory.is_none());

        let mem_reqs = self.get_memory_requirements();
        let device_memory = DeviceMemory::new(self.device.clone(), mem_reqs);
        device_memory.bind_to_buffer(self.handle);

        self.device_memory = Some(device_memory);
    }

    fn get_memory_requirements(&self) -> vk::MemoryRequirements {
        unsafe { self.device.get_buffer_memory_requirements(self.handle) }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(Some(self.handle), None);
        }
    }
}
