use crate::vulkan::device::Device;
use bytemuck::Pod;
use erupt::vk;
use gpu_alloc::MemoryBlock;
use gpu_alloc_erupt::EruptMemoryDevice;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::rc::Rc;

pub struct DeviceMemory {
    mem_block: ManuallyDrop<MemoryBlock<vk::DeviceMemory>>,
    device: Rc<Device>,
}

impl DeviceMemory {
    pub fn new(
        device: Rc<Device>,
        mem_reqs: vk::MemoryRequirements,
        allocation_flags: gpu_alloc::UsageFlags,
    ) -> Self {
        let mem_block = ManuallyDrop::new(device.gpu_alloc_memory(mem_reqs, allocation_flags));
        DeviceMemory { mem_block, device }
    }

    pub fn bind_to_image(&self, image: vk::Image) {
        unsafe {
            self.device
                .bind_image_memory(image, *self.mem_block.memory(), self.mem_block.offset())
                .unwrap();
        }
    }

    pub fn bind_to_buffer(&self, buffer: vk::Buffer) {
        unsafe {
            self.device
                .bind_buffer_memory(buffer, *self.mem_block.memory(), self.mem_block.offset())
                .unwrap();
        }
    }

    pub fn map(&mut self, offset: u64, size: usize) -> NonNull<u8> {
        unsafe {
            self.mem_block
                .map(EruptMemoryDevice::wrap(&self.device), offset, size)
                .unwrap()
        }
    }

    pub fn unmap(&mut self) {
        unsafe {
            self.mem_block.unmap(EruptMemoryDevice::wrap(&self.device));
        }
    }

    pub fn write_data<T: Pod>(&mut self, data: &[T], offset: u64) {
        unsafe {
            self.mem_block
                .write_bytes(
                    EruptMemoryDevice::wrap(&self.device),
                    offset,
                    bytemuck::cast_slice(data),
                )
                .unwrap()
        }
    }
}

impl Drop for DeviceMemory {
    fn drop(&mut self) {
        unsafe {
            self.device
                .gpu_dealloc_memory(ManuallyDrop::take(&mut self.mem_block));
        }
    }
}
