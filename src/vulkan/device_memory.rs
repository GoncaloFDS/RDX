use crate::vulkan::device::Device;
use erupt::vk;
use gpu_alloc::MemoryBlock;
use std::mem::ManuallyDrop;
use std::rc::Rc;

pub struct DeviceMemory {
    mem_block: ManuallyDrop<MemoryBlock<vk::DeviceMemory>>,
    device: Rc<Device>,
}

impl DeviceMemory {
    pub fn new(device: Rc<Device>, mem_reqs: vk::MemoryRequirements) -> Self {
        let mem_block = ManuallyDrop::new(device.gpu_alloc_memory(mem_reqs));
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
}

impl Drop for DeviceMemory {
    fn drop(&mut self) {
        unsafe {
            self.device
                .gpu_dealloc_memory(ManuallyDrop::take(&mut self.mem_block));
        }
    }
}
