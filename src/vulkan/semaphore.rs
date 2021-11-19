use crate::vulkan::device::Device;
use erupt::vk;
use std::rc::Rc;

pub struct Semaphore {
    handle: vk::Semaphore,
    device: Rc<Device>,
}

impl Semaphore {
    pub fn new(device: Rc<Device>) -> Self {
        let create_info = vk::SemaphoreCreateInfoBuilder::default();
        let semaphore = unsafe { device.create_semaphore(&create_info, None).unwrap() };

        Semaphore {
            handle: semaphore,
            device,
        }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_semaphore(Some(self.handle), None);
        }
    }
}
