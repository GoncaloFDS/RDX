use crate::vulkan::device::Device;
use crate::vulkan::device_memory::DeviceMemory;
use bytemuck::Pod;
use erupt::vk;
use std::mem::{size_of, size_of_val};
use std::rc::Rc;

pub struct Buffer {
    device: Rc<Device>,
    handle: vk::Buffer,
    device_memory: Option<DeviceMemory>,
}

impl Buffer {
    pub fn handle(&self) -> vk::Buffer {
        self.handle
    }

    pub fn uninitialized(device: Rc<Device>) -> Self {
        Buffer {
            device,
            handle: Default::default(),
            device_memory: None,
        }
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

    pub fn with_data<T>(device: Rc<Device>, data: &[T], usage: vk::BufferUsageFlags) -> Self {
        let size = size_of_val(data) as u64;

        let mut buffer = Buffer::new(device, size, usage);
        buffer.allocate_memory(
            gpu_alloc::UsageFlags::HOST_ACCESS | gpu_alloc::UsageFlags::DEVICE_ADDRESS,
        );

        let ptr = buffer.device_memory.as_mut().unwrap().map(0, size as _);

        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr() as *const u8, ptr.as_ptr(), size as _);
        }

        buffer.device_memory.as_mut().unwrap().unmap();

        buffer
    }

    pub fn allocate_memory(&mut self, allocation_flags: gpu_alloc::UsageFlags) {
        assert!(self.device_memory.is_none());

        let mem_reqs = self.get_memory_requirements();
        let device_memory = DeviceMemory::new(self.device.clone(), mem_reqs, allocation_flags);
        device_memory.bind_to_buffer(self.handle);

        self.device_memory = Some(device_memory);
    }

    fn get_memory_requirements(&self) -> vk::MemoryRequirements {
        unsafe { self.device.get_buffer_memory_requirements(self.handle) }
    }

    pub fn write_data<T: Pod>(&mut self, data: &[T], offset: u64) {
        self.device_memory
            .as_mut()
            .unwrap()
            .write_data(data, offset * size_of::<T>() as u64);
    }

    pub fn get_device_address(&self) -> vk::DeviceAddress {
        let info = vk::BufferDeviceAddressInfoBuilder::new().buffer(self.handle);
        unsafe { self.device.get_buffer_device_address(&info) }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(Some(self.handle), None);
        }
    }
}
