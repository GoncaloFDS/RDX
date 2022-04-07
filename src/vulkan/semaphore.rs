use crate::vulkan::device::Device;
use erupt::{vk, DeviceLoader};

pub struct Semaphore {
    handle: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: &DeviceLoader) -> Self {
        let create_info = vk::SemaphoreCreateInfo::default();
        let semaphore = unsafe { device.create_semaphore(&create_info, None).unwrap() };

        Self { handle: semaphore }
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.handle().destroy_semaphore(self.handle, None);
        }
    }

    pub fn handle(&self) -> vk::Semaphore {
        self.handle
    }
}
