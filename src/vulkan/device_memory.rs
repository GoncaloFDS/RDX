use crate::vulkan::device::Device;
use erupt::vk;
use gpu_alloc::MemoryBlock;
use gpu_alloc_erupt::EruptMemoryDevice;
use std::mem::{size_of, size_of_val, ManuallyDrop};
use std::ptr::NonNull;

pub struct DeviceMemory {
    memory_block: ManuallyDrop<MemoryBlock<vk::DeviceMemory>>,
}

impl DeviceMemory {
    pub fn new(
        device: &mut Device,
        memory_requirements: vk::MemoryRequirements,
        allocation_flags: gpu_alloc::UsageFlags,
    ) -> Self {
        let memory_block =
            ManuallyDrop::new(device.allocate_memory(memory_requirements, allocation_flags));
        DeviceMemory { memory_block }
    }

    pub unsafe fn destroy(&mut self, device: &mut Device) {
        device.dealloc_memory(ManuallyDrop::take(&mut self.memory_block))
    }

    pub fn bind_to_buffer(&self, device: &Device, buffer: vk::Buffer) {
        unsafe {
            device
                .handle()
                .bind_buffer_memory(
                    buffer,
                    *self.memory_block.memory(),
                    self.memory_block.offset(),
                )
                .unwrap();
        }
    }

    pub fn bind_to_image(&self, device: &Device, image: vk::Image) {
        unsafe {
            device
                .handle()
                .bind_image_memory(
                    image,
                    *self.memory_block.memory(),
                    self.memory_block.offset(),
                )
                .unwrap();
        }
    }

    pub fn map(&mut self, device: &Device, offset: u64, size: usize) -> NonNull<u8> {
        unsafe {
            self.memory_block
                .map(EruptMemoryDevice::wrap(device.handle()), offset, size)
                .unwrap()
        }
    }

    pub fn unmap(&mut self, device: &Device) {
        unsafe {
            self.memory_block
                .unmap(EruptMemoryDevice::wrap(device.handle()));
        }
    }

    pub fn write_data<T>(&mut self, device: &Device, data: &[T], offset: u64) {
        unsafe {
            self.memory_block
                .write_bytes(
                    EruptMemoryDevice::wrap(device.handle()),
                    offset,
                    cast_slice(data),
                )
                .unwrap()
        }
    }
}

unsafe fn cast_slice<T>(p: &[T]) -> &[u8] {
    if size_of::<T>() == size_of::<u8>() {
        std::slice::from_raw_parts(p.as_ptr() as *const u8, p.len())
    } else if size_of_val(p) % size_of::<u8>() == 0 {
        let new_len = size_of_val(p) / size_of::<u8>();
        std::slice::from_raw_parts(p.as_ptr() as *const u8, new_len)
    } else {
        panic!("can't cast slice!")
    }
}
