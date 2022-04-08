use crate::vulkan::device::Device;
use crate::vulkan::device_memory::DeviceMemory;
use erupt::vk;
use std::mem::size_of;
use std::mem::size_of_val;

pub struct Buffer {
    handle: vk::Buffer,
    device_memory: Option<DeviceMemory>,
}

impl Buffer {
    pub fn empty(
        device: &mut Device,
        size: u64,
        usage: vk::BufferUsageFlags,
        allocation_flags: gpu_alloc::UsageFlags,
    ) -> Self {
        let create_info = vk::BufferCreateInfoBuilder::new()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { device.handle().create_buffer(&create_info, None).unwrap() };

        let memory_requirements = get_memory_requiremets(&device, buffer);

        let device_memory = DeviceMemory::new(
            device,
            memory_requirements.memory_requirements,
            allocation_flags,
        );
        device_memory.bind_to_buffer(device, buffer);

        Buffer {
            handle: buffer,
            device_memory: Some(device_memory),
        }
    }

    pub fn with_data<T>(device: &mut Device, data: &[T], usage: vk::BufferUsageFlags) -> Self {
        let size = size_of_val(data) as u64;

        let mut buffer = Buffer::empty(
            device,
            size,
            usage,
            gpu_alloc::UsageFlags::HOST_ACCESS | gpu_alloc::UsageFlags::DEVICE_ADDRESS,
        );

        buffer.write_data(device, data, 0);

        buffer
    }

    pub fn destroy(&mut self, device: &mut Device) {
        unsafe {
            if let Some(memory) = self.device_memory.as_mut() {
                memory.destroy(device)
            }
            device.handle().destroy_buffer(self.handle, None);
        }
    }

    pub fn handle(&self) -> vk::Buffer {
        self.handle
    }

    pub fn device_memory(&self) -> &Option<DeviceMemory> {
        &self.device_memory
    }

    pub fn write_data<T>(&mut self, device: &Device, data: &[T], offset: u64) {
        let memory = self.device_memory.as_mut().unwrap();
        memory.write_data(device, data, offset * size_of::<T>() as u64)
    }
}

fn get_memory_requiremets(device: &Device, buffer: vk::Buffer) -> vk::MemoryRequirements2 {
    let info = vk::BufferMemoryRequirementsInfo2Builder::new().buffer(buffer);
    unsafe { device.handle().get_buffer_memory_requirements2(&info, None) }
}
