use crate::vulkan::device::Device;
use erupt::vk;
use std::rc::Rc;

pub struct Fence {
    handle: vk::Fence,
    device: Rc<Device>,
}

impl Fence {
    pub fn new(device: Rc<Device>, signaled: bool) -> Self {
        let create_info = vk::FenceCreateInfoBuilder::new().flags(if signaled {
            vk::FenceCreateFlags::SIGNALED
        } else {
            vk::FenceCreateFlags::empty()
        });
        let fence = unsafe { device.create_fence(&create_info, None).unwrap() };

        Fence {
            handle: fence,
            device,
        }
    }

    pub fn reset(&self) {
        unsafe {
            self.device.reset_fences(&[self.handle]).unwrap();
        }
    }

    pub fn wait(&self, timeout: u64) {
        unsafe {
            self.device
                .wait_for_fences(&[self.handle], true, timeout)
                .unwrap()
        }
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_fence(Some(self.handle), None);
        }
    }
}
