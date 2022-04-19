use crate::vulkan::device::Device;
use erupt::vk;

pub struct Fence {
    handle: vk::Fence,
}

impl Fence {
    pub fn new(device: &Device) -> Self {
        let create_info = vk::FenceCreateInfo::default();
        let semaphore = unsafe { device.handle().create_fence(&create_info, None).unwrap() };

        Self { handle: semaphore }
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.handle().destroy_fence(self.handle, None);
        }
    }

    pub fn handle(&self) -> vk::Fence {
        self.handle
    }
}
